-- sdkwork:migration
-- id: 0002_add_supplier_code
-- engine: postgres
-- module: sdkwork-models
-- purpose: Reconcile supplier ownership columns added after the initial baseline
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s
-- contract_version: 1.0.1

ALTER TABLE ai_model_catalog_source
    ADD COLUMN IF NOT EXISTS supplier_code VARCHAR(64);

ALTER TABLE ai_model_catalog_sync_run
    ADD COLUMN IF NOT EXISTS supplier_code VARCHAR(64);

ALTER TABLE ai_model_pricing
    ADD COLUMN IF NOT EXISTS supplier_code VARCHAR(64);

ALTER TABLE ai_model_rank_snapshot
    ADD COLUMN IF NOT EXISTS supplier_code VARCHAR(64);
