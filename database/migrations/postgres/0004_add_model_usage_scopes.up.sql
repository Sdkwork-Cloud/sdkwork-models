-- sdkwork:migration
-- id: 0004_add_model_usage_scopes
-- engine: postgres
-- module: sdkwork-models
-- purpose: Add model usage scope and code IDE visibility columns to ai_model
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s
-- contract_version: 1.1.0
-- rewrite: column addition only; no row backfill beyond column defaults

ALTER TABLE ai_model
    ADD COLUMN IF NOT EXISTS usage_scopes JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE ai_model
    ADD COLUMN IF NOT EXISTS coding_visible BOOLEAN NOT NULL DEFAULT TRUE;
