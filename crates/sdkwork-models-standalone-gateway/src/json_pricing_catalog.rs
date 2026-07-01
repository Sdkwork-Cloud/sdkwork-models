use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use sdkwork_models::{load_catalog, ModelCatalog, ModelInfo, ModelPricing, VendorCatalog};
use sdkwork_models_catalog_repository_sqlx::model_catalog_import::public_catalog_identity_models;
use sdkwork_models_catalog_service::domain::{
    AiModel, AiModelPublicMetadata, BillingMeter, ChannelGroup, ChannelGroupMetricSnapshot,
    GatewayAccessPolicy, GatewayApiKey, GatewayRiskRule, ModelMappingRule, ModelPrice,
    ModelProviderRoute, ModelVendor, ModelVendorDefinition, Money, PriceSide, PricingPlan,
    ProviderChannelRoute, QuotaPolicy, ResolveModelMappingContext, RoutingPolicy, RoutingRule,
};
use sdkwork_models_catalog_service::ports::PricingCatalog;

#[derive(Debug, Clone)]
pub struct JsonPricingCatalog {
    vendors: Vec<ModelVendorDefinition>,
    models: Vec<AiModel>,
    prices: Vec<ModelPrice>,
}

impl JsonPricingCatalog {
    pub fn from_catalog(catalog: &ModelCatalog) -> Self {
        let mut vendors = BTreeMap::new();
        for vendor in &catalog.vendors {
            vendors.insert(
                vendor.vendor_code.clone(),
                map_vendor_definition(vendor),
            );
        }

        let mut models = Vec::new();
        for (_, (vendor, model)) in public_catalog_identity_models(catalog) {
            vendors
                .entry(vendor.vendor_code.clone())
                .or_insert_with(|| map_vendor_definition(vendor));
            models.push(map_model(vendor, model));
        }

        let mut prices = Vec::new();
        for vendor in &catalog.vendors {
            for pricing in &vendor.pricing {
                prices.extend(map_vendor_pricing(vendor, pricing));
            }
        }

        Self {
            vendors: vendors.into_values().collect(),
            models,
            prices,
        }
    }

    pub fn load_from_root(root: impl AsRef<Path>) -> Result<Arc<Self>, String> {
        let catalog = load_catalog(root).map_err(|error| format!("load catalog JSON failed: {error}"))?;
        Ok(Arc::new(Self::from_catalog(&catalog)))
    }
}

impl PricingCatalog for JsonPricingCatalog {
    fn list_models(&self, vendor_code: Option<&str>) -> Vec<AiModel> {
        self.models
            .iter()
            .filter(|model| {
                model.is_publicly_active()
                    && vendor_code
                        .map(|code| model.vendor_code == code)
                        .unwrap_or(true)
            })
            .cloned()
            .collect()
    }

    fn list_provider_routes(&self, _model: &str) -> Vec<ModelProviderRoute> {
        Vec::new()
    }

    fn list_provider_channel_routes(&self) -> Vec<ProviderChannelRoute> {
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

    fn list_channel_groups(&self) -> Vec<ChannelGroup> {
        Vec::new()
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
                price.model == model
                    && price.price_side == price_side
                    && price.billing_meter == billing_meter
            })
            .cloned()
            .collect()
    }

    fn list_model_prices_for_side(&self, model: &str, price_side: PriceSide) -> Vec<ModelPrice> {
        self.prices
            .iter()
            .filter(|price| price.model == model && price.price_side == price_side)
            .cloned()
            .collect()
    }

    fn find_api_key(&self, _api_key_id: i64) -> Option<GatewayApiKey> {
        None
    }

    fn find_api_key_by_hash(&self, _key_hash: &str) -> Option<GatewayApiKey> {
        None
    }

    fn find_channel_group(&self, _group_id: i64) -> Option<ChannelGroup> {
        None
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

    fn find_latest_channel_group_metric_snapshot(
        &self,
        _group_id: i64,
    ) -> Option<ChannelGroupMetricSnapshot> {
        None
    }

    fn find_pricing_plan(&self, _plan_code: &str) -> Option<PricingPlan> {
        None
    }

    fn find_model(&self, model: &str) -> Option<AiModel> {
        self.models
            .iter()
            .find(|entry| entry.catalog_key == model || entry.model == model)
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

    fn find_provider_route(&self, _model: &str, _provider_code: &str) -> Option<ModelProviderRoute> {
        None
    }

    fn find_model_price(
        &self,
        model: &str,
        price_side: PriceSide,
        billing_meter: BillingMeter,
        provider_code: Option<&str>,
        pricing_plan_code: Option<&str>,
    ) -> Option<ModelPrice> {
        self.prices.iter().find(|price| {
            price.model == model
                && price.price_side == price_side
                && price.billing_meter == billing_meter
                && price.provider_code.as_deref() == provider_code
                && price.pricing_plan_code.as_deref() == pricing_plan_code
        }).cloned()
    }
}

fn map_vendor_definition(vendor: &VendorCatalog) -> ModelVendorDefinition {
    ModelVendorDefinition::new(
        &vendor.vendor_code,
        ModelVendor::from_code(&vendor.vendor_code),
        &vendor.vendor.display_name,
    )
}

fn map_model(vendor: &VendorCatalog, model: &ModelInfo) -> AiModel {
    AiModel::new(
        &model.model_id,
        &model.display_name,
        &vendor.vendor_code,
        model.capabilities.iter().map(String::as_str).collect(),
    )
    .with_catalog_key(&model.catalog_key)
    .with_public_metadata(AiModelPublicMetadata {
        description: model.description.clone(),
        modalities: model
            .capabilities
            .iter()
            .chain(model.input_modalities.iter())
            .chain(model.output_modalities.iter())
            .cloned()
            .collect(),
        input_modalities: model.input_modalities.clone(),
        output_modalities: model.output_modalities.clone(),
        api_format: Some(model.api_format.clone()),
        capability_intro: None,
        limitations: model.strengths.clone(),
        supported_languages: Vec::new(),
        use_cases: Vec::new(),
        training_data_cutoff: None,
        context_tokens: model.context_tokens,
        max_output_tokens: model.max_output_tokens,
        supports_streaming: model.supports_streaming,
        supports_tools: model.supports_tools,
        supports_json_schema: model.supports_json_schema,
        release_stage: Some(release_stage_code(&model.release_stage)),
        shelf_state: Some(shelf_state_code(&model.shelf_state)),
        routing_state: Some(routing_state_code(&model.routing_state)),
        replacement_model: model.replacement_model.clone(),
    })
}

fn map_vendor_pricing(_vendor: &VendorCatalog, pricing: &ModelPricing) -> Vec<ModelPrice> {
    pricing
        .prices
        .iter()
        .filter_map(|price| {
            let price_side = map_price_side(&price.price_side)?;
            let billing_meter = BillingMeter::from_code(&price.meter_code);
            let currency = price
                .currency
                .clone()
                .unwrap_or_else(|| pricing.currency.clone());
            let unit_price = Money::new(&currency, &price.unit_price).ok()?;
            Some(ModelPrice {
                catalog_key: pricing.catalog_key.clone(),
                model: pricing.catalog_key.clone(),
                region_code: pricing.region_code.clone(),
                price_side,
                billing_meter,
                provider_code: None,
                channel_id: None,
                pricing_plan_code: None,
                unit_price,
            })
        })
        .collect()
}

fn map_price_side(value: &str) -> Option<PriceSide> {
    match value {
        "official" | "official_reference" => Some(PriceSide::OfficialReference),
        "upstream" | "upstream_cost" => Some(PriceSide::UpstreamCost),
        "customer" | "customer_charge" => Some(PriceSide::CustomerCharge),
        "internal" | "internal_transfer" => Some(PriceSide::InternalTransfer),
        _ => None,
    }
}

fn release_stage_code(value: &str) -> i32 {
    match value {
        "preview" => 2,
        "deprecated" => 3,
        "retired" => 4,
        _ => 1,
    }
}

fn shelf_state_code(value: &str) -> i32 {
    match value {
        "hidden" => 2,
        "archived" => 3,
        _ => 1,
    }
}

fn routing_state_code(value: &str) -> i32 {
    match value {
        "enabled" => 1,
        _ => 0,
    }
}
