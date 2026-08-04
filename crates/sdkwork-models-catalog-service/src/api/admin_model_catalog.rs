use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, Uri};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_cloudrouter_http::ApiKeyIdentity;
use sdkwork_utils_rust::SdkWorkResultCode;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};

use crate::api::page_info::{offset_page_info, ApiPageInfo};
use crate::api::response::{finish_success, problem_for};
use crate::application::{
    ApiKeyAuthenticator, ApiKeySecretHasher, AuthenticateApiKeyQuery, ListModelCatalogQuery,
    ModelCatalogItem, ModelCatalogPage, ModelCatalogPriceView, ModelCatalogQueryService,
    PriceAvailability,
};
use crate::domain::BillingMeter;
use crate::ports::PricingCatalog;

struct AdminModelCatalogState<C> {
    catalog: Arc<C>,
    api_key_hasher: Option<Arc<dyn ApiKeySecretHasher + Send + Sync>>,
}

impl<C> Clone for AdminModelCatalogState<C> {
    fn clone(&self) -> Self {
        Self {
            catalog: Arc::clone(&self.catalog),
            api_key_hasher: self.api_key_hasher.clone(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminModelListRequest {
    api_key_id: Option<i64>,
    billing_meter: Option<String>,
    vendor_code: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminModelListResponse {
    items: Vec<AdminModelItemResponse>,
    page_info: ApiPageInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminModelItemResponse {
    model: String,
    display_name: String,
    vendor_code: String,
    vendor: String,
    capabilities: Vec<String>,
    supplier_codes: Vec<String>,
    official_reference_unit_price: Option<String>,
    lowest_upstream_cost_unit_price: Option<String>,
    price_availability: AdminPriceAvailabilityResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminPriceAvailabilityResponse {
    status: &'static str,
    group_code: Option<String>,
    pricing_plan_code: Option<String>,
    customer_unit_price: Option<String>,
    gross_margin_per_unit: Option<String>,
    reason: Option<String>,
}

pub fn admin_model_catalog_router<C>(catalog: Arc<C>) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    admin_model_catalog_router_with_optional_api_key_hasher(catalog, None)
}

pub fn admin_model_catalog_router_with_api_key_hasher<C>(
    catalog: Arc<C>,
    api_key_hasher: Arc<dyn ApiKeySecretHasher + Send + Sync>,
) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    admin_model_catalog_router_with_optional_api_key_hasher(catalog, Some(api_key_hasher))
}

fn admin_model_catalog_router_with_optional_api_key_hasher<C>(
    catalog: Arc<C>,
    api_key_hasher: Option<Arc<dyn ApiKeySecretHasher + Send + Sync>>,
) -> Router
where
    C: PricingCatalog + Send + Sync + 'static,
{
    Router::new()
        .route("/backend/v3/api/ai/models", get(fetch_models::<C>))
        .with_state(AdminModelCatalogState {
            catalog,
            api_key_hasher,
        })
}

async fn fetch_models<C>(
    ctx: WebRequestContext,
    State(state): State<AdminModelCatalogState<C>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response
where
    C: PricingCatalog + Send + Sync + 'static,
{
    let request = match parse_query(uri.query()) {
        Ok(request) => request,
        Err(message) => {
            return problem_for(&ctx, SdkWorkResultCode::ValidationError, message);
        }
    };
    let (page, page_size) = match validate_page_query(request.page, request.page_size) {
        Ok(value) => value,
        Err(message) => {
            return problem_for(&ctx, SdkWorkResultCode::ValidationError, message);
        }
    };
    let offset = (page - 1) * page_size;
    let identity = match ApiKeyIdentity::from_headers_and_uri(&headers, &uri) {
        Ok(identity) => identity,
        Err(error) => {
            return problem_for(&ctx, SdkWorkResultCode::ValidationError, error.to_string());
        }
    };
    let api_key_id = match resolve_api_key_id(&state, &identity, request.api_key_id) {
        Ok(api_key_id) => api_key_id,
        Err((code, message)) => return problem_for(&ctx, code, message),
    };
    let billing_meter = request
        .billing_meter
        .as_deref()
        .map(BillingMeter::from_code)
        .unwrap_or(BillingMeter::LlmInputToken);

    let service = ModelCatalogQueryService::new(state.catalog.as_ref());
    match service.list_models(ListModelCatalogQuery {
        api_key_id,
        billing_meter: billing_meter.clone(),
        vendor_code: request.vendor_code,
        vendor_codes: Vec::new(),
        modalities: Vec::new(),
        capabilities: Vec::new(),
        categories: Vec::new(),
        groups: Vec::new(),
        search_query: None,
        page_size: Some(page_size),
        offset: Some(offset),
    }) {
        Ok(page) => finish_success(&ctx, to_response(page, &billing_meter)),
        Err(error) => problem_for(&ctx, SdkWorkResultCode::ValidationError, error.to_string()),
    }
}

fn parse_query(query: Option<&str>) -> Result<AdminModelListRequest, String> {
    let Some(query) = query else {
        return Ok(AdminModelListRequest::default());
    };
    let mut request = AdminModelListRequest::default();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair
            .split_once('=')
            .map(|(name, value)| (name.trim(), value.trim()))
            .unwrap_or_else(|| (pair.trim(), ""));
        match name {
            "api_key_id" if !value.is_empty() => {
                let api_key_id = value
                    .parse::<i64>()
                    .map_err(|_| "api_key_id must be a positive integer".to_owned())?;
                if api_key_id <= 0 {
                    return Err("api_key_id must be a positive integer".to_owned());
                }
                request.api_key_id = Some(api_key_id);
            }
            "billing_meter" if !value.is_empty() => {
                request.billing_meter = Some(value.to_owned());
            }
            "vendor_code" if !value.is_empty() => {
                request.vendor_code = Some(value.to_owned());
            }
            "page" if !value.is_empty() => {
                let page_value = value
                    .parse::<usize>()
                    .map_err(|_| "page must be a positive integer".to_owned())?;
                request.page = Some(page_value);
            }
            "page_size" if !value.is_empty() => {
                let size = value
                    .parse::<usize>()
                    .map_err(|_| "page_size must be a positive integer".to_owned())?;
                request.page_size = Some(size);
            }
            "" => {}
            _ => {}
        }
    }
    Ok(request)
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

fn resolve_api_key_id<C>(
    state: &AdminModelCatalogState<C>,
    identity: &ApiKeyIdentity,
    request_api_key_id: Option<i64>,
) -> Result<Option<i64>, (SdkWorkResultCode, &'static str)>
where
    C: PricingCatalog + Send + Sync + 'static,
{
    if let Some(hasher) = state.api_key_hasher.as_ref() {
        let Some(credential_secret) = identity.credential_secret() else {
            return Err((
                SdkWorkResultCode::AuthenticationRequired,
                "api key credential is required",
            ));
        };

        let authenticator = ApiKeyAuthenticator::new(state.catalog.as_ref(), hasher.as_ref());
        return authenticator
            .authenticate(AuthenticateApiKeyQuery { credential_secret })
            .map(|context| Some(context.api_key_id))
            .map_err(|_| {
                (
                    SdkWorkResultCode::InvalidToken,
                    "api key credential is invalid",
                )
            });
    }

    if let Some(api_key_id) = identity.api_key_id() {
        return Ok(Some(api_key_id));
    }
    if identity.credential_secret().is_none() {
        return Ok(request_api_key_id);
    }
    Err((
        SdkWorkResultCode::AuthenticationRequired,
        "api key credential authentication is not configured",
    ))
}

fn to_response(page: ModelCatalogPage, billing_meter: &BillingMeter) -> AdminModelListResponse {
    AdminModelListResponse {
        items: page
            .items
            .into_iter()
            .map(|item| to_item_response(item, billing_meter))
            .collect(),
        page_info: offset_page_info(
            ((page.offset / page.page_size) + 1) as i64,
            page.page_size as i64,
            page.total_items as i64,
        ),
    }
}

fn to_item_response(
    item: ModelCatalogItem,
    billing_meter: &BillingMeter,
) -> AdminModelItemResponse {
    let official_reference_unit_price =
        selected_reference_price(&item, billing_meter).map(|price| price.unit_price.clone());
    AdminModelItemResponse {
        model: item.model,
        display_name: item.display_name,
        vendor_code: item.vendor_code,
        vendor: item.vendor.code().to_owned(),
        capabilities: item.capabilities,
        supplier_codes: item.supplier_codes,
        official_reference_unit_price,
        lowest_upstream_cost_unit_price: item.lowest_upstream_cost_unit_price,
        price_availability: to_price_availability_response(item.price_availability),
    }
}

fn selected_reference_price<'a>(
    item: &'a ModelCatalogItem,
    billing_meter: &BillingMeter,
) -> Option<&'a crate::application::ModelCatalogReferencePriceView> {
    let meter_code = billing_meter.code();
    item.official_reference_prices
        .iter()
        .filter(|price| price.billing_meter == meter_code)
        .min_by_key(|price| reference_region_sort_key(&price.region_code))
}

fn reference_region_sort_key(region_code: &str) -> usize {
    match region_code.trim().to_ascii_lowercase().as_str() {
        "global" => 0,
        "cn" | "china" | "mainland" => 10,
        _ => 20,
    }
}

fn to_price_availability_response(
    availability: PriceAvailability,
) -> AdminPriceAvailabilityResponse {
    match availability {
        PriceAvailability::Available(price) => available_price(price),
        PriceAvailability::Unavailable { reason } => AdminPriceAvailabilityResponse {
            status: "unavailable",
            group_code: None,
            pricing_plan_code: None,
            customer_unit_price: None,
            gross_margin_per_unit: None,
            reason: Some(reason),
        },
    }
}

fn available_price(price: ModelCatalogPriceView) -> AdminPriceAvailabilityResponse {
    AdminPriceAvailabilityResponse {
        status: "available",
        group_code: Some(price.group_code),
        pricing_plan_code: Some(price.pricing_plan_code),
        customer_unit_price: Some(price.customer_unit_price),
        gross_margin_per_unit: price.gross_margin_per_unit,
        reason: None,
    }
}
