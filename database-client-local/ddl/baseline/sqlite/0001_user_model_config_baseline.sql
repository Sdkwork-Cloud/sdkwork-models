-- SDKWork user model configuration client-local baseline (sqlite)
-- Authoritative schema for the user's model access channels, per-tool
-- (agent engine) configurations, and locally persisted API keys.
-- Fully decoupled from the server-side ai_resource catalog tables.

CREATE TABLE IF NOT EXISTS ops_schema_migration (
    id INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    migration_id TEXT NOT NULL,
    contract_version TEXT NOT NULL,
    status TEXT NOT NULL,
    applied_at TEXT NOT NULL,
    checksum TEXT NOT NULL,
    diagnostics_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_ops_schema_migration_uuid UNIQUE (uuid),
    CONSTRAINT uk_ops_schema_migration_provider_migration UNIQUE (provider_id, migration_id)
);

CREATE INDEX IF NOT EXISTS idx_ops_schema_migration_status_applied
    ON ops_schema_migration (provider_id, status, applied_at, id);

-- Model access channel owned by the local user (code == configurationId).
CREATE TABLE IF NOT EXISTS user_model_channel (
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('official', 'relay', 'custom')),
    base_url TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    default_vendor_code TEXT NOT NULL,
    default_model_id TEXT NOT NULL,
    api_key_configured INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (code)
);

-- Vendor offering within a channel.
CREATE TABLE IF NOT EXISTS user_model_channel_offering (
    id INTEGER NOT NULL,
    channel_code TEXT NOT NULL,
    vendor_code TEXT NOT NULL,
    vendor_name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT uk_user_model_channel_offering UNIQUE (channel_code, vendor_code),
    CONSTRAINT fk_user_model_channel_offering_channel
        FOREIGN KEY (channel_code) REFERENCES user_model_channel (code) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_model_channel_offering_channel
    ON user_model_channel_offering (channel_code, sort_order, id);

-- Model row within a vendor offering, including capability metadata.
CREATE TABLE IF NOT EXISTS user_model_channel_model (
    id INTEGER NOT NULL,
    offering_id INTEGER NOT NULL,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    context_tokens INTEGER,
    max_output_tokens INTEGER,
    tool_call_rounds INTEGER,
    supports_multimodal INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT uk_user_model_channel_model UNIQUE (offering_id, model_id),
    CONSTRAINT fk_user_model_channel_model_offering
        FOREIGN KEY (offering_id) REFERENCES user_model_channel_offering (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_model_channel_model_offering
    ON user_model_channel_model (offering_id, sort_order, id);

-- Locally persisted API key for a channel (plaintext at OS file permission
-- level; reserved for future encryption-at-rest upgrade).
CREATE TABLE IF NOT EXISTS user_model_key (
    channel_code TEXT NOT NULL,
    api_key TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (channel_code),
    CONSTRAINT fk_user_model_key_channel
        FOREIGN KEY (channel_code) REFERENCES user_model_channel (code) ON DELETE CASCADE
);

-- Applied configuration per agent engine (tool) for a channel.
CREATE TABLE IF NOT EXISTS user_model_engine_config (
    id INTEGER NOT NULL,
    engine_id TEXT NOT NULL,
    channel_code TEXT NOT NULL,
    vendor_code TEXT NOT NULL,
    base_url TEXT NOT NULL,
    default_model_id TEXT NOT NULL,
    supported_model_ids TEXT NOT NULL DEFAULT '[]',
    supported_provider_ids TEXT NOT NULL DEFAULT '[]',
    input_context_tokens INTEGER,
    output_context_tokens INTEGER,
    tool_call_rounds INTEGER,
    supports_multimodal INTEGER NOT NULL DEFAULT 0,
    api_key_configured INTEGER NOT NULL DEFAULT 0,
    applied_at TEXT NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT uk_user_model_engine_config UNIQUE (engine_id, channel_code),
    CONSTRAINT fk_user_model_engine_config_channel
        FOREIGN KEY (channel_code) REFERENCES user_model_channel (code) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_model_engine_config_engine
    ON user_model_engine_config (engine_id, applied_at, id);

-- Active selection per agent engine.
CREATE TABLE IF NOT EXISTS user_model_engine_selection (
    engine_id TEXT NOT NULL,
    channel_code TEXT NOT NULL,
    model_id TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (engine_id),
    CONSTRAINT fk_user_model_engine_selection_channel
        FOREIGN KEY (channel_code) REFERENCES user_model_channel (code) ON DELETE CASCADE
);
