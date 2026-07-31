-- sdkwork:migration
-- id: 0003_align_pricing_account_identity
-- engine: postgres
-- module: sdkwork-models
-- purpose: Align model pricing ownership with canonical supplier account identity
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 30s
-- contract_version: 1.1.0
-- rewrite: metadata-only column rename; replacement index does not rewrite table rows
-- replication_wal: index replacement only; no row backfill or heap rewrite
-- backfill: none; channel_id values already represent upstream account ids
-- observability: verify account_id type, retired column absence, and exact index columns
-- cancellation: cancel before commit; the transaction restores the previous schema shape
-- recovery: resolve the reported conflicting shape, then rerun the migration

DO $sdkwork_migration$
DECLARE
    target_schema TEXT := current_schema();
    has_channel_id BOOLEAN;
    has_account_id BOOLEAN;
    actual_index_columns TEXT[];
BEGIN
    IF target_schema IS NULL THEN
        RAISE EXCEPTION 'sdkwork-models migration requires a canonical current schema';
    END IF;

    IF to_regclass(format('%I.%I', target_schema, 'ai_model_pricing')) IS NULL THEN
        RAISE EXCEPTION 'required table %.ai_model_pricing does not exist', target_schema;
    END IF;

    SELECT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = target_schema
          AND table_name = 'ai_model_pricing'
          AND column_name = 'channel_id'
    )
    INTO has_channel_id;

    SELECT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = target_schema
          AND table_name = 'ai_model_pricing'
          AND column_name = 'account_id'
    )
    INTO has_account_id;

    IF has_channel_id AND has_account_id THEN
        RAISE EXCEPTION 'ai_model_pricing contains both channel_id and account_id; refusing ambiguous ownership migration';
    ELSIF NOT has_channel_id AND NOT has_account_id THEN
        RAISE EXCEPTION 'ai_model_pricing contains neither channel_id nor account_id';
    ELSIF has_channel_id THEN
        EXECUTE format(
            'ALTER TABLE %I.%I RENAME COLUMN %I TO %I',
            target_schema,
            'ai_model_pricing',
            'channel_id',
            'account_id'
        );
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = target_schema
          AND table_name = 'ai_model_pricing'
          AND column_name = 'account_id'
          AND udt_name = 'int8'
    ) THEN
        RAISE EXCEPTION '%.ai_model_pricing.account_id must be BIGINT', target_schema;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_class relation
        JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
        WHERE namespace.nspname = target_schema
          AND relation.relname = 'idx_ai_model_pricing_provider_channel'
          AND relation.relkind <> 'i'
    ) THEN
        RAISE EXCEPTION 'relation %.idx_ai_model_pricing_provider_channel is not an index', target_schema;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_class relation
        JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
        WHERE namespace.nspname = target_schema
          AND relation.relname = 'idx_ai_model_pricing_provider_channel'
          AND relation.relkind = 'i'
    ) THEN
        EXECUTE format(
            'DROP INDEX %I.%I',
            target_schema,
            'idx_ai_model_pricing_provider_channel'
        );
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_class relation
        JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
        WHERE namespace.nspname = target_schema
          AND relation.relname = 'idx_ai_model_pricing_supplier_account'
          AND relation.relkind <> 'i'
    ) THEN
        RAISE EXCEPTION 'relation %.idx_ai_model_pricing_supplier_account is not an index', target_schema;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_class relation
        JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
        WHERE namespace.nspname = target_schema
          AND relation.relname = 'idx_ai_model_pricing_supplier_account'
          AND relation.relkind = 'i'
    ) THEN
        EXECUTE format(
            'CREATE INDEX %I ON %I.%I (tenant_id, organization_id, supplier_code, account_id, catalog_key, price_side, status, effective_from, id)',
            'idx_ai_model_pricing_supplier_account',
            target_schema,
            'ai_model_pricing'
        );
    END IF;

    SELECT array_agg(attribute.attname ORDER BY index_key.ordinality)
    INTO actual_index_columns
    FROM pg_class index_relation
    JOIN pg_namespace namespace
      ON namespace.oid = index_relation.relnamespace
    JOIN pg_index index_metadata
      ON index_metadata.indexrelid = index_relation.oid
    JOIN pg_class table_relation
      ON table_relation.oid = index_metadata.indrelid
    CROSS JOIN LATERAL unnest(index_metadata.indkey::SMALLINT[])
      WITH ORDINALITY AS index_key(attnum, ordinality)
    JOIN pg_attribute attribute
      ON attribute.attrelid = table_relation.oid
     AND attribute.attnum = index_key.attnum
    WHERE namespace.nspname = target_schema
      AND index_relation.relname = 'idx_ai_model_pricing_supplier_account'
      AND table_relation.relname = 'ai_model_pricing';

    IF actual_index_columns IS DISTINCT FROM ARRAY[
        'tenant_id',
        'organization_id',
        'supplier_code',
        'account_id',
        'catalog_key',
        'price_side',
        'status',
        'effective_from',
        'id'
    ]::TEXT[] THEN
        RAISE EXCEPTION 'idx_ai_model_pricing_supplier_account has invalid columns: %', actual_index_columns;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = target_schema
          AND table_name = 'ai_model_pricing'
          AND column_name = 'channel_id'
    ) THEN
        RAISE EXCEPTION 'retired %.ai_model_pricing.channel_id still exists', target_schema;
    END IF;
END
$sdkwork_migration$;
