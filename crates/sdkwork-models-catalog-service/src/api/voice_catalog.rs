use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_models::{list_voices, list_voices_for_model, ModelCatalog, TtsVoice, VoiceFilter};
use sdkwork_utils_rust::{PageInfo, PageMode, SdkWorkPageData, SdkWorkResultCode};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;

use crate::api::response::{finish_success, problem_for};

#[derive(Debug, Clone, Copy)]
enum VoiceRouteSurface {
    App,
    Backend,
}

impl VoiceRouteSurface {
    fn list_voices_path(self) -> &'static str {
        match self {
            Self::App => "/app/v3/api/ai/voices",
            Self::Backend => "/backend/v3/api/ai/voices",
        }
    }

    fn list_model_voices_path(self) -> &'static str {
        match self {
            Self::App => "/app/v3/api/ai/models/{modelId}/voices",
            Self::Backend => "/backend/v3/api/ai/models/{modelId}/voices",
        }
    }
}

struct VoiceCatalogState {
    catalog: Arc<ModelCatalog>,
}

impl Clone for VoiceCatalogState {
    fn clone(&self) -> Self {
        Self {
            catalog: Arc::clone(&self.catalog),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct VoiceCatalogQuery {
    vendor_code: Option<String>,
    region_code: Option<String>,
    locale: Option<String>,
    model_id: Option<String>,
    catalog_key: Option<String>,
    q: Option<String>,
}

fn voice_list_page(items: Vec<TtsVoice>) -> SdkWorkPageData<TtsVoice> {
    let total = items.len();
    SdkWorkPageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: None,
            page_size: None,
            total_items: Some(total.to_string()),
            total_pages: None,
            next_cursor: None,
            has_more: Some(false),
        },
    }
}

fn collect_voices(catalog: &ModelCatalog, query: &VoiceCatalogQuery) -> Vec<TtsVoice> {
    let model_catalog_key = query.catalog_key.clone().or_else(|| {
        if let (Some(vendor_code), Some(model_id)) = (&query.vendor_code, &query.model_id) {
            Some(format!("{vendor_code}/{model_id}"))
        } else {
            None
        }
    });
    list_voices(
        catalog,
        VoiceFilter {
            vendor_code: query.vendor_code.as_deref(),
            region_code: query.region_code.as_deref(),
            locale: query.locale.as_deref(),
            model_catalog_key: model_catalog_key.as_deref(),
            search_query: query.q.as_deref(),
        },
    )
    .into_iter()
    .cloned()
    .collect()
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ModelVoicePath {
    modelId: String,
}

fn voice_catalog_router(catalog: Arc<ModelCatalog>, surface: VoiceRouteSurface) -> Router {
    let state = VoiceCatalogState { catalog };
    let list_model_voices = match surface {
        VoiceRouteSurface::App => get(list_app_model_voices_handler),
        VoiceRouteSurface::Backend => get(list_backend_model_voices_handler),
    };
    Router::new()
        .route(surface.list_voices_path(), get(list_voices_handler))
        .route(surface.list_model_voices_path(), list_model_voices)
        .with_state(state)
}

pub fn app_voice_catalog_router(catalog: Arc<ModelCatalog>) -> Router {
    voice_catalog_router(catalog, VoiceRouteSurface::App)
}

pub fn backend_voice_catalog_router(catalog: Arc<ModelCatalog>) -> Router {
    voice_catalog_router(catalog, VoiceRouteSurface::Backend)
}

async fn list_voices_handler(
    ctx: WebRequestContext,
    State(state): State<VoiceCatalogState>,
    Query(query): Query<VoiceCatalogQuery>,
) -> Response {
    let items = collect_voices(state.catalog.as_ref(), &query);
    finish_success(&ctx, voice_list_page(items))
}

async fn list_app_model_voices_handler(
    ctx: WebRequestContext,
    State(state): State<VoiceCatalogState>,
    Path(path): Path<ModelVoicePath>,
    Query(query): Query<VoiceCatalogQuery>,
) -> Response {
    respond_model_voices(&ctx, state.catalog.as_ref(), path.modelId, &query)
}

async fn list_backend_model_voices_handler(
    ctx: WebRequestContext,
    State(state): State<VoiceCatalogState>,
    Path(path): Path<ModelVoicePath>,
    Query(query): Query<VoiceCatalogQuery>,
) -> Response {
    respond_model_voices(&ctx, state.catalog.as_ref(), path.modelId, &query)
}

fn respond_model_voices(
    ctx: &WebRequestContext,
    catalog: &ModelCatalog,
    model_id: String,
    query: &VoiceCatalogQuery,
) -> Response {
    let catalog_key = if model_id.contains('/') {
        model_id
    } else if let Some(vendor_code) = &query.vendor_code {
        format!("{vendor_code}/{model_id}")
    } else {
        return problem_for(
            ctx,
            SdkWorkResultCode::ValidationError,
            "vendor_code is required when model_id is not a catalog key",
        );
    };
    let items = list_voices_for_model(catalog, &catalog_key)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    finish_success(ctx, voice_list_page(items))
}
