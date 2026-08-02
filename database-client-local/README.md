# User Model Configuration Client-Local Database

Client-local SQLite database owned by `sdkwork-models` for the user's model
access channels (official / relay / custom), locally persisted API keys, and
per-agent-engine (tool) configurations and selections.

- Fully decoupled from the server-side `ai_resource` catalog tables.
- Single source of truth: `ddl/baseline/sqlite/0001_user_model_config_baseline.sql`
  (consumed by `SqliteUserModelConfigStore::initialize_schema` via `include_str!`).
- Runtime file: `<app-data>/birdcoder-user-config.sqlite3`
  (`SDKWORK_USER_MODEL_CONFIG_SQLITE_URL`).
- Keys are stored plaintext at OS file-permission level; encryption-at-rest is
  reserved for a future upgrade (`local-data-policy.yaml`).
