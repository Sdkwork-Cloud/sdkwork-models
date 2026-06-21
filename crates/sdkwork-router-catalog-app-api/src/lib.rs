//! Route crate for intelligence catalog app API (`sdkwork-router-catalog-app-api`).

pub mod http_route_manifest;
pub mod paths;
pub mod routes;

pub use http_route_manifest::app_route_manifest;
pub use routes::{route_definitions, RouteDefinition};
pub use sdkwork_models_catalog_service::{
    app_model_catalog_router, app_model_rankings_router, app_model_rankings_router_with_read_store,
    app_models_router,
};
