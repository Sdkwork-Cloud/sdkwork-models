# SDKWork Models Catalog Database Module

Canonical lifecycle assets for `sdkwork-models` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `sdkwork-models`
- serviceCode: `SDKWORK_MODELS`
- tablePrefix: `ai_`

## Owned Tables

Catalog dictionary and public model facts imported from the JSON catalog:

- `ai_model_vendor`, `ai_model_family`, `ai_model`, `ai_model_capability`
- `ai_modality`, `ai_api_endpoint`, vendor/model modality and endpoint join tables
- `ai_resource`, `ai_resource_group`, `ai_resource_group_item`
- `ai_billing_meter`, `ai_model_pricing` (reference prices)
- `ai_model_catalog_source`, `ai_model_catalog_sync_run`, `ai_model_rank_snapshot`
- `ai_model_voice`, `ai_model_voice_binding` (TTS speaker catalog and model bindings)
- `ai_model_video_profile` (video generation profile catalog per video model)

Claw Router retains tenant routing overlays (`ai_model_mapping_*`), gateway channels, and tenant pricing plans.

## Initialization state

This module uses a baseline plus forward migrations:

1. **Baseline** — `database/ddl/baseline/postgres/0001_sdkwork-models_baseline.sql` contains the full PostgreSQL DDL snapshot.
2. **Migrations** — `database/migrations/postgres/` contains ordered, idempotent upgrades for databases created from an earlier baseline. Migration `0002_add_supplier_code` reconciles supplier ownership columns introduced after the initial baseline.
3. **Drift** — run `pnpm db:drift:check` before release.

The authoritative server contract is PostgreSQL-only. SQLite persistence, when required by a native client, must live in a separately owned `client-local` database module.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
