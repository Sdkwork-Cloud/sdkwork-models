use chrono::{DateTime, Utc};

use crate::domain::{BillingMeter, DecimalValue, PricingDimensionContext};

/// Canonical description of one resource presented to the pricing service.
///
/// The catalog key identifies the priced resource while the remaining fields
/// provide the vendor, provider, region, API, operation, and dynamic dimension
/// context needed to select one unambiguous rate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDefinition {
    pub api_key_id: i64,
    pub account_group_id: Option<i64>,
    pub vendor_code: Option<String>,
    pub provider_code: Option<String>,
    pub account_id: Option<i64>,
    pub region_code: Option<String>,
    /// Configured default billing region for the resource (admin "default
    /// region" setting). The resolver probes it as the first fallback when the
    /// requested region has no price, before the generic `global` bucket.
    pub default_billing_region_code: Option<String>,
    pub catalog_key: String,
    pub model: Option<String>,
    pub api_code: Option<String>,
    pub product_code: Option<String>,
    pub operation_code: Option<String>,
    pub meter: BillingMeter,
    pub measured_quantity: Option<DecimalValue>,
    pub dimensions: PricingDimensionContext,
    pub occurred_at: DateTime<Utc>,
}

impl ResourceDefinition {
    pub fn new(
        catalog_key: impl Into<String>,
        meter: BillingMeter,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            api_key_id: 0,
            account_group_id: None,
            vendor_code: None,
            provider_code: None,
            account_id: None,
            region_code: None,
            default_billing_region_code: None,
            catalog_key: catalog_key.into(),
            model: None,
            api_code: None,
            product_code: None,
            operation_code: None,
            meter,
            measured_quantity: None,
            dimensions: PricingDimensionContext::new(),
            occurred_at,
        }
    }

    pub fn with_pricing_subject(mut self, api_key_id: i64, account_group_id: Option<i64>) -> Self {
        self.api_key_id = api_key_id;
        self.account_group_id = account_group_id;
        self
    }

    pub fn with_vendor_code(mut self, vendor_code: impl Into<String>) -> Self {
        self.vendor_code = normalized_optional(vendor_code.into());
        self
    }

    pub fn with_provider(
        mut self,
        provider_code: impl Into<String>,
        account_id: Option<i64>,
    ) -> Self {
        self.provider_code = normalized_optional(provider_code.into());
        self.account_id = account_id;
        self
    }

    pub fn with_region_code(mut self, region_code: impl Into<String>) -> Self {
        self.region_code = normalized_optional(region_code.into());
        self
    }

    /// Attaches the configured default billing region for the resource. The
    /// resolver falls back to it when the requested region carries no price,
    /// before the generic `global` bucket; `None`/blank keeps the legacy
    /// `requested -> global -> any` behavior.
    pub fn with_default_billing_region(mut self, region_code: Option<String>) -> Self {
        self.default_billing_region_code = normalized_optional(region_code.unwrap_or_default());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = normalized_optional(model.into());
        self
    }

    pub fn with_api_code(mut self, api_code: impl Into<String>) -> Self {
        self.api_code = normalized_optional(api_code.into());
        self
    }

    pub fn with_product_operation(
        mut self,
        product_code: impl Into<String>,
        operation_code: impl Into<String>,
    ) -> Self {
        self.product_code = normalized_optional(product_code.into());
        self.operation_code = normalized_optional(operation_code.into());
        self
    }

    pub fn with_measured_quantity(mut self, measured_quantity: DecimalValue) -> Self {
        self.measured_quantity = Some(measured_quantity);
        self
    }

    pub fn with_dimensions(mut self, dimensions: PricingDimensionContext) -> Self {
        self.dimensions = dimensions;
        self
    }
}

fn normalized_optional(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
