use crate::domain::{
    BillingMeter, DecimalValue, DomainError, DomainResult, ModelPrice, ModelVendor, Money,
    PriceSide, PricingDimensionContext,
};
use crate::ports::PricingCatalog;

const DEFAULT_PRICE_REGION_CODE: &str = "global";

pub struct PricingResolver<'a, C: PricingCatalog> {
    catalog: &'a C,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveModelPriceQuery {
    pub api_key_id: i64,
    pub account_group_id: Option<i64>,
    pub model: String,
    pub billing_meter: BillingMeter,
    pub supplier_code: Option<String>,
    pub account_id: Option<i64>,
    pub region_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedPriceSource {
    ExplicitCustomerCharge,
    DerivedFromOfficialReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModelPrice {
    pub model: String,
    pub vendor_code: String,
    pub vendor: ModelVendor,
    pub group_code: String,
    pub pricing_plan_code: String,
    pub supplier_code: Option<String>,
    pub billing_meter: BillingMeter,
    pub official_reference: ModelPrice,
    pub raw_upstream_cost: Option<ModelPrice>,
    pub raw_customer_charge: Option<ModelPrice>,
    pub procurement_cost: Option<Money>,
    pub account_contract_cost_multiplier: Option<DecimalValue>,
    pub account_group_cost_multiplier: Option<DecimalValue>,
    pub procurement_cost_multiplier: Option<DecimalValue>,
    pub customer_charge_before_sale_multiplier: Money,
    pub sale_multiplier: DecimalValue,
    pub reference_multiplier: DecimalValue,
    pub default_markup_amount: Money,
    pub customer_charge: Money,
    pub gross_margin_per_unit: Option<crate::domain::DecimalValue>,
    pub source: ResolvedPriceSource,
}

impl<'a, C: PricingCatalog> PricingResolver<'a, C> {
    pub fn new(catalog: &'a C) -> Self {
        Self { catalog }
    }

    pub fn resolve(&self, query: ResolveModelPriceQuery) -> DomainResult<ResolvedModelPrice> {
        self.resolve_with_dimensions(query, &PricingDimensionContext::default())
    }

    pub fn resolve_with_dimensions(
        &self,
        query: ResolveModelPriceQuery,
        dimensions: &PricingDimensionContext,
    ) -> DomainResult<ResolvedModelPrice> {
        // Auth-token sessions (api_key_id == 0) carry no gateway API key
        // record: the resolved account group is the authoritative
        // tenant/organization scope, mirroring the billing-subject semantics
        // of the open-api billing guard.
        let api_key = (query.api_key_id > 0)
            .then(|| self.find_api_key(query.api_key_id))
            .transpose()?;
        let group = self.find_group(
            query
                .account_group_id
                .or_else(|| api_key.as_ref().map(|key| key.default_account_group_id))
                .ok_or_else(|| {
                    DomainError::new(
                        "account group id is required when no API key backs the pricing session",
                    )
                })?,
        )?;
        // Verify the account group belongs to the same tenant/organization as the API key,
        // or is a global resource (tenant_id == 0); auth-token sessions have no
        // API key to compare against, so the group itself is authoritative.
        if let Some(api_key) = api_key.as_ref() {
            if group.tenant_id != 0 && group.tenant_id != api_key.tenant_id {
                return Err(DomainError::new(format!(
                    "account group {} does not belong to tenant {}",
                    group.id, api_key.tenant_id
                )));
            }
            if group.organization_id != 0 && group.organization_id != api_key.organization_id {
                return Err(DomainError::new(format!(
                    "account group {} does not belong to organization {}",
                    group.id, api_key.organization_id
                )));
            }
        }
        let (tenant_id, organization_id) = match api_key.as_ref() {
            Some(api_key) => (api_key.tenant_id, api_key.organization_id),
            None => (group.tenant_id, group.organization_id),
        };
        let plan = self.find_plan(tenant_id, organization_id, &group)?;
        let model = self.find_model(&query.model)?;
        let vendor = self.find_vendor(&model.vendor_code)?;
        let explicit_region_code = query
            .region_code
            .as_deref()
            .and_then(normalized_optional_region_code);
        let route = match query.supplier_code.as_deref() {
            Some(supplier_code) => Some(self.find_upstream_route(
                &query.model,
                supplier_code,
                query.account_id,
                explicit_region_code.as_deref(),
            )?),
            None => None,
        };
        let region_code = explicit_region_code
            .or_else(|| {
                route
                    .as_ref()
                    .map(|route| normalize_region_code(&route.region_code))
            })
            .unwrap_or_else(|| DEFAULT_PRICE_REGION_CODE.to_owned());
        let mut rate_dimensions = dimensions.clone();
        rate_dimensions.insert(
            "vendor_code",
            serde_json::json!(vendor.vendor_code.as_str()),
        );
        rate_dimensions.insert("region_code", serde_json::json!(region_code.as_str()));
        rate_dimensions.insert("catalog_key", serde_json::json!(model.catalog_key.as_str()));
        rate_dimensions.insert("model", serde_json::json!(model.model.as_str()));
        rate_dimensions.insert("meter_code", serde_json::json!(query.billing_meter.code()));
        if let Some(supplier_code) = query.supplier_code.as_deref() {
            rate_dimensions.insert("provider_code", serde_json::json!(supplier_code));
        }
        let raw_upstream_cost =
            self.find_upstream_cost(&query, tenant_id, organization_id, &region_code)?;
        let price_scope = raw_upstream_cost
            .as_ref()
            .map(|price| price.catalog_key.as_str())
            .unwrap_or(query.model.as_str());
        let official = self.find_official_reference(
            &query,
            tenant_id,
            organization_id,
            price_scope,
            &region_code,
            &rate_dimensions,
        )?;
        if query.supplier_code.is_some() && raw_upstream_cost.is_none() {
            return Err(missing_upstream_cost_error(&query, &region_code));
        }
        let procurement_multipliers = self.resolve_procurement_multipliers(
            &query,
            group.id,
            group.cost_multiplier,
            &region_code,
        )?;
        let procurement_cost = match (&raw_upstream_cost, procurement_multipliers.as_ref()) {
            (Some(price), Some(multipliers)) => Some(
                price
                    .unit_price
                    .checked_multiply(multipliers.combined_multiplier)?,
            ),
            (None, None) => None,
            _ => {
                return Err(DomainError::new(
                    "upstream price and procurement multiplier context must be resolved together",
                ));
            }
        };

        let explicit_customer = self
            .catalog
            .find_model_price_for_scope(
                tenant_id,
                organization_id,
                price_scope,
                PriceSide::CustomerCharge,
                query.billing_meter.clone(),
                None,
                Some(&plan.plan_code),
            )
            .filter(|price| same_region(&price.region_code, &region_code));
        let reference_multiplier = plan.default_multiplier;
        let (customer_charge_before_sale_multiplier, source) = match explicit_customer.as_ref() {
            Some(price) => (
                price.unit_price.clone(),
                ResolvedPriceSource::ExplicitCustomerCharge,
            ),
            None => (
                add_default_markup(
                    official.unit_price.checked_multiply(reference_multiplier)?,
                    &plan.default_markup_amount,
                )?,
                ResolvedPriceSource::DerivedFromOfficialReference,
            ),
        };
        require_positive_multiplier("account group sale multiplier", group.sale_multiplier)?;
        let customer_charge =
            customer_charge_before_sale_multiplier.checked_multiply(group.sale_multiplier)?;
        let gross_margin_per_unit = procurement_cost
            .as_ref()
            .map(|cost| customer_charge.subtract(cost))
            .transpose()?;

        Ok(ResolvedModelPrice {
            model: model.model,
            vendor_code: model.vendor_code,
            vendor: vendor.vendor,
            group_code: group.code,
            pricing_plan_code: plan.plan_code,
            supplier_code: query.supplier_code,
            billing_meter: query.billing_meter,
            official_reference: official,
            raw_upstream_cost,
            raw_customer_charge: explicit_customer,
            procurement_cost,
            account_contract_cost_multiplier: procurement_multipliers
                .as_ref()
                .map(|multipliers| multipliers.account_contract_multiplier),
            account_group_cost_multiplier: procurement_multipliers
                .as_ref()
                .map(|multipliers| multipliers.account_group_multiplier),
            procurement_cost_multiplier: procurement_multipliers
                .as_ref()
                .map(|multipliers| multipliers.combined_multiplier),
            customer_charge_before_sale_multiplier,
            sale_multiplier: group.sale_multiplier,
            reference_multiplier,
            default_markup_amount: plan.default_markup_amount,
            customer_charge,
            gross_margin_per_unit,
            source,
        })
    }

    fn find_api_key(&self, api_key_id: i64) -> DomainResult<crate::domain::GatewayApiKey> {
        self.catalog
            .find_api_key(api_key_id)
            .ok_or_else(|| DomainError::new(format!("api key not found: {api_key_id}")))
    }

    fn find_group(
        &self,
        account_group_id: i64,
    ) -> DomainResult<crate::domain::UpstreamAccountGroup> {
        self.catalog
            .find_upstream_account_group(account_group_id)
            .ok_or_else(|| DomainError::new(format!("account group not found: {account_group_id}")))
    }

    fn find_plan(
        &self,
        tenant_id: i64,
        organization_id: i64,
        group: &crate::domain::UpstreamAccountGroup,
    ) -> DomainResult<crate::domain::PricingPlan> {
        let plan = if group.pricing_plan_id > 0 {
            self.catalog.find_pricing_plan_by_identity(
                group.pricing_plan_tenant_id,
                group.pricing_plan_organization_id,
                group.pricing_plan_id,
                &group.pricing_plan_code,
            )
        } else {
            self.catalog.find_pricing_plan_for_scope(
                tenant_id,
                organization_id,
                &group.pricing_plan_code,
            )
        };
        plan.ok_or_else(|| {
            DomainError::new(format!(
                "pricing plan not found: {}",
                group.pricing_plan_code
            ))
        })
    }

    fn find_model(&self, model: &str) -> DomainResult<crate::domain::AiModel> {
        self.catalog
            .find_model(model)
            .ok_or_else(|| DomainError::new(format!("model not found: {model}")))
    }

    fn find_vendor(&self, vendor_code: &str) -> DomainResult<crate::domain::ModelVendorDefinition> {
        self.catalog
            .find_vendor(vendor_code)
            .ok_or_else(|| DomainError::new(format!("model vendor not found: {vendor_code}")))
    }

    fn find_official_reference(
        &self,
        query: &ResolveModelPriceQuery,
        tenant_id: i64,
        organization_id: i64,
        price_scope: &str,
        region_code: &str,
        dimensions: &PricingDimensionContext,
    ) -> DomainResult<ModelPrice> {
        let mut candidates = self
            .catalog
            .list_model_prices_for_scope(
                tenant_id,
                organization_id,
                price_scope,
                PriceSide::OfficialReference,
                query.billing_meter.clone(),
            )
            .into_iter()
            .filter(|price| {
                price.pricing_plan_code.is_none() && same_region(&price.region_code, region_code)
            })
            .filter(|price| {
                price
                    .rate_metadata
                    .as_ref()
                    .is_none_or(|metadata| metadata.matches(dimensions))
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            rate_specificity(right)
                .cmp(&rate_specificity(left))
                .then_with(|| rate_priority(left).cmp(&rate_priority(right)))
                .then_with(|| rate_hash(left).cmp(rate_hash(right)))
        });
        let selected = candidates.first().cloned().ok_or_else(|| {
            DomainError::new(format!(
                "official reference price not found for model {} meter {} and region {}",
                query.model,
                query.billing_meter.code(),
                region_code
            ))
        })?;
        if let Some(next) = candidates.get(1) {
            let same_rank = rate_specificity(&selected) == rate_specificity(next)
                && rate_priority(&selected) == rate_priority(next);
            let distinct_rates = rate_hash(&selected) != rate_hash(next);
            if same_rank && distinct_rates {
                return Err(DomainError::new(format!(
                    "official reference rate ambiguous for model {} meter {} and region {}",
                    query.model,
                    query.billing_meter.code(),
                    region_code
                )));
            }
        }
        Ok(selected)
    }

    fn find_upstream_cost(
        &self,
        query: &ResolveModelPriceQuery,
        tenant_id: i64,
        organization_id: i64,
        region_code: &str,
    ) -> DomainResult<Option<ModelPrice>> {
        let supplier_code = query.supplier_code.as_deref();
        if supplier_code.is_none() && query.account_id.is_none() {
            return Ok(None);
        }
        let supplier_code = supplier_code.ok_or_else(|| {
            DomainError::new("supplier code is required when an upstream account is selected")
        })?;
        let account_id = query.account_id.ok_or_else(|| {
            DomainError::new("upstream account id is required when a supplier is selected")
        })?;
        Ok(self
            .catalog
            .list_model_prices_for_scope(
                tenant_id,
                organization_id,
                &query.model,
                PriceSide::UpstreamCost,
                query.billing_meter.clone(),
            )
            .into_iter()
            .find(|price| {
                price.supplier_code.as_deref() == Some(supplier_code)
                    && price.account_id == Some(account_id)
                    && price.pricing_plan_code.is_none()
                    && same_region(&price.region_code, region_code)
            }))
    }

    fn resolve_procurement_multipliers(
        &self,
        query: &ResolveModelPriceQuery,
        account_group_id: i64,
        default_group_multiplier: DecimalValue,
        region_code: &str,
    ) -> DomainResult<Option<ProcurementMultipliers>> {
        if query.supplier_code.is_none() && query.account_id.is_none() {
            return Ok(None);
        }
        let supplier_code = query.supplier_code.as_deref().ok_or_else(|| {
            DomainError::new("supplier code is required when an upstream account is selected")
        })?;
        let account_id = query.account_id.ok_or_else(|| {
            DomainError::new("upstream account id is required when a supplier is selected")
        })?;

        let mut resolved: Option<ProcurementMultipliers> = None;
        for route in self
            .catalog
            .list_upstream_account_routes()
            .into_iter()
            .filter(|route| {
                route.supplier_code == supplier_code
                    && route.account_id == account_id
                    && same_region(&route.region_code, region_code)
            })
        {
            let Some(binding) = route
                .account_group_bindings
                .iter()
                .find(|binding| binding.account_group_id == account_group_id)
            else {
                continue;
            };
            let account_group_multiplier = binding
                .cost_multiplier_override
                .unwrap_or(default_group_multiplier);
            require_positive_multiplier(
                "upstream account contract cost multiplier",
                route.contract_cost_multiplier,
            )?;
            require_positive_multiplier(
                "upstream account group cost multiplier",
                account_group_multiplier,
            )?;
            let multipliers = ProcurementMultipliers {
                account_contract_multiplier: route.contract_cost_multiplier,
                account_group_multiplier,
                combined_multiplier: route
                    .contract_cost_multiplier
                    .checked_multiply(account_group_multiplier)?,
            };
            if resolved
                .as_ref()
                .is_some_and(|current| current != &multipliers)
            {
                return Err(DomainError::new(format!(
                    "inconsistent procurement multipliers for supplier {supplier_code}, account {account_id}, and account group {account_group_id}"
                )));
            }
            resolved = Some(multipliers);
        }

        resolved.map(Some).ok_or_else(|| {
            DomainError::new(format!(
                "upstream account {account_id} is not bound to account group {account_group_id} for supplier {supplier_code} and region {region_code}"
            ))
        })
    }

    fn find_upstream_route(
        &self,
        model: &str,
        supplier_code: &str,
        account_id: Option<i64>,
        region_code: Option<&str>,
    ) -> DomainResult<crate::domain::ModelUpstreamRoute> {
        if let Some(route) = self
            .catalog
            .list_model_upstream_routes(model)
            .into_iter()
            .find(|route| {
                route.supplier_code == supplier_code
                    && account_id
                        .map(|account_id| route.account_id == account_id)
                        .unwrap_or(true)
                    && region_code
                        .map(|region_code| same_region(&route.region_code, region_code))
                        .unwrap_or(true)
            })
        {
            return Ok(route);
        }

        if let Some(route) = self
            .catalog
            .list_upstream_account_routes()
            .into_iter()
            .find(|route| {
                route.supplier_code == supplier_code
                    && account_id
                        .map(|account_id| route.account_id == account_id)
                        .unwrap_or(true)
                    && region_code
                        .map(|region_code| same_region(&route.region_code, region_code))
                        .unwrap_or(true)
            })
        {
            return Ok(crate::domain::ModelUpstreamRoute::new_for_catalog_key(
                model,
                model,
                &route.supplier_code,
                route.account_id,
                model,
            )
            .with_region_code(&route.region_code)
            .with_upstream_endpoint(route.base_url, route.secret_ref)
            .with_auth_profile(route.auth_profile));
        }

        Err(if let Some(account_id) = account_id {
            DomainError::new(format!(
                "upstream route not found for model {model}, supplier {supplier_code}, and account {account_id}"
            ))
        } else {
            DomainError::new(format!(
                "upstream route not found for model {model} and supplier {supplier_code}"
            ))
        })
    }
}

fn rate_specificity(price: &ModelPrice) -> usize {
    price
        .rate_metadata
        .as_ref()
        .map(|metadata| metadata.condition_count())
        .unwrap_or_default()
}

fn rate_priority(price: &ModelPrice) -> i32 {
    price
        .rate_metadata
        .as_ref()
        .map(|metadata| metadata.priority)
        .unwrap_or(i32::MAX)
}

fn rate_hash(price: &ModelPrice) -> &str {
    price
        .rate_metadata
        .as_ref()
        .map(|metadata| metadata.rate_hash.as_str())
        .unwrap_or("")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProcurementMultipliers {
    account_contract_multiplier: DecimalValue,
    account_group_multiplier: DecimalValue,
    combined_multiplier: DecimalValue,
}

fn normalize_region_code(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "global".to_owned()
    } else {
        value.to_owned()
    }
}

fn normalized_optional_region_code(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(normalize_region_code(value))
    }
}

fn same_region(actual: &str, expected: &str) -> bool {
    normalize_region_code(actual).eq_ignore_ascii_case(&normalize_region_code(expected))
}

fn add_default_markup(base: Money, markup: &Money) -> DomainResult<Money> {
    if markup.is_zero() && base.currency != markup.currency {
        return Ok(base);
    }
    base.add(markup)
}

fn require_positive_multiplier(field: &str, value: DecimalValue) -> DomainResult<()> {
    if value <= DecimalValue::ZERO {
        return Err(DomainError::new(format!("{field} must be positive")));
    }
    Ok(())
}

fn missing_upstream_cost_error(query: &ResolveModelPriceQuery, region_code: &str) -> DomainError {
    DomainError::new(format!(
        "upstream cost not found for model {}, supplier {}, account {}, meter {}, and region {}",
        query.model,
        query.supplier_code.as_deref().unwrap_or("<missing>"),
        query
            .account_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_owned()),
        query.billing_meter.code(),
        region_code
    ))
}
