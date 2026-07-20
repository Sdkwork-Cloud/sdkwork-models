//! Route crate for intelligence catalog app API (`sdkwork-routes-catalog-app-api`).

pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use http_route_manifest::app_route_manifest;
pub use routes::{route_definitions, RouteDefinition};
pub use sdkwork_models_catalog_service::{
    app_model_catalog_router, app_model_rankings_router, app_model_rankings_router_with_read_store,
    app_models_router,
};
pub use web_bootstrap::{
    intelligence_catalog_app_api_prefixes, intelligence_catalog_app_api_public_path_prefixes,
    wrap_router_with_web_framework, wrap_router_with_web_framework_from_env,
};

use sdkwork_web_core::HttpRouteManifest;

pub fn gateway_route_manifest() -> HttpRouteManifest {
    app_route_manifest()
}

pub fn gateway_mount() -> HttpRouteManifest {
    gateway_route_manifest()
}
