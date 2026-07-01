# SQLite baseline

SQLite baseline DDL for the catalog module is applied through host installers that call `sdkwork-models-database-bootstrap`. Authoritative Postgres baseline lives in `../postgres/0001_sdkwork-models_baseline.sql`; SQLite-specific DDL will be materialized here when drift checks require a checked-in sqlite mirror.
