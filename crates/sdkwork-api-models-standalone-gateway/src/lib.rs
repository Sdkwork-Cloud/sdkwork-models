//! Canonical HTTP route composition for the `sdkwork-models` intelligence catalog product.

mod cors;
mod health_router;

pub use cors::application_cors_layer;
pub use health_router::{
    models_health_router, models_health_router_with_database_pool,
    models_health_router_with_readiness, ModelsReadinessProbe,
};
