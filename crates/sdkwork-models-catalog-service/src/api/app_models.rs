use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_utils_rust::SdkWorkResultCode;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};

use crate::api::page_info::{offset_page_info, ApiPageInfo};
use crate::api::response::{finish_success, problem_for};
use crate::application::{
    ListModelCatalogQuery, ModelCatalogGroup, ModelCatalogItem, ModelCatalogPage,
    ModelCatalogQueryService, PriceAvailability,
};
use crate::domain::BillingMeter;
use crate::ports::PricingCatalog;

struct AppModelCatalogState<C> {
    catalog: Arc<C>,
}

impl<C> Clone for AppModelCatalogState<C> {
    fn clone(&self) -> Self {
        Self {
            catalog: Arc::clone(&self.catalog),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppModelCatalogQuery {
    billing_meter: Option<String>,
    vendor_code: Option<String>,
    vendor_codes: Option<String>,
    modalities: Option<String>,
    capabilities: Option<String>,
    categories: Option<String>,
    groups: Option<String>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelCatalogResponse {
    items: Vec<AppModelCatalogItemResponse>,
    groups: Vec<AppModelCatalogGroupResponse>,
    page_info: ApiPageInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelVendorCatalogResponse {
    items: Vec<AppModelVendorOptionResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelVendorOptionResponse {
    label: String,
    code: String,
    model_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelCatalogGroupResponse {
    key: String,
    label: String,
    model_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelCatalogItemResponse {
    catalog_key: String,
    model: String,
    display_name: String,
    vendor_code: String,
    vendor: String,
    capabilities: Vec<String>,
    groups: Vec<String>,
    categories: Vec<String>,
    description: Option<String>,
    modalities: Vec<String>,
    input_modalities: Vec<String>,
    output_modalities: Vec<String>,
    api_format: Option<String>,
    capability_intro: Option<String>,
    limitations: Vec<String>,
    supported_languages: Vec<String>,
    use_cases: Vec<String>,
    training_data_cutoff: Option<String>,
    context_tokens: Option<i64>,
    max_output_tokens: Option<i64>,
    supports_streaming: bool,
    supports_tools: bool,
    supports_json_schema: bool,
    release_stage: Option<i32>,
    shelf_state: Option<i32>,
    routing_state: Option<i32>,
    replacement_model: Option<String>,
    supplier_codes: Vec<String>,
    official_reference_prices: Vec<AppModelCatalogReferencePriceResponse>,
    price_availability: AppModelCatalogPriceAvailabilityResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelCatalogReferencePriceResponse {
    region_code: String,
    billing_meter: String,
    unit_price: String,
    currency: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppModelCatalogPriceAvailabilityResponse {
    status: &'static str,
    reason: Option<String>,
}

const PUBLIC_REFERENCE_PRICE_REASON: &str =
    "Public reference price only. Customer-specific pricing requires an API key context.";
const PUBLIC_PRICE_UNAVAILABLE_REASON: &str =
    "Public reference price is not configured for this model.";

pub fn app_model_catalog_router<C>(catalog: Arc<C>) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/app/v3/api/ai/model_vendors",
            get(fetch_model_vendors::<C>),
        )
        .route("/app/v3/api/ai/models", get(fetch_models::<C>))
        .with_state(AppModelCatalogState { catalog })
}

async fn fetch_model_vendors<C>(
    ctx: WebRequestContext,
    State(state): State<AppModelCatalogState<C>>,
) -> Response
where
    C: PricingCatalog + Send + Sync + 'static,
{
    finish_success(&ctx, to_vendor_response(state.catalog.as_ref()))
}

async fn fetch_models<C>(
    ctx: WebRequestContext,
    State(state): State<AppModelCatalogState<C>>,
    Query(query): Query<AppModelCatalogQuery>,
) -> Response
where
    C: PricingCatalog + Send + Sync + 'static,
{
    let billing_meter = query
        .billing_meter
        .as_deref()
        .map(BillingMeter::from_code)
        .unwrap_or(BillingMeter::LlmInputToken);
    let (page, page_size) = match validate_page_query(query.page, query.page_size) {
        Ok(value) => value,
        Err(message) => return problem_for(&ctx, SdkWorkResultCode::ValidationError, message),
    };
    let offset = (page - 1) * page_size;
    let service = ModelCatalogQueryService::new(state.catalog.as_ref());

    match service.list_models(ListModelCatalogQuery {
        api_key_id: None,
        billing_meter,
        vendor_code: query.vendor_code,
        vendor_codes: comma_separated_query_values(query.vendor_codes.as_deref()),
        modalities: comma_separated_query_values(query.modalities.as_deref()),
        capabilities: comma_separated_query_values(query.capabilities.as_deref()),
        categories: comma_separated_query_values(query.categories.as_deref()),
        groups: comma_separated_query_values(query.groups.as_deref()),
        search_query: query.q,
        page_size: Some(page_size),
        offset: Some(offset),
    }) {
        Ok(page) => finish_success(&ctx, to_response(page)),
        Err(error) => problem_for(&ctx, SdkWorkResultCode::ValidationError, error.to_string()),
    }
}

fn to_response(page: ModelCatalogPage) -> AppModelCatalogResponse {
    AppModelCatalogResponse {
        items: page.items.into_iter().map(to_item_response).collect(),
        groups: page.groups.into_iter().map(to_group_response).collect(),
        page_info: offset_page_info(
            ((page.offset / page.page_size) + 1) as i64,
            page.page_size as i64,
            page.total_items as i64,
        ),
    }
}

fn validate_page_query(
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<(usize, usize), String> {
    let page = page.unwrap_or(1);
    if page == 0 {
        return Err("page must be greater than or equal to 1".to_owned());
    }
    let page_size = page_size.unwrap_or(20);
    if !(1..=200).contains(&page_size) {
        return Err("page_size must be between 1 and 200".to_owned());
    }
    Ok((page, page_size))
}

fn to_group_response(group: ModelCatalogGroup) -> AppModelCatalogGroupResponse {
    AppModelCatalogGroupResponse {
        key: group.key,
        label: group.label,
        model_count: group.model_count,
    }
}

fn to_vendor_response(
    catalog: &(impl PricingCatalog + Send + Sync),
) -> AppModelVendorCatalogResponse {
    let mut vendors_by_code: BTreeMap<String, AppModelVendorOptionResponse> = BTreeMap::new();
    for model in catalog.list_models(None) {
        let code = model.vendor_code.trim();
        if code.is_empty() {
            continue;
        }
        let entry = vendors_by_code.entry(code.to_owned()).or_insert_with(|| {
            AppModelVendorOptionResponse {
                label: catalog
                    .find_vendor(code)
                    .map(|vendor| vendor.display_name)
                    .unwrap_or_else(|| code.to_owned()),
                code: code.to_owned(),
                model_count: 0,
            }
        });
        entry.model_count += 1;
    }

    let mut items: Vec<_> = vendors_by_code.into_values().collect();
    items.sort_by(|first, second| {
        first
            .label
            .to_lowercase()
            .cmp(&second.label.to_lowercase())
            .then_with(|| first.code.cmp(&second.code))
    });
    AppModelVendorCatalogResponse { items }
}

fn to_item_response(item: ModelCatalogItem) -> AppModelCatalogItemResponse {
    let price_availability =
        to_price_availability_response(&item.official_reference_prices, item.price_availability);

    AppModelCatalogItemResponse {
        catalog_key: item.catalog_key,
        model: item.model,
        display_name: item.display_name,
        vendor_code: item.vendor_code,
        vendor: item.vendor.code().to_owned(),
        capabilities: item.capabilities,
        groups: item.groups,
        categories: item.categories,
        description: item.description,
        modalities: item.modalities,
        input_modalities: item.input_modalities,
        output_modalities: item.output_modalities,
        api_format: item.api_format,
        capability_intro: item.capability_intro,
        limitations: item.limitations,
        supported_languages: item.supported_languages,
        use_cases: item.use_cases,
        training_data_cutoff: item.training_data_cutoff,
        context_tokens: item.context_tokens,
        max_output_tokens: item.max_output_tokens,
        supports_streaming: item.supports_streaming,
        supports_tools: item.supports_tools,
        supports_json_schema: item.supports_json_schema,
        release_stage: item.release_stage,
        shelf_state: item.shelf_state,
        routing_state: item.routing_state,
        replacement_model: item.replacement_model,
        supplier_codes: item.supplier_codes,
        official_reference_prices: item
            .official_reference_prices
            .into_iter()
            .map(|price| AppModelCatalogReferencePriceResponse {
                region_code: price.region_code,
                billing_meter: price.billing_meter,
                unit_price: price.unit_price,
                currency: price.currency,
            })
            .collect(),
        price_availability,
    }
}

fn comma_separated_query_values(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn to_price_availability_response(
    official_reference_prices: &[crate::application::ModelCatalogReferencePriceView],
    availability: PriceAvailability,
) -> AppModelCatalogPriceAvailabilityResponse {
    if public_reference_price_is_configured(official_reference_prices) {
        return AppModelCatalogPriceAvailabilityResponse {
            status: "reference",
            reason: Some(PUBLIC_REFERENCE_PRICE_REASON.to_owned()),
        };
    }

    match availability {
        PriceAvailability::Available(_) | PriceAvailability::Unavailable { .. } => {
            AppModelCatalogPriceAvailabilityResponse {
                status: "unavailable",
                reason: Some(PUBLIC_PRICE_UNAVAILABLE_REASON.to_owned()),
            }
        }
    }
}

fn public_reference_price_is_configured(
    official_reference_prices: &[crate::application::ModelCatalogReferencePriceView],
) -> bool {
    official_reference_prices
        .iter()
        .any(|price| !price.unit_price.trim().is_empty() && !price.currency.trim().is_empty())
}
