# SQLite baseline

`0001_user_model_config_baseline.sql` is the authoritative client-local schema
for user model configuration. The `SqliteUserModelConfigStore` initializes its
schema from this file (`include_str!`), so this DDL is the single source of
truth.
