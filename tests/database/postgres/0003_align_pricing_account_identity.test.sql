\set ON_ERROR_STOP on

BEGIN;

CREATE SCHEMA sdkwork_models_mig_0003_target;
CREATE SCHEMA sdkwork_models_mig_0003_fallback;
CREATE SCHEMA sdkwork_models_mig_0003_current;
CREATE SCHEMA sdkwork_models_mig_0003_ambiguous;

CREATE TABLE sdkwork_models_mig_0003_target.ai_model_pricing (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    supplier_code VARCHAR(64),
    provider_code VARCHAR(64),
    channel_id BIGINT,
    catalog_key VARCHAR(256) NOT NULL,
    price_side INTEGER,
    status INTEGER NOT NULL,
    effective_from TIMESTAMPTZ
);

CREATE INDEX idx_ai_model_pricing_provider_channel
    ON sdkwork_models_mig_0003_target.ai_model_pricing (
        tenant_id,
        organization_id,
        provider_code,
        channel_id,
        catalog_key,
        price_side,
        status,
        effective_from,
        id
    );

CREATE TABLE sdkwork_models_mig_0003_fallback.ai_model_pricing (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    supplier_code VARCHAR(64),
    provider_code VARCHAR(64),
    channel_id BIGINT,
    catalog_key VARCHAR(256) NOT NULL,
    price_side INTEGER,
    status INTEGER NOT NULL,
    effective_from TIMESTAMPTZ
);

CREATE INDEX idx_ai_model_pricing_provider_channel
    ON sdkwork_models_mig_0003_fallback.ai_model_pricing (
        tenant_id,
        organization_id,
        provider_code,
        channel_id,
        catalog_key,
        price_side,
        status,
        effective_from,
        id
    );

SET LOCAL search_path = sdkwork_models_mig_0003_target, sdkwork_models_mig_0003_fallback;
\ir ../../../database/migrations/postgres/0003_align_pricing_account_identity.up.sql

DO $test$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'sdkwork_models_mig_0003_target'
          AND table_name = 'ai_model_pricing'
          AND column_name = 'account_id'
          AND udt_name = 'int8'
    ) OR EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'sdkwork_models_mig_0003_target'
          AND table_name = 'ai_model_pricing'
          AND column_name = 'channel_id'
    ) THEN
        RAISE EXCEPTION 'legacy target schema was not upgraded';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'sdkwork_models_mig_0003_fallback'
          AND table_name = 'ai_model_pricing'
          AND column_name = 'channel_id'
    ) OR EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'sdkwork_models_mig_0003_fallback'
          AND table_name = 'ai_model_pricing'
          AND column_name = 'account_id'
    ) THEN
        RAISE EXCEPTION 'fallback schema decoy was modified';
    END IF;

    IF to_regclass('sdkwork_models_mig_0003_target.idx_ai_model_pricing_supplier_account') IS NULL
       OR to_regclass('sdkwork_models_mig_0003_target.idx_ai_model_pricing_provider_channel') IS NOT NULL THEN
        RAISE EXCEPTION 'legacy target indexes were not aligned';
    END IF;

    IF to_regclass('sdkwork_models_mig_0003_fallback.idx_ai_model_pricing_provider_channel') IS NULL
       OR to_regclass('sdkwork_models_mig_0003_fallback.idx_ai_model_pricing_supplier_account') IS NOT NULL THEN
        RAISE EXCEPTION 'fallback schema decoy indexes were modified';
    END IF;
END
$test$;

CREATE TABLE sdkwork_models_mig_0003_current.ai_model_pricing (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    supplier_code VARCHAR(64),
    provider_code VARCHAR(64),
    account_id BIGINT,
    catalog_key VARCHAR(256) NOT NULL,
    price_side INTEGER,
    status INTEGER NOT NULL,
    effective_from TIMESTAMPTZ
);

CREATE INDEX idx_ai_model_pricing_supplier_account
    ON sdkwork_models_mig_0003_current.ai_model_pricing (
        tenant_id,
        organization_id,
        supplier_code,
        account_id,
        catalog_key,
        price_side,
        status,
        effective_from,
        id
    );

SET LOCAL search_path = sdkwork_models_mig_0003_current;
\ir ../../../database/migrations/postgres/0003_align_pricing_account_identity.up.sql

CREATE TABLE sdkwork_models_mig_0003_ambiguous.ai_model_pricing (
    id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL,
    supplier_code VARCHAR(64),
    provider_code VARCHAR(64),
    channel_id BIGINT,
    account_id BIGINT,
    catalog_key VARCHAR(256) NOT NULL,
    price_side INTEGER,
    status INTEGER NOT NULL,
    effective_from TIMESTAMPTZ
);

SAVEPOINT ambiguous_shape;
SET LOCAL search_path = sdkwork_models_mig_0003_ambiguous;
\set ON_ERROR_STOP off
\ir ../../../database/migrations/postgres/0003_align_pricing_account_identity.up.sql
\set migration_failed :ERROR
\set ON_ERROR_STOP on
ROLLBACK TO SAVEPOINT ambiguous_shape;

\if :migration_failed
\else
    \echo 'ambiguous dual-column shape was not rejected'
    \quit 1
\endif

ROLLBACK;

\echo '0003_align_pricing_account_identity PostgreSQL migration tests passed'
