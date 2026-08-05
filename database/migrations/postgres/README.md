# PostgreSQL Migrations

Pre-launch the Models Catalog schema is consolidated on the single greenfield
baseline: `database/ddl/baseline/postgres/0001_sdkwork-models_baseline.sql`.
It contains the complete initial schema (vendor/family/model/capability
catalog, modalities and endpoints, resources, billing meters and pricing with
supplier ownership, catalog sources and sync runs, rank snapshots, voice and
video profiles, and model usage scopes).

No ordered post-baseline migrations exist while the app is pre-launch; the
lifecycle orchestrator applies the baseline once on an empty schema
(`baseline-plus-migrations`, `lifecycle.autoMigrate=false`). The drift gate
then verifies the live schema against `database/contract/`. Shared development
schemas converge by resetting the module state to the baseline instead of
replaying forward-only migrations.

After the first production release, add ordered expand/contract migrations here
without rewriting the released baseline; the previous greenfield migration
inventory was folded into the baseline before launch.
