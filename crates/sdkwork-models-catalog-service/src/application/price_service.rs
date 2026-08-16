use serde_json::json;

use crate::application::{
    BillingStrategyKind, BillingStrategyRegistry, BillingStructure, PricingResolver,
    ResolveModelPriceQuery, ResolvedModelPrice,
};
use crate::domain::{DomainError, DomainResult, PricingDimensionContext, ResourceDefinition};
use crate::ports::PricingCatalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceResolutionStatus {
    Quoted,
    Rated,
    NonChargeable,
    Unrated,
}

impl PriceResolutionStatus {
    pub fn code(self) -> &'static str {
        match self {
            Self::Quoted => "quoted",
            Self::Rated => "rated",
            Self::NonChargeable => "non_chargeable",
            Self::Unrated => "unrated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceBillability {
    Chargeable,
    Free,
    NotApplicable,
    Unknown,
}

impl ResourceBillability {
    pub fn code(self) -> &'static str {
        match self {
            Self::Chargeable => "chargeable",
            Self::Free => "free",
            Self::NotApplicable => "not_applicable",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceResolutionFailureCode {
    PriceNotFound,
    AmbiguousRate,
    ResourceMismatch,
    UnknownBillability,
    UnsupportedBillingStrategy,
}

impl PriceResolutionFailureCode {
    pub fn code(self) -> &'static str {
        match self {
            Self::PriceNotFound => "price_not_found",
            Self::AmbiguousRate => "ambiguous_rate",
            Self::ResourceMismatch => "resource_mismatch",
            Self::UnknownBillability => "unknown_billability",
            Self::UnsupportedBillingStrategy => "unsupported_billing_strategy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceResolutionFailure {
    pub code: PriceResolutionFailureCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRateIdentity {
    pub price_book_code: Option<String>,
    pub rate_hash: Option<String>,
    pub product_code: Option<String>,
    pub operation_code: Option<String>,
    pub vendor_code: String,
    pub provider_code: Option<String>,
    pub region_code: String,
    pub catalog_key: String,
    pub meter_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingAuditSnapshot {
    pub resource: ResourceDefinition,
    pub status: PriceResolutionStatus,
    pub billability: ResourceBillability,
    pub rate_identity: Option<ResolvedRateIdentity>,
    pub strategy: Option<BillingStrategyKind>,
    pub failure: Option<PriceResolutionFailure>,
}

impl PricingAuditSnapshot {
    pub fn to_json_value(&self) -> serde_json::Value {
        let dimensions = self
            .resource
            .dimensions
            .iter()
            .map(|(code, value)| (code.to_owned(), value.clone()))
            .collect::<serde_json::Map<_, _>>();
        json!({
            "resource": {
                "vendorCode": self.resource.vendor_code.as_deref(),
                "providerCode": self.resource.provider_code.as_deref(),
                "accountId": self.resource.account_id,
                "regionCode": self.resource.region_code.as_deref(),
                "catalogKey": self.resource.catalog_key.as_str(),
                "model": self.resource.model.as_deref(),
                "apiCode": self.resource.api_code.as_deref(),
                "productCode": self.resource.product_code.as_deref(),
                "operationCode": self.resource.operation_code.as_deref(),
                "meterCode": self.resource.meter.code(),
                "measuredQuantity": self.resource.measured_quantity.map(|value| value.to_fixed_string(12)),
                "occurredAt": self.resource.occurred_at.to_rfc3339(),
                "dimensions": dimensions,
            },
            "resolution": {
                "status": self.status.code(),
                "billability": self.billability.code(),
                "strategy": self.strategy.map(BillingStrategyKind::code),
                "failureCode": self.failure.as_ref().map(|failure| failure.code.code()),
                "failureMessage": self.failure.as_ref().map(|failure| failure.message.as_str()),
            },
            "rate": self.rate_identity.as_ref().map(|rate| json!({
                "priceBookCode": rate.price_book_code.as_deref(),
                "rateHash": rate.rate_hash.as_deref(),
                "productCode": rate.product_code.as_deref(),
                "operationCode": rate.operation_code.as_deref(),
                "vendorCode": rate.vendor_code.as_str(),
                "providerCode": rate.provider_code.as_deref(),
                "regionCode": rate.region_code.as_str(),
                "catalogKey": rate.catalog_key.as_str(),
                "meterCode": rate.meter_code.as_str(),
            })),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceResolution {
    pub status: PriceResolutionStatus,
    pub billability: ResourceBillability,
    pub resolved_price: Option<ResolvedModelPrice>,
    pub rate_identity: Option<ResolvedRateIdentity>,
    pub billing: Option<BillingStructure>,
    pub failure: Option<PriceResolutionFailure>,
    pub audit_snapshot: PricingAuditSnapshot,
}

#[derive(Clone)]
pub struct PriceService {
    strategies: BillingStrategyRegistry,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            strategies: BillingStrategyRegistry::standard(),
        }
    }

    pub fn with_strategies(strategies: BillingStrategyRegistry) -> Self {
        Self { strategies }
    }

    pub fn resolve<C: PricingCatalog>(
        &self,
        catalog: &C,
        resource: ResourceDefinition,
    ) -> DomainResult<PriceResolution> {
        validate_resource(&resource)?;
        let dimensions = resolution_dimensions(&resource);
        let resolved = match PricingResolver::new(catalog).resolve_with_dimensions(
            ResolveModelPriceQuery {
                api_key_id: resource.api_key_id,
                account_group_id: resource.account_group_id,
                model: resource.catalog_key.clone(),
                billing_meter: resource.meter.clone(),
                supplier_code: resource.provider_code.clone(),
                account_id: resource.account_id,
                region_code: resource.region_code.clone(),
            },
            &dimensions,
        ) {
            Ok(resolved) => resolved,
            Err(error) => {
                return match classify_resolution_error(&error) {
                    Some(code) => Ok(unrated(
                        resource,
                        ResourceBillability::Unknown,
                        code,
                        error.to_string(),
                        None,
                        None,
                    )),
                    None => Err(error),
                };
            }
        };
        self.rate_resolved(resource, resolved)
    }

    /// Applies billability and a registered billing strategy to a price that
    /// was resolved earlier. This supports streaming calls where the rate is
    /// fixed before dispatch and the measured quantity arrives at stream end.
    pub fn rate_resolved(
        &self,
        resource: ResourceDefinition,
        resolved: ResolvedModelPrice,
    ) -> DomainResult<PriceResolution> {
        validate_resource(&resource)?;
        let rate_identity = rate_identity(&resource, &resolved);
        if let Some(message) = resource_mismatch(&resource, &resolved) {
            return Ok(unrated(
                resource,
                ResourceBillability::Unknown,
                PriceResolutionFailureCode::ResourceMismatch,
                message,
                Some(resolved),
                Some(rate_identity),
            ));
        }
        let billability = billability(&resolved);
        match billability {
            ResourceBillability::Free | ResourceBillability::NotApplicable => Ok(resolution(
                resource,
                PriceResolutionStatus::NonChargeable,
                billability,
                Some(resolved),
                Some(rate_identity),
                None,
                None,
            )),
            ResourceBillability::Unknown => Ok(unrated(
                resource,
                ResourceBillability::Unknown,
                PriceResolutionFailureCode::UnknownBillability,
                "matched rate has unknown billability",
                Some(resolved),
                Some(rate_identity),
            )),
            ResourceBillability::Chargeable if resource.measured_quantity.is_none() => {
                Ok(resolution(
                    resource,
                    PriceResolutionStatus::Quoted,
                    billability,
                    Some(resolved),
                    Some(rate_identity),
                    None,
                    None,
                ))
            }
            ResourceBillability::Chargeable => {
                match self.strategies.calculate(&resource, &resolved) {
                    Ok(billing) => Ok(resolution(
                        resource,
                        PriceResolutionStatus::Rated,
                        billability,
                        Some(resolved),
                        Some(rate_identity),
                        Some(billing),
                        None,
                    )),
                    Err(error) => Ok(unrated(
                        resource,
                        billability,
                        PriceResolutionFailureCode::UnsupportedBillingStrategy,
                        error.to_string(),
                        Some(resolved),
                        Some(rate_identity),
                    )),
                }
            }
        }
    }
}

impl Default for PriceService {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_resource(resource: &ResourceDefinition) -> DomainResult<()> {
    if resource.catalog_key.trim().is_empty() {
        return Err(DomainError::new(
            "pricing resource catalog key must not be empty",
        ));
    }
    if resource.meter == crate::domain::BillingMeter::Unknown {
        return Err(DomainError::new("pricing resource meter must be known"));
    }
    if resource
        .measured_quantity
        .is_some_and(|quantity| quantity < crate::domain::DecimalValue::ZERO)
    {
        return Err(DomainError::new(
            "pricing resource measured quantity must not be negative",
        ));
    }
    Ok(())
}

fn resolution_dimensions(resource: &ResourceDefinition) -> PricingDimensionContext {
    let mut dimensions = resource.dimensions.clone();
    for (code, value) in [
        ("vendor_code", resource.vendor_code.as_deref()),
        ("provider_code", resource.provider_code.as_deref()),
        ("region_code", resource.region_code.as_deref()),
        ("catalog_key", Some(resource.catalog_key.as_str())),
        ("model", resource.model.as_deref()),
        ("api_code", resource.api_code.as_deref()),
        ("product_code", resource.product_code.as_deref()),
        ("operation_code", resource.operation_code.as_deref()),
        ("meter_code", Some(resource.meter.code())),
    ] {
        if let Some(value) = value {
            dimensions.insert(code, json!(value));
        }
    }
    dimensions.insert("occurred_at", json!(resource.occurred_at.to_rfc3339()));
    dimensions
}

fn classify_resolution_error(error: &DomainError) -> Option<PriceResolutionFailureCode> {
    let message = error.to_string();
    if message.contains("official reference rate ambiguous") {
        Some(PriceResolutionFailureCode::AmbiguousRate)
    } else if message.contains("official reference price not found")
        || message.contains("model not found")
        || message.contains("model is not available")
    {
        Some(PriceResolutionFailureCode::PriceNotFound)
    } else {
        None
    }
}

fn billability(resolved: &ResolvedModelPrice) -> ResourceBillability {
    match resolved
        .official_reference
        .rate_metadata
        .as_ref()
        .map(|metadata| metadata.billability.as_str())
        .unwrap_or("chargeable")
    {
        "chargeable" => ResourceBillability::Chargeable,
        "free" => ResourceBillability::Free,
        "not_applicable" => ResourceBillability::NotApplicable,
        _ => ResourceBillability::Unknown,
    }
}

fn resource_mismatch(
    resource: &ResourceDefinition,
    resolved: &ResolvedModelPrice,
) -> Option<String> {
    if resource.catalog_key != resolved.official_reference.catalog_key {
        return Some(format!(
            "pricing resource catalog mismatch: expected {}, resolved {}",
            resource.catalog_key, resolved.official_reference.catalog_key
        ));
    }
    if resource.meter != resolved.billing_meter {
        return Some(format!(
            "pricing resource meter mismatch: expected {}, resolved {}",
            resource.meter.code(),
            resolved.billing_meter.code()
        ));
    }
    if let Some(expected) = resource.vendor_code.as_deref() {
        let actual = resolved.vendor_code.as_str();
        if !expected.eq_ignore_ascii_case(actual) {
            return Some(format!(
                "pricing resource vendor mismatch: expected {expected}, resolved {actual}"
            ));
        }
    }
    if let Some(expected) = resource.provider_code.as_deref() {
        let actual = resolved.supplier_code.as_deref().unwrap_or("<none>");
        if !expected.eq_ignore_ascii_case(actual) {
            return Some(format!(
                "pricing resource provider mismatch: expected {expected}, resolved {actual}"
            ));
        }
    }
    if let Some(expected) = resource.region_code.as_deref() {
        let actual = resolved.official_reference.region_code.as_str();
        if !expected.eq_ignore_ascii_case(actual) {
            return Some(format!(
                "pricing resource region mismatch: expected {expected}, resolved {actual}"
            ));
        }
    }
    let metadata = resolved.official_reference.rate_metadata.as_ref()?;
    for (label, expected, actual) in [
        (
            "product",
            resource.product_code.as_deref(),
            metadata.product_code.as_str(),
        ),
        (
            "operation",
            resource.operation_code.as_deref(),
            metadata.operation_code.as_str(),
        ),
    ] {
        if expected.is_some_and(|expected| !expected.eq_ignore_ascii_case(actual)) {
            return Some(format!(
                "pricing resource {label} mismatch: expected {}, resolved {actual}",
                expected.unwrap_or_default()
            ));
        }
    }
    None
}

fn rate_identity(
    resource: &ResourceDefinition,
    resolved: &ResolvedModelPrice,
) -> ResolvedRateIdentity {
    let metadata = resolved.official_reference.rate_metadata.as_ref();
    ResolvedRateIdentity {
        price_book_code: metadata.map(|metadata| metadata.price_book_code.clone()),
        rate_hash: metadata.map(|metadata| metadata.rate_hash.clone()),
        product_code: metadata
            .map(|metadata| metadata.product_code.clone())
            .or_else(|| resource.product_code.clone()),
        operation_code: metadata
            .map(|metadata| metadata.operation_code.clone())
            .or_else(|| resource.operation_code.clone()),
        vendor_code: resolved.vendor_code.clone(),
        provider_code: resource.provider_code.clone(),
        region_code: resolved.official_reference.region_code.clone(),
        catalog_key: resolved.official_reference.catalog_key.clone(),
        meter_code: resolved.billing_meter.code().to_owned(),
    }
}

fn unrated(
    resource: ResourceDefinition,
    billability: ResourceBillability,
    code: PriceResolutionFailureCode,
    message: impl Into<String>,
    resolved_price: Option<ResolvedModelPrice>,
    rate_identity: Option<ResolvedRateIdentity>,
) -> PriceResolution {
    resolution(
        resource,
        PriceResolutionStatus::Unrated,
        billability,
        resolved_price,
        rate_identity,
        None,
        Some(PriceResolutionFailure {
            code,
            message: message.into(),
        }),
    )
}

fn resolution(
    resource: ResourceDefinition,
    status: PriceResolutionStatus,
    billability: ResourceBillability,
    resolved_price: Option<ResolvedModelPrice>,
    rate_identity: Option<ResolvedRateIdentity>,
    billing: Option<BillingStructure>,
    failure: Option<PriceResolutionFailure>,
) -> PriceResolution {
    let audit_snapshot = PricingAuditSnapshot {
        resource,
        status,
        billability,
        rate_identity: rate_identity.clone(),
        strategy: billing.as_ref().map(|billing| billing.strategy),
        failure: failure.clone(),
    };
    PriceResolution {
        status,
        billability,
        resolved_price,
        rate_identity,
        billing,
        failure,
        audit_snapshot,
    }
}
