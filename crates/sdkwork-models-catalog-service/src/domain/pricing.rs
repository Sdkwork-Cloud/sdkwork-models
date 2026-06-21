use crate::domain::{BillingMeter, DecimalValue, Money};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriceSide {
    OfficialReference,
    UpstreamCost,
    CustomerCharge,
    InternalTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingPlan {
    pub plan_code: String,
    pub base_price_side: PriceSide,
    pub default_multiplier: DecimalValue,
    pub default_markup_amount: Money,
}

impl PricingPlan {
    pub fn new(
        plan_code: &str,
        base_price_side: PriceSide,
        default_multiplier: DecimalValue,
        default_markup_amount: Money,
    ) -> Self {
        Self {
            plan_code: plan_code.to_owned(),
            base_price_side,
            default_multiplier,
            default_markup_amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPrice {
    pub catalog_key: String,
    pub model: String,
    pub region_code: String,
    pub price_side: PriceSide,
    pub billing_meter: BillingMeter,
    pub unit_price: Money,
    pub provider_code: Option<String>,
    pub channel_id: Option<i64>,
    pub pricing_plan_code: Option<String>,
}

impl ModelPrice {
    pub fn new(
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
        unit_price: Money,
    ) -> Self {
        Self {
            catalog_key: model.to_owned(),
            model: model.to_owned(),
            region_code: "global".to_owned(),
            price_side,
            billing_meter,
            unit_price,
            provider_code: None,
            channel_id: None,
            pricing_plan_code: None,
        }
    }

    pub fn new_for_catalog_key(
        catalog_key: &str,
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
        unit_price: Money,
    ) -> Self {
        Self {
            catalog_key: catalog_key.to_owned(),
            model: model.to_owned(),
            region_code: "global".to_owned(),
            price_side,
            billing_meter,
            unit_price,
            provider_code: None,
            channel_id: None,
            pricing_plan_code: None,
        }
    }

    pub fn with_catalog_key(mut self, catalog_key: &str) -> Self {
        self.catalog_key = catalog_key.to_owned();
        self
    }

    pub fn with_region_code(mut self, region_code: &str) -> Self {
        self.region_code = normalized_region_code(region_code);
        self
    }

    pub fn for_provider(mut self, provider_code: &str, channel_id: i64) -> Self {
        self.provider_code = Some(provider_code.to_owned());
        self.channel_id = Some(channel_id);
        self
    }

    pub fn for_pricing_plan(mut self, pricing_plan_code: &str) -> Self {
        self.pricing_plan_code = Some(pricing_plan_code.to_owned());
        self
    }
}

fn normalized_region_code(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "global".to_owned()
    } else {
        value.to_owned()
    }
}
