//! Route crate for intelligence catalog backend API (`sdkwork-routes-catalog-backend-api`).

pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use http_route_manifest::backend_route_manifest;
pub use routes::{route_definitions, RouteDefinition};
pub use sdkwork_models_catalog_service::{
    admin_ai_resource_router_with_store, admin_model_catalog_management_router_with_store,
    admin_model_catalog_router, admin_model_catalog_router_with_api_key_hasher,
    admin_model_catalog_router_with_store, admin_model_management_router_with_store,
    admin_model_rankings_router, admin_model_rankings_router_with_read_store,
    admin_model_rankings_router_with_read_store_and_refresh_store, app_video_profile_catalog_router,
    app_voice_catalog_router, backend_video_profile_catalog_router, backend_voice_catalog_router,
};
pub use web_bootstrap::{
    intelligence_catalog_backend_api_prefixes,
    intelligence_catalog_backend_api_public_path_prefixes, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};
