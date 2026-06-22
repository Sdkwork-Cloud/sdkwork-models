//! Catalog database bootstrap authority for the `sdkwork-models` product.
//!
//! Claw Router and other hosts must consume this crate instead of embedding
//! `sdkwork-models` DDL paths or table inventories in their own installers.

/// Canonical Postgres baseline for catalog dictionary tables owned by `sdkwork-models`.
pub fn models_catalog_foundation_migration_sql() -> &'static str {
    include_str!("../../../database/ddl/baseline/postgres/0001_sdkwork_models_catalog_baseline.sql")
}

/// SQLite mirror of the catalog module baseline for host installers and drift checks.
pub fn models_catalog_foundation_migration_sqlite() -> &'static str {
    include_str!("../../../database/ddl/baseline/sqlite/0001_sdkwork_models_catalog_baseline.sql")
}

/// Tables that must exist after the catalog module baseline is applied.
pub fn models_catalog_module_table_names() -> Vec<&'static str> {
    vec![
        "ai_model_vendor",
        "ai_modality",
        "ai_api_endpoint",
        "ai_vendor_modality",
        "ai_vendor_api_endpoint",
        "ai_modality_api_endpoint",
        "ai_model_modality",
        "ai_model_api_endpoint",
        "ai_resource",
        "ai_resource_group",
        "ai_resource_group_item",
        "ai_model_family",
        "ai_model",
        "ai_model_capability",
        "ai_model_catalog_source",
        "ai_model_catalog_sync_run",
        "ai_billing_meter",
        "ai_model_pricing",
        "ai_model_rank_snapshot",
    ]
}
