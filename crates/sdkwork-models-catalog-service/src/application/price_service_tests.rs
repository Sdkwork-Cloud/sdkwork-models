use chrono::{TimeZone, Utc};
use serde_json::json;

use super::{
    BillingStrategyKind, PriceResolutionFailureCode, PriceResolutionStatus, PriceService,
    ResourceBillability,
};
use crate::domain::{
    AiModel, BillingMeter, DecimalValue, GatewayAccessPolicy, GatewayApiKey, GatewayRiskRule,
    ModelMappingRule, ModelPrice, ModelUpstreamRoute, ModelVendor, ModelVendorDefinition, Money,
    PriceSide, PricingDimensionContext, PricingFormula, PricingFormulaTerm, PricingPlan,
    PricingRateCondition, PricingRateMetadata, PricingRateTier, QuotaPolicy,
    ResolveModelMappingContext, ResourceDefinition, RoutingPolicy, RoutingRule,
    UpstreamAccountGroup, UpstreamAccountGroupMetricSnapshot, UpstreamAccountRoute,
};
use crate::ports::PricingCatalog;

const CATALOG_KEY: &str = "openai/test-model";
const MODEL_ID: &str = "test-model";
const API_CODE: &str = "openai.responses";
const PRODUCT_CODE: &str = "openai-model-api";
const OPERATION_CODE: &str = "responses.create";
const GROUP_ID: i64 = 1;

#[derive(Default)]
struct TestPricingCatalog {
    vendors: Vec<ModelVendorDefinition>,
    models: Vec<AiModel>,
    groups: Vec<UpstreamAccountGroup>,
    plans: Vec<PricingPlan>,
    prices: Vec<ModelPrice>,
}

impl TestPricingCatalog {
    fn with_prices(prices: Vec<ModelPrice>) -> Self {
        Self {
            vendors: vec![ModelVendorDefinition::new(
                "openai",
                ModelVendor::OpenAi,
                "OpenAI",
            )],
            models: vec![AiModel::new(MODEL_ID, "Test model", "openai", vec!["chat"])
                .with_catalog_key(CATALOG_KEY)],
            groups: vec![UpstreamAccountGroup::new_scoped(
                GROUP_ID,
                10,
                20,
                "default",
                "standard",
                DecimalValue::ONE,
                DecimalValue::ONE,
            )],
            plans: vec![PricingPlan::new(
                "standard",
                PriceSide::OfficialReference,
                DecimalValue::ONE,
                Money::usd("0").expect("valid zero price"),
            )],
            prices,
        }
    }
}

impl PricingCatalog for TestPricingCatalog {
    fn visit_models(&self, vendor_code: Option<&str>, visitor: &mut dyn FnMut(&AiModel) -> bool) {
        for model in self.models.iter().filter(|model| {
            vendor_code
                .map(|vendor_code| model.vendor_code == vendor_code)
                .unwrap_or(true)
        }) {
            if !visitor(model) {
                break;
            }
        }
    }

    fn list_model_upstream_routes(&self, _model: &str) -> Vec<ModelUpstreamRoute> {
        Vec::new()
    }

    fn list_upstream_account_routes(&self) -> Vec<UpstreamAccountRoute> {
        Vec::new()
    }

    fn list_routing_policies(&self) -> Vec<RoutingPolicy> {
        Vec::new()
    }

    fn list_routing_rules(&self, _profile_id: i64) -> Vec<RoutingRule> {
        Vec::new()
    }

    fn list_model_mappings(&self) -> Vec<ModelMappingRule> {
        Vec::new()
    }

    fn list_api_keys(&self) -> Vec<GatewayApiKey> {
        Vec::new()
    }

    fn list_upstream_account_groups(&self) -> Vec<UpstreamAccountGroup> {
        self.groups.clone()
    }

    fn list_model_prices(
        &self,
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
    ) -> Vec<ModelPrice> {
        self.prices
            .iter()
            .filter(|price| {
                price.catalog_key == model
                    && price.price_side == price_side
                    && price.billing_meter == billing_meter
            })
            .cloned()
            .collect()
    }

    fn list_model_prices_for_side(&self, model: &str, price_side: PriceSide) -> Vec<ModelPrice> {
        self.prices
            .iter()
            .filter(|price| price.catalog_key == model && price.price_side == price_side)
            .cloned()
            .collect()
    }

    fn find_api_key(&self, _api_key_id: i64) -> Option<GatewayApiKey> {
        None
    }

    fn find_api_key_by_hash(&self, _key_hash: &str) -> Option<GatewayApiKey> {
        None
    }

    fn find_upstream_account_group(&self, account_group_id: i64) -> Option<UpstreamAccountGroup> {
        self.groups
            .iter()
            .find(|group| group.id == account_group_id)
            .cloned()
    }

    fn find_access_policy(&self, _policy_id: i64) -> Option<GatewayAccessPolicy> {
        None
    }

    fn find_quota_policy(&self, _policy_id: i64) -> Option<QuotaPolicy> {
        None
    }

    fn list_gateway_risk_rules(&self) -> Vec<GatewayRiskRule> {
        Vec::new()
    }

    fn find_latest_upstream_account_group_metric_snapshot(
        &self,
        _account_group_id: i64,
    ) -> Option<UpstreamAccountGroupMetricSnapshot> {
        None
    }

    fn find_pricing_plan(&self, plan_code: &str) -> Option<PricingPlan> {
        self.plans
            .iter()
            .find(|plan| plan.plan_code == plan_code)
            .cloned()
    }

    fn find_model(&self, model: &str) -> Option<AiModel> {
        self.models
            .iter()
            .find(|candidate| candidate.catalog_key == model)
            .cloned()
    }

    fn find_vendor(&self, vendor_code: &str) -> Option<ModelVendorDefinition> {
        self.vendors
            .iter()
            .find(|vendor| vendor.vendor_code == vendor_code)
            .cloned()
    }

    fn resolve_model_mapping(
        &self,
        _source_model: &str,
        _context: &ResolveModelMappingContext,
    ) -> Option<ModelMappingRule> {
        None
    }

    fn find_model_upstream_route(
        &self,
        _model: &str,
        _supplier_code: &str,
    ) -> Option<ModelUpstreamRoute> {
        None
    }

    fn find_model_price(
        &self,
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
        supplier_code: Option<&str>,
        pricing_plan_code: Option<&str>,
    ) -> Option<ModelPrice> {
        self.prices
            .iter()
            .find(|price| {
                price.catalog_key == model
                    && price.price_side == price_side
                    && price.billing_meter == billing_meter
                    && price.supplier_code.as_deref() == supplier_code
                    && price.pricing_plan_code.as_deref() == pricing_plan_code
            })
            .cloned()
    }
}

fn decimal(value: &str) -> DecimalValue {
    DecimalValue::parse(value).expect("valid decimal")
}

fn metadata(
    rate_hash: &str,
    billability: &str,
    calculation_mode: &str,
    minimum_quantity: &str,
    quantity_step: Option<&str>,
    priority: i32,
    conditions: Vec<PricingRateCondition>,
) -> PricingRateMetadata {
    PricingRateMetadata {
        price_book_code: "openai-cn-usd".to_owned(),
        rate_hash: rate_hash.to_owned(),
        product_code: PRODUCT_CODE.to_owned(),
        operation_code: OPERATION_CODE.to_owned(),
        billability: billability.to_owned(),
        charge_timing: "usage_reported".to_owned(),
        calculation_mode: calculation_mode.to_owned(),
        quantity_aggregation: "sum".to_owned(),
        minimum_quantity: decimal(minimum_quantity),
        quantity_step: quantity_step.map(decimal),
        priority,
        conditions,
        tiers: Vec::new(),
        formula: None,
    }
}

fn official_price(
    meter: BillingMeter,
    unit_size: &str,
    unit_price: &str,
    metadata: PricingRateMetadata,
) -> ModelPrice {
    ModelPrice::new_for_catalog_key(
        CATALOG_KEY,
        MODEL_ID,
        PriceSide::OfficialReference,
        meter,
        Money::usd(unit_price).expect("valid price"),
    )
    .with_region_code("cn")
    .with_unit_size(decimal(unit_size))
    .with_rate_metadata(metadata)
}

fn resource(meter: BillingMeter) -> ResourceDefinition {
    ResourceDefinition::new(
        CATALOG_KEY,
        meter,
        Utc.with_ymd_and_hms(2026, 8, 15, 0, 0, 0)
            .single()
            .expect("valid timestamp"),
    )
    .with_pricing_subject(0, Some(GROUP_ID))
    .with_vendor_code("openai")
    .with_region_code("cn")
    .with_model(MODEL_ID)
    .with_api_code(API_CODE)
    .with_product_operation(PRODUCT_CODE, OPERATION_CODE)
}

#[test]
fn resolves_condition_specific_rate_by_vendor_region_api_and_model() {
    let generic = official_price(
        BillingMeter::LlmInputToken,
        "1000",
        "0.001",
        metadata("generic", "chargeable", "per_unit", "0", None, 100, vec![]),
    );
    let conditional = official_price(
        BillingMeter::LlmInputToken,
        "1000",
        "0.002",
        metadata(
            "responses-cn",
            "chargeable",
            "per_unit",
            "0",
            None,
            10,
            vec![PricingRateCondition {
                dimension_code: "api_code".to_owned(),
                operator_code: "eq".to_owned(),
                value: json!(API_CODE),
            }],
        ),
    );
    let catalog = TestPricingCatalog::with_prices(vec![generic, conditional]);

    let resolution = PriceService::new()
        .resolve(&catalog, resource(BillingMeter::LlmInputToken))
        .expect("price resolution succeeds");

    assert_eq!(PriceResolutionStatus::Quoted, resolution.status);
    assert_eq!(ResourceBillability::Chargeable, resolution.billability);
    let identity = resolution.rate_identity.expect("resolved rate identity");
    assert_eq!(Some("responses-cn"), identity.rate_hash.as_deref());
    assert_eq!("openai", identity.vendor_code);
    assert_eq!("cn", identity.region_code);
    assert_eq!(CATALOG_KEY, identity.catalog_key);
}

#[test]
fn token_strategy_applies_minimum_step_unit_size_and_exact_decimal_amount() {
    let price = official_price(
        BillingMeter::LlmInputToken,
        "1000",
        "0.002",
        metadata(
            "token-stepped",
            "chargeable",
            "per_unit",
            "1000",
            Some("500"),
            10,
            vec![],
        ),
    );
    let catalog = TestPricingCatalog::with_prices(vec![price]);
    let resource = resource(BillingMeter::LlmInputToken).with_measured_quantity(decimal("1001"));

    let resolution = PriceService::new()
        .resolve(&catalog, resource)
        .expect("rating succeeds");
    let billing = resolution.billing.expect("rated billing structure");

    assert_eq!(PriceResolutionStatus::Rated, resolution.status);
    assert_eq!(BillingStrategyKind::TokenUsage, billing.strategy);
    assert_eq!(decimal("1001"), billing.measured_quantity);
    assert_eq!(decimal("1500"), billing.rated_quantity);
    assert_eq!(decimal("1000"), billing.unit_size);
    assert_eq!(decimal("0.003"), billing.customer_charge_amount.unit_price);
}

#[test]
fn standard_registry_selects_independent_api_image_duration_unit_and_flat_strategies() {
    for (meter, quantity, calculation_mode, expected_strategy) in [
        (
            BillingMeter::ApiRequest,
            "2",
            "per_unit",
            BillingStrategyKind::ApiCall,
        ),
        (
            BillingMeter::ImageResult,
            "3",
            "per_unit",
            BillingStrategyKind::ImageQuantity,
        ),
        (
            BillingMeter::VideoOutputSecond,
            "2.5",
            "per_unit",
            BillingStrategyKind::Duration,
        ),
        (
            BillingMeter::TtsInputCharacter,
            "250",
            "per_unit",
            BillingStrategyKind::UnitQuantity,
        ),
        (
            BillingMeter::ApiItem,
            "7",
            "flat",
            BillingStrategyKind::FlatFee,
        ),
    ] {
        let price = official_price(
            meter.clone(),
            "1",
            "0.01",
            metadata(
                &format!("{}-{calculation_mode}", meter.code()),
                "chargeable",
                calculation_mode,
                "0",
                None,
                10,
                vec![],
            ),
        );
        let catalog = TestPricingCatalog::with_prices(vec![price]);
        let resource = resource(meter).with_measured_quantity(decimal(quantity));

        let resolution = PriceService::new()
            .resolve(&catalog, resource)
            .expect("rating succeeds");

        assert_eq!(PriceResolutionStatus::Rated, resolution.status);
        assert_eq!(
            expected_strategy,
            resolution.billing.expect("billing structure").strategy
        );
    }
}

#[test]
fn flat_strategy_rejects_a_non_unit_catalog_base() {
    let price = official_price(
        BillingMeter::ApiRequest,
        "2",
        "0.01",
        metadata(
            "invalid-flat-unit-size",
            "chargeable",
            "flat",
            "0",
            None,
            10,
            vec![],
        ),
    );
    let catalog = TestPricingCatalog::with_prices(vec![price]);

    let resolution = PriceService::new()
        .resolve(
            &catalog,
            resource(BillingMeter::ApiRequest).with_measured_quantity(DecimalValue::ONE),
        )
        .expect("invalid flat fee resolves to an unrated decision");

    assert_eq!(PriceResolutionStatus::Unrated, resolution.status);
    assert!(resolution.billing.is_none());
    assert_eq!(
        Some(PriceResolutionFailureCode::UnsupportedBillingStrategy),
        resolution.failure.as_ref().map(|failure| failure.code)
    );
    assert!(resolution
        .failure
        .expect("classified failure")
        .message
        .contains("flat fee pricing unit size must equal one"));
}

#[test]
fn explicit_free_and_not_applicable_rates_are_non_chargeable() {
    for (billability, expected) in [
        ("free", ResourceBillability::Free),
        ("not_applicable", ResourceBillability::NotApplicable),
    ] {
        let price = official_price(
            BillingMeter::ApiRequest,
            "1",
            "0",
            metadata(billability, billability, "per_unit", "0", None, 10, vec![]),
        );
        let catalog = TestPricingCatalog::with_prices(vec![price]);

        let resolution = PriceService::new()
            .resolve(
                &catalog,
                resource(BillingMeter::ApiRequest).with_measured_quantity(DecimalValue::ONE),
            )
            .expect("resolution succeeds");

        assert_eq!(PriceResolutionStatus::NonChargeable, resolution.status);
        assert_eq!(expected, resolution.billability);
        assert!(resolution.billing.is_none());
    }
}

#[test]
fn unknown_billability_fails_closed_without_a_billing_structure() {
    let price = official_price(
        BillingMeter::ApiRequest,
        "1",
        "0.01",
        metadata(
            "unknown-billability",
            "vendor_specific",
            "per_unit",
            "0",
            None,
            10,
            vec![],
        ),
    );
    let catalog = TestPricingCatalog::with_prices(vec![price]);

    let resolution = PriceService::new()
        .resolve(
            &catalog,
            resource(BillingMeter::ApiRequest).with_measured_quantity(DecimalValue::ONE),
        )
        .expect("resolution returns a classified failure");

    assert_eq!(PriceResolutionStatus::Unrated, resolution.status);
    assert_eq!(ResourceBillability::Unknown, resolution.billability);
    assert_eq!(
        Some(PriceResolutionFailureCode::UnknownBillability),
        resolution.failure.map(|failure| failure.code)
    );
    assert!(resolution.billing.is_none());
}

#[test]
fn equally_specific_distinct_rates_fail_closed_as_ambiguous() {
    let first = official_price(
        BillingMeter::ApiRequest,
        "1",
        "0.01",
        metadata("rate-a", "chargeable", "per_unit", "0", None, 10, vec![]),
    );
    let second = official_price(
        BillingMeter::ApiRequest,
        "1",
        "0.02",
        metadata("rate-b", "chargeable", "per_unit", "0", None, 10, vec![]),
    );
    let catalog = TestPricingCatalog::with_prices(vec![first, second]);

    let resolution = PriceService::new()
        .resolve(&catalog, resource(BillingMeter::ApiRequest))
        .expect("ambiguity is classified instead of charged");

    assert_eq!(PriceResolutionStatus::Unrated, resolution.status);
    assert_eq!(
        Some(PriceResolutionFailureCode::AmbiguousRate),
        resolution.failure.map(|failure| failure.code)
    );
    assert!(resolution.billing.is_none());
}

#[test]
fn unsupported_calculation_mode_fails_closed() {
    let price = official_price(
        BillingMeter::ApiRequest,
        "1",
        "0.01",
        metadata("script-rate", "chargeable", "script", "0", None, 10, vec![]),
    );
    let catalog = TestPricingCatalog::with_prices(vec![price]);

    let resolution = PriceService::new()
        .resolve(
            &catalog,
            resource(BillingMeter::ApiRequest).with_measured_quantity(DecimalValue::ONE),
        )
        .expect("unsupported strategy is classified");

    assert_eq!(PriceResolutionStatus::Unrated, resolution.status);
    assert_eq!(ResourceBillability::Chargeable, resolution.billability);
    assert_eq!(
        Some(PriceResolutionFailureCode::UnsupportedBillingStrategy),
        resolution.failure.map(|failure| failure.code)
    );
    assert!(resolution.billing.is_none());
}

#[test]
fn graduated_tier_strategy_rates_each_contiguous_band_exactly() {
    let mut rate_metadata = metadata(
        "graduated-rate",
        "chargeable",
        "graduated",
        "0",
        None,
        10,
        vec![],
    );
    rate_metadata.tiers = vec![
        PricingRateTier {
            tier_code: "first-10".to_owned(),
            lower_bound: decimal("0"),
            upper_bound: Some(decimal("10")),
            unit_size: decimal("1"),
            unit_price: Money::usd("1").expect("valid tier price"),
            flat_amount: Money::usd("0").expect("valid flat amount"),
        },
        PricingRateTier {
            tier_code: "over-10".to_owned(),
            lower_bound: decimal("10"),
            upper_bound: None,
            unit_size: decimal("1"),
            unit_price: Money::usd("0.5").expect("valid tier price"),
            flat_amount: Money::usd("0").expect("valid flat amount"),
        },
    ];
    let catalog = TestPricingCatalog::with_prices(vec![official_price(
        BillingMeter::ApiItem,
        "1",
        "0",
        rate_metadata,
    )]);

    let resolution = PriceService::new()
        .resolve(
            &catalog,
            resource(BillingMeter::ApiItem).with_measured_quantity(decimal("15")),
        )
        .expect("graduated pricing succeeds");
    let billing = resolution.billing.expect("rated billing structure");

    assert_eq!(BillingStrategyKind::GraduatedTier, billing.strategy);
    assert_eq!(
        decimal("12.5"),
        billing.official_reference_amount.unit_price
    );
    assert_eq!(
        2,
        billing
            .components
            .iter()
            .filter(|item| item.price_side == PriceSide::OfficialReference)
            .count()
    );
}

#[test]
fn volume_tier_strategy_applies_the_selected_tier_to_the_whole_quantity() {
    let mut rate_metadata = metadata("volume-rate", "chargeable", "volume", "0", None, 10, vec![]);
    rate_metadata.tiers = vec![
        PricingRateTier {
            tier_code: "small".to_owned(),
            lower_bound: decimal("0"),
            upper_bound: Some(decimal("10")),
            unit_size: decimal("1"),
            unit_price: Money::usd("1").expect("valid tier price"),
            flat_amount: Money::usd("0").expect("valid flat amount"),
        },
        PricingRateTier {
            tier_code: "large".to_owned(),
            lower_bound: decimal("10"),
            upper_bound: None,
            unit_size: decimal("1"),
            unit_price: Money::usd("0.5").expect("valid tier price"),
            flat_amount: Money::usd("0").expect("valid flat amount"),
        },
    ];
    let catalog = TestPricingCatalog::with_prices(vec![official_price(
        BillingMeter::ApiItem,
        "1",
        "0",
        rate_metadata,
    )]);

    let resolution = PriceService::new()
        .resolve(
            &catalog,
            resource(BillingMeter::ApiItem).with_measured_quantity(decimal("15")),
        )
        .expect("volume pricing succeeds");
    let billing = resolution.billing.expect("rated billing structure");

    assert_eq!(BillingStrategyKind::VolumeTier, billing.strategy);
    assert_eq!(decimal("7.5"), billing.official_reference_amount.unit_price);
}

#[test]
fn formula_strategy_uses_only_bounded_typed_numeric_terms() {
    let mut rate_metadata = metadata(
        "formula-rate",
        "chargeable",
        "formula",
        "0",
        None,
        10,
        vec![],
    );
    rate_metadata.formula = Some(PricingFormula {
        formula_code: "duration-weighted".to_owned(),
        formula_version: "1".to_owned(),
        constant_units: decimal("2"),
        quantity_coefficient: decimal("1.5"),
        minimum_units: None,
        maximum_units: Some(decimal("10")),
        terms: vec![PricingFormulaTerm {
            term_code: "duration".to_owned(),
            dimension_code: "duration_seconds".to_owned(),
            coefficient: decimal("0.25"),
        }],
    });
    let catalog = TestPricingCatalog::with_prices(vec![official_price(
        BillingMeter::ApiItem,
        "1",
        "0.2",
        rate_metadata,
    )]);
    let dimensions = PricingDimensionContext::new().with_value("duration_seconds", json!("8"));

    let resolution = PriceService::new()
        .resolve(
            &catalog,
            resource(BillingMeter::ApiItem)
                .with_dimensions(dimensions)
                .with_measured_quantity(decimal("4")),
        )
        .expect("formula pricing succeeds");
    let billing = resolution.billing.expect("rated billing structure");

    assert_eq!(BillingStrategyKind::Formula, billing.strategy);
    assert_eq!(decimal("10"), billing.rated_quantity);
    assert_eq!(decimal("2"), billing.official_reference_amount.unit_price);
}

#[test]
fn resolved_rate_rejects_resource_identity_mismatches() {
    let price = official_price(
        BillingMeter::ApiRequest,
        "1",
        "0.01",
        metadata(
            "matched-rate",
            "chargeable",
            "per_unit",
            "0",
            None,
            10,
            vec![],
        ),
    );
    let catalog = TestPricingCatalog::with_prices(vec![price]);
    let quoted = PriceService::new()
        .resolve(&catalog, resource(BillingMeter::ApiRequest))
        .expect("quote succeeds");
    let resolved_price = quoted.resolved_price.expect("resolved price");

    let mut mismatches = Vec::new();
    let mut vendor = resource(BillingMeter::ApiRequest);
    vendor.vendor_code = Some("google".to_owned());
    mismatches.push(vendor);
    let mut product = resource(BillingMeter::ApiRequest);
    product.product_code = Some("other-product".to_owned());
    mismatches.push(product);
    let mut operation = resource(BillingMeter::ApiRequest);
    operation.operation_code = Some("other.operation".to_owned());
    mismatches.push(operation);
    let mut catalog_key = resource(BillingMeter::ApiRequest);
    catalog_key.catalog_key = "openai/other-model".to_owned();
    mismatches.push(catalog_key);
    let meter = resource(BillingMeter::LlmInputToken);
    mismatches.push(meter);
    let mut provider = resource(BillingMeter::ApiRequest);
    provider.provider_code = Some("other-provider".to_owned());
    mismatches.push(provider);
    let mut region = resource(BillingMeter::ApiRequest);
    region.region_code = Some("global".to_owned());
    mismatches.push(region);

    for mismatch in mismatches {
        let resolution = PriceService::new()
            .rate_resolved(mismatch, resolved_price.clone())
            .expect("mismatch is classified");
        assert_eq!(PriceResolutionStatus::Unrated, resolution.status);
        assert_eq!(
            Some(PriceResolutionFailureCode::ResourceMismatch),
            resolution.failure.map(|failure| failure.code)
        );
        assert!(resolution.billing.is_none());
    }
}
