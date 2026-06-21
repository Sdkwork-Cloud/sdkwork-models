use crate::domain::{
    BillingMeter, DecimalValue, DomainError, DomainResult, ModelPrice, ModelVendor, Money,
    PriceSide,
};
use crate::ports::PricingCatalog;

const DEFAULT_PRICE_REGION_CODE: &str = "global";

pub struct PricingResolver<'a, C: PricingCatalog> {
    catalog: &'a C,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveModelPriceQuery {
    pub api_key_id: i64,
    pub channel_group_id: Option<i64>,
    pub model: String,
    pub billing_meter: BillingMeter,
    pub provider_code: Option<String>,
    pub channel_id: Option<i64>,
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
    pub vendor: ModelVendor,
    pub group_code: String,
    pub pricing_plan_code: String,
    pub provider_code: Option<String>,
    pub billing_meter: BillingMeter,
    pub official_reference: ModelPrice,
    pub upstream_cost: Option<ModelPrice>,
    pub customer_charge_before_rate: Money,
    pub rate_multiplier: DecimalValue,
    pub reference_multiplier: DecimalValue,
    pub customer_charge: Money,
    pub gross_margin_per_unit: Option<crate::domain::DecimalValue>,
    pub source: ResolvedPriceSource,
}

impl<'a, C: PricingCatalog> PricingResolver<'a, C> {
    pub fn new(catalog: &'a C) -> Self {
        Self { catalog }
    }

    pub fn resolve(&self, query: ResolveModelPriceQuery) -> DomainResult<ResolvedModelPrice> {
        let api_key = self.find_api_key(query.api_key_id)?;
        let group = self.find_group(query.channel_group_id.unwrap_or(api_key.group_id))?;
        // Verify the channel group belongs to the same tenant/organization as the API key,
        // or is a global resource (tenant_id == 0)
        if group.tenant_id != 0 && group.tenant_id != api_key.tenant_id {
            return Err(DomainError::new(format!(
                "channel group {} does not belong to tenant {}",
                group.id, api_key.tenant_id
            )));
        }
        if group.organization_id != 0 && group.organization_id != api_key.organization_id {
            return Err(DomainError::new(format!(
                "channel group {} does not belong to organization {}",
                group.id, api_key.organization_id
            )));
        }
        let plan = self.find_plan(&group.pricing_plan_code)?;
        let model = self.find_model(&query.model)?;
        let vendor = self.find_vendor(&model.vendor_code)?;
        let explicit_region_code = query
            .region_code
            .as_deref()
            .and_then(normalized_optional_region_code);
        let route = match query.provider_code.as_deref() {
            Some(provider_code) => Some(self.find_provider_route(
                &query.model,
                provider_code,
                query.channel_id,
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
        let upstream = self.find_upstream_cost(&query, &region_code);
        let price_scope = upstream
            .as_ref()
            .map(|price| price.catalog_key.as_str())
            .unwrap_or(query.model.as_str());
        let official = self.find_official_reference(&query, price_scope, &region_code)?;

        let explicit_customer = self
            .catalog
            .find_model_price(
                price_scope,
                PriceSide::CustomerCharge,
                query.billing_meter.clone(),
                None,
                Some(&plan.plan_code),
            )
            .filter(|price| same_region(&price.region_code, &region_code));
        let reference_multiplier = plan
            .default_multiplier
            .checked_multiply(group.official_price_multiplier)?;
        let (customer_charge_before_rate, source) = match explicit_customer {
            Some(price) => (
                price.unit_price,
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
        let customer_charge =
            customer_charge_before_rate.checked_multiply(group.rate_multiplier)?;
        let gross_margin_per_unit = upstream
            .as_ref()
            .map(|price| customer_charge.subtract(&price.unit_price))
            .transpose()?;

        Ok(ResolvedModelPrice {
            model: model.model,
            vendor: vendor.vendor,
            group_code: group.code,
            pricing_plan_code: plan.plan_code,
            provider_code: query.provider_code,
            billing_meter: query.billing_meter,
            official_reference: official,
            upstream_cost: upstream,
            customer_charge_before_rate,
            rate_multiplier: group.rate_multiplier,
            reference_multiplier,
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

    fn find_group(&self, group_id: i64) -> DomainResult<crate::domain::ChannelGroup> {
        self.catalog
            .find_channel_group(group_id)
            .ok_or_else(|| DomainError::new(format!("channel group not found: {group_id}")))
    }

    fn find_plan(&self, plan_code: &str) -> DomainResult<crate::domain::PricingPlan> {
        self.catalog
            .find_pricing_plan(plan_code)
            .ok_or_else(|| DomainError::new(format!("pricing plan not found: {plan_code}")))
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
        price_scope: &str,
        region_code: &str,
    ) -> DomainResult<ModelPrice> {
        self.catalog
            .list_model_prices(
                price_scope,
                PriceSide::OfficialReference,
                query.billing_meter.clone(),
            )
            .into_iter()
            .find(|price| {
                price.pricing_plan_code.is_none() && same_region(&price.region_code, region_code)
            })
            .ok_or_else(|| {
                DomainError::new(format!(
                    "official reference price not found for model {} meter {} and region {}",
                    query.model,
                    query.billing_meter.code(),
                    region_code
                ))
            })
    }

    fn find_upstream_cost(
        &self,
        query: &ResolveModelPriceQuery,
        region_code: &str,
    ) -> Option<ModelPrice> {
        let provider_code = query.provider_code.as_deref();
        if let Some(channel_id) = query.channel_id {
            return self
                .catalog
                .list_model_prices(
                    &query.model,
                    PriceSide::UpstreamCost,
                    query.billing_meter.clone(),
                )
                .into_iter()
                .find(|price| {
                    price.provider_code.as_deref() == provider_code
                        && price.channel_id == Some(channel_id)
                        && price.pricing_plan_code.is_none()
                        && same_region(&price.region_code, region_code)
                });
        }

        self.catalog
            .list_model_prices(
                &query.model,
                PriceSide::UpstreamCost,
                query.billing_meter.clone(),
            )
            .into_iter()
            .find(|price| {
                price.provider_code.as_deref() == provider_code
                    && price.pricing_plan_code.is_none()
                    && same_region(&price.region_code, region_code)
            })
    }

    fn find_provider_route(
        &self,
        model: &str,
        provider_code: &str,
        channel_id: Option<i64>,
        region_code: Option<&str>,
    ) -> DomainResult<crate::domain::ModelProviderRoute> {
        if let Some(route) = self
            .catalog
            .list_provider_routes(model)
            .into_iter()
            .find(|route| {
                route.provider_code == provider_code
                    && channel_id
                        .map(|channel_id| route.channel_id == channel_id)
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
            .list_provider_channel_routes()
            .into_iter()
            .find(|route| {
                route.provider_code == provider_code
                    && channel_id
                        .map(|channel_id| route.channel_id == channel_id)
                        .unwrap_or(true)
                    && region_code
                        .map(|region_code| same_region(&route.region_code, region_code))
                        .unwrap_or(true)
            })
        {
            return Ok(crate::domain::ModelProviderRoute::new_for_catalog_key(
                model,
                model,
                &route.provider_code,
                route.channel_id,
                model,
            )
            .with_region_code(&route.region_code)
            .with_provider_endpoint(route.base_url, route.secret_ref)
            .with_auth_profile(route.auth_profile));
        }

        Err(if let Some(channel_id) = channel_id {
            DomainError::new(format!(
                "provider route not found for model {model}, provider {provider_code}, and channel {channel_id}"
            ))
        } else {
            DomainError::new(format!(
                "provider route not found for model {model} and provider {provider_code}"
            ))
        })
    }
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
