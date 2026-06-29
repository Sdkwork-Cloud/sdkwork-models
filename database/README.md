# SDKWork Models Catalog Database Module

Canonical lifecycle assets for `sdkwork-models` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `sdkwork-models`
- serviceCode: `SDKWORK_MODELS`
- tablePrefix: `ai_model_` (catalog dictionary tables retain legacy `ai_*` names during composed migration)

## Owned Tables

Catalog dictionary and public model facts imported from the JSON catalog:

- `ai_model_vendor`, `ai_model_family`, `ai_model`, `ai_model_capability`
- `ai_modality`, `ai_api_endpoint`, vendor/model modality and endpoint join tables
- `ai_resource`, `ai_resource_group`, `ai_resource_group_item`
- `ai_billing_meter`, `ai_model_pricing` (reference prices)
- `ai_model_catalog_source`, `ai_model_catalog_sync_run`, `ai_model_rank_snapshot`

Claw Router retains tenant routing overlays (`ai_model_mapping_*`), gateway channels, and tenant pricing plans.

## Commands

Catalog DDL authority lives in this module. Composed hosts such as `sdkwork-clawrouter`
must consume `crates/sdkwork-models-database-bootstrap` for baseline SQL and table
inventory, matching the `sdkwork-commerce (deleted) (deleted)` / `sdkwork-appstore` composed-module pattern.

Hosts must not embed `database/ddl` paths or duplicate catalog table inventories in
their own installers.
