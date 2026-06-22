use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

use crate::paths::SDK_DOMAIN;

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/ai/model_vendors",
        SDK_DOMAIN,
        "modelVendors.list",
    )
    .with_required_permission("intelligence.models.read"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/ai/models",
        SDK_DOMAIN,
        "models.list",
    )
    .with_required_permission("intelligence.models.read"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/ai/model_rankings",
        SDK_DOMAIN,
        "modelRankings.list",
    )
    .with_required_permission("intelligence.models.read"),
];

pub fn app_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
