use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_models::{
    list_video_profiles, list_video_profiles_for_model, ModelCatalog, VideoGenerationProfile,
    VideoProfileFilter,
};
use sdkwork_utils_rust::{PageInfo, PageMode, SdkWorkPageData, SdkWorkResultCode};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;

use crate::api::response::{finish_success, problem_for};

#[derive(Debug, Clone, Copy)]
enum VideoProfileRouteSurface {
    App,
    Backend,
}

impl VideoProfileRouteSurface {
    fn list_profiles_path(self) -> &'static str {
        match self {
            Self::App => "/app/v3/api/ai/video_profiles",
            Self::Backend => "/backend/v3/api/ai/video_profiles",
        }
    }

    fn list_model_profiles_path(self) -> &'static str {
        match self {
            Self::App => "/app/v3/api/ai/models/{modelId}/video_profiles",
            Self::Backend => "/backend/v3/api/ai/models/{modelId}/video_profiles",
        }
    }
}

struct VideoProfileCatalogState {
    catalog: Arc<ModelCatalog>,
}

impl Clone for VideoProfileCatalogState {
    fn clone(&self) -> Self {
        Self {
            catalog: Arc::clone(&self.catalog),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct VideoProfileCatalogQuery {
    vendor_code: Option<String>,
    region_code: Option<String>,
    model_id: Option<String>,
    catalog_key: Option<String>,
    generation_mode: Option<String>,
    duration_tier_code: Option<String>,
    resolution: Option<String>,
}

fn video_profile_list_page(
    items: Vec<VideoGenerationProfile>,
) -> SdkWorkPageData<VideoGenerationProfile> {
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

fn collect_video_profiles(
    catalog: &ModelCatalog,
    query: &VideoProfileCatalogQuery,
) -> Vec<VideoGenerationProfile> {
    let model_catalog_key = query.catalog_key.clone().or_else(|| {
        if let (Some(vendor_code), Some(model_id)) = (&query.vendor_code, &query.model_id) {
            Some(format!("{vendor_code}/{model_id}"))
        } else {
            None
        }
    });
    list_video_profiles(
        catalog,
        VideoProfileFilter {
            vendor_code: query.vendor_code.as_deref(),
            region_code: query.region_code.as_deref(),
            model_catalog_key: model_catalog_key.as_deref(),
            generation_mode: query.generation_mode.as_deref(),
            duration_tier_code: query.duration_tier_code.as_deref(),
            resolution: query.resolution.as_deref(),
        },
    )
    .into_iter()
    .cloned()
    .collect()
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ModelVideoProfilePath {
    modelId: String,
}

fn video_profile_catalog_router(
    catalog: Arc<ModelCatalog>,
    surface: VideoProfileRouteSurface,
) -> Router {
    let state = VideoProfileCatalogState { catalog };
    Router::new()
        .route(
            surface.list_profiles_path(),
            get(list_video_profiles_handler),
        )
        .route(
            surface.list_model_profiles_path(),
            get(list_model_video_profiles_handler),
        )
        .with_state(state)
}

pub fn app_video_profile_catalog_router(catalog: Arc<ModelCatalog>) -> Router {
    video_profile_catalog_router(catalog, VideoProfileRouteSurface::App)
}

pub fn backend_video_profile_catalog_router(catalog: Arc<ModelCatalog>) -> Router {
    video_profile_catalog_router(catalog, VideoProfileRouteSurface::Backend)
}

async fn list_video_profiles_handler(
    ctx: WebRequestContext,
    State(state): State<VideoProfileCatalogState>,
    Query(query): Query<VideoProfileCatalogQuery>,
) -> Response {
    let items = collect_video_profiles(state.catalog.as_ref(), &query);
    finish_success(&ctx, video_profile_list_page(items))
}

async fn list_model_video_profiles_handler(
    ctx: WebRequestContext,
    State(state): State<VideoProfileCatalogState>,
    Path(path): Path<ModelVideoProfilePath>,
    Query(query): Query<VideoProfileCatalogQuery>,
) -> Response {
    let catalog_key = if path.modelId.contains('/') {
        path.modelId
    } else if let Some(vendor_code) = &query.vendor_code {
        format!("{vendor_code}/{}", path.modelId)
    } else {
        return problem_for(
            &ctx,
            SdkWorkResultCode::ValidationError,
            "vendor_code is required when modelId is not a catalog key",
        );
    };
    let items = list_video_profiles_for_model(state.catalog.as_ref(), &catalog_key)
        .into_iter()
        .cloned()
        .collect();
    finish_success(&ctx, video_profile_list_page(items))
}
