use std::collections::BTreeMap;

use serde_json::Value;

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
    pub unit_size: DecimalValue,
    pub unit_price: Money,
    pub supplier_code: Option<String>,
    pub account_id: Option<i64>,
    pub pricing_plan_code: Option<String>,
    pub rate_metadata: Option<PricingRateMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingRateMetadata {
    pub price_book_code: String,
    pub rate_hash: String,
    pub product_code: String,
    pub operation_code: String,
    pub billability: String,
    pub charge_timing: String,
    pub calculation_mode: String,
    pub quantity_aggregation: String,
    pub minimum_quantity: DecimalValue,
    pub quantity_step: Option<DecimalValue>,
    pub priority: i32,
    pub conditions: Vec<PricingRateCondition>,
    pub tiers: Vec<PricingRateTier>,
    pub formula: Option<PricingFormula>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingRateTier {
    pub tier_code: String,
    pub lower_bound: DecimalValue,
    pub upper_bound: Option<DecimalValue>,
    pub unit_size: DecimalValue,
    pub unit_price: Money,
    pub flat_amount: Money,
}

/// A deliberately bounded formula contract. The formula first converts the
/// measured quantity and selected numeric dimensions into pricing units; each
/// resolved price side then multiplies those units by its unit price.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingFormula {
    pub formula_code: String,
    pub formula_version: String,
    pub constant_units: DecimalValue,
    pub quantity_coefficient: DecimalValue,
    pub minimum_units: Option<DecimalValue>,
    pub maximum_units: Option<DecimalValue>,
    pub terms: Vec<PricingFormulaTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingFormulaTerm {
    pub term_code: String,
    pub dimension_code: String,
    pub coefficient: DecimalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingRateCondition {
    pub dimension_code: String,
    pub operator_code: String,
    pub value: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PricingDimensionContext {
    values: BTreeMap<String, Value>,
}

impl PricingDimensionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, dimension_code: impl Into<String>, value: Value) {
        let dimension_code = dimension_code.into();
        let dimension_code = dimension_code.trim();
        if !dimension_code.is_empty() && !value.is_null() {
            self.values.insert(dimension_code.to_owned(), value);
        }
    }

    pub fn with_value(mut self, dimension_code: impl Into<String>, value: Value) -> Self {
        self.insert(dimension_code, value);
        self
    }

    pub fn get(&self, dimension_code: &str) -> Option<&Value> {
        self.values.get(dimension_code.trim())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.values
            .iter()
            .map(|(dimension_code, value)| (dimension_code.as_str(), value))
    }

    pub fn decimal(&self, dimension_code: &str) -> Option<DecimalValue> {
        self.get(dimension_code).and_then(decimal_value)
    }
}

impl PricingRateMetadata {
    pub fn matches(&self, dimensions: &PricingDimensionContext) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.matches(dimensions))
    }

    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }
}

impl PricingRateCondition {
    pub fn matches(&self, dimensions: &PricingDimensionContext) -> bool {
        let actual = dimensions.get(&self.dimension_code);
        match self.operator_code.as_str() {
            "exists" => self.value.as_bool().unwrap_or(true) == actual.is_some(),
            "eq" => actual.is_some_and(|actual| pricing_values_equal(actual, &self.value)),
            "neq" => actual.is_some_and(|actual| !pricing_values_equal(actual, &self.value)),
            "gt" => compare_decimal_values(actual, &self.value).is_some_and(|order| order.is_gt()),
            "gte" => compare_decimal_values(actual, &self.value)
                .is_some_and(|order| order.is_gt() || order.is_eq()),
            "lt" => compare_decimal_values(actual, &self.value).is_some_and(|order| order.is_lt()),
            "lte" => compare_decimal_values(actual, &self.value)
                .is_some_and(|order| order.is_lt() || order.is_eq()),
            "in" => actual.is_some_and(|actual| pricing_value_in(actual, &self.value)),
            "not_in" => actual.is_some_and(|actual| !pricing_value_in(actual, &self.value)),
            _ => false,
        }
    }
}

fn pricing_values_equal(actual: &Value, expected: &Value) -> bool {
    match (decimal_value(actual), decimal_value(expected)) {
        (Some(actual), Some(expected)) => actual == expected,
        _ => actual == expected,
    }
}

fn pricing_value_in(actual: &Value, expected: &Value) -> bool {
    match expected.as_array() {
        Some(values) => values
            .iter()
            .any(|candidate| pricing_values_equal(actual, candidate)),
        None => actual.as_array().is_some_and(|values| {
            values
                .iter()
                .any(|value| pricing_values_equal(value, expected))
        }),
    }
}

fn compare_decimal_values(actual: Option<&Value>, expected: &Value) -> Option<std::cmp::Ordering> {
    decimal_value(actual?)?.partial_cmp(&decimal_value(expected)?)
}

fn decimal_value(value: &Value) -> Option<DecimalValue> {
    let value = match value {
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.trim().to_owned(),
        _ => return None,
    };
    DecimalValue::parse(&value).ok()
}

impl ModelPrice {
    pub fn new(
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
        unit_price: Money,
    ) -> Self {
        let unit_size = default_unit_size(&billing_meter);
        Self {
            catalog_key: model.to_owned(),
            model: model.to_owned(),
            region_code: "global".to_owned(),
            price_side,
            billing_meter,
            unit_size,
            unit_price,
            supplier_code: None,
            account_id: None,
            pricing_plan_code: None,
            rate_metadata: None,
        }
    }

    pub fn new_for_catalog_key(
        catalog_key: &str,
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
        unit_price: Money,
    ) -> Self {
        let unit_size = default_unit_size(&billing_meter);
        Self {
            catalog_key: catalog_key.to_owned(),
            model: model.to_owned(),
            region_code: "global".to_owned(),
            price_side,
            billing_meter,
            unit_size,
            unit_price,
            supplier_code: None,
            account_id: None,
            pricing_plan_code: None,
            rate_metadata: None,
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

    pub fn with_unit_size(mut self, unit_size: DecimalValue) -> Self {
        self.unit_size = unit_size;
        self
    }

    pub fn for_upstream_account(mut self, supplier_code: &str, account_id: i64) -> Self {
        self.supplier_code = Some(supplier_code.to_owned());
        self.account_id = Some(account_id);
        self
    }

    pub fn for_pricing_plan(mut self, pricing_plan_code: &str) -> Self {
        self.pricing_plan_code = Some(pricing_plan_code.to_owned());
        self
    }

    pub fn with_rate_metadata(mut self, rate_metadata: PricingRateMetadata) -> Self {
        self.rate_metadata = Some(rate_metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{PricingDimensionContext, PricingRateCondition};

    #[test]
    fn pricing_conditions_match_string_numeric_membership_and_existence() {
        let dimensions = PricingDimensionContext::new()
            .with_value("quality", json!("hd"))
            .with_value("duration_seconds", json!(10))
            .with_value("resolution", json!("1080p"));

        for condition in [
            PricingRateCondition {
                dimension_code: "quality".to_owned(),
                operator_code: "eq".to_owned(),
                value: json!("hd"),
            },
            PricingRateCondition {
                dimension_code: "duration_seconds".to_owned(),
                operator_code: "gte".to_owned(),
                value: json!("10.000000"),
            },
            PricingRateCondition {
                dimension_code: "resolution".to_owned(),
                operator_code: "in".to_owned(),
                value: json!(["720p", "1080p"]),
            },
            PricingRateCondition {
                dimension_code: "quality".to_owned(),
                operator_code: "exists".to_owned(),
                value: json!(true),
            },
        ] {
            assert!(condition.matches(&dimensions));
        }
    }

    #[test]
    fn missing_dimensions_never_satisfy_negative_conditions() {
        let dimensions = PricingDimensionContext::new();
        for operator_code in ["neq", "not_in", "gt"] {
            let condition = PricingRateCondition {
                dimension_code: "quality".to_owned(),
                operator_code: operator_code.to_owned(),
                value: json!("hd"),
            };
            assert!(!condition.matches(&dimensions));
        }
    }
}

fn default_unit_size(meter: &BillingMeter) -> DecimalValue {
    if matches!(
        meter,
        BillingMeter::LlmInputToken
            | BillingMeter::LlmOutputToken
            | BillingMeter::LlmReasoningToken
            | BillingMeter::LlmCacheWriteToken
            | BillingMeter::LlmCacheReadToken
            | BillingMeter::EmbeddingInputToken
            | BillingMeter::ImageInputToken
            | BillingMeter::ImageOutputToken
            | BillingMeter::AudioInputToken
            | BillingMeter::AudioOutputToken
            | BillingMeter::VideoInputToken
            | BillingMeter::VideoOutputToken
    ) {
        DecimalValue::parse("1000000").expect("token billing unit size is valid")
    } else {
        DecimalValue::ONE
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
