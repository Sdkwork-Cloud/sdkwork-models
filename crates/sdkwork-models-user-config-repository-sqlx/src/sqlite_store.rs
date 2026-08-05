//! SQLite implementation of the client-local user model configuration store.
//!
//! The authoritative schema is the database-client-local baseline DDL; this
//! store initializes its tables from that single source (`include_str!`).
//!
//! API keys never live in the SQLite file: raw credentials are stored in the
//! operating-system credential store (OS Keyring) behind the
//! `ApiKeySecretStore` port, and SQLite keeps only the `api_key_configured`
//! channel flag. This follows `DATABASE_SPEC.md` §33.4 ("Access/refresh
//! tokens, private keys, and raw credentials belong in OS secure storage or
//! an approved secret store, not ordinary SQLite columns").

use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    UserModelChannel, UserModelChannelModel, UserModelChannelOffering, UserModelConfigStore,
    UserModelConfigStoreError, UserModelConfigStoreResult, UserModelEngineConfig,
    UserModelEngineSelection,
};

/// Authoritative client-local DDL (single source of truth with
/// `database-client-local/ddl/baseline/sqlite/0001_user_model_config_baseline.sql`).
const USER_MODEL_CONFIG_SCHEMA_SQL: &str = include_str!(
    "../../../database-client-local/ddl/baseline/sqlite/0001_user_model_config_baseline.sql"
);

const API_KEY_CREDENTIAL_SERVICE: &str = "com.sdkwork.models.user-config";
const MAX_API_KEY_BYTES: usize = 16 * 1024;

/// Secret-store port for channel API keys. The SQLite store never persists
/// the raw credential; it only records the configured flag. Tests inject an
/// in-memory implementation, production uses the OS credential store.
pub trait ApiKeySecretStore: Send + Sync {
    fn read(&self, channel_code: &str) -> Result<Option<String>, String>;
    fn write(&self, channel_code: &str, api_key: &str) -> Result<(), String>;
    fn delete(&self, channel_code: &str) -> Result<(), String>;
}

fn api_key_account(channel_code: &str) -> String {
    format!("model-config.api-key.{channel_code}")
}

/// OS credential store backed API key secret store.
#[derive(Debug, Clone, Default)]
pub struct OsKeyringApiKeySecretStore;

impl ApiKeySecretStore for OsKeyringApiKeySecretStore {
    fn read(&self, channel_code: &str) -> Result<Option<String>, String> {
        match Entry::new(API_KEY_CREDENTIAL_SERVICE, &api_key_account(channel_code))
            .map_err(|_| "the operating-system credential store is unavailable".to_owned())?
            .get_secret()
        {
            Ok(value) => {
                let raw = String::from_utf8(value)
                    .map_err(|_| "stored channel API key is not valid UTF-8".to_owned())?;
                Ok(Some(raw))
            }
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err("the operating-system credential store is unavailable".to_owned()),
        }
    }

    fn write(&self, channel_code: &str, api_key: &str) -> Result<(), String> {
        if api_key.is_empty() {
            return Err("channel API key must not be empty".to_owned());
        }
        if api_key.len() > MAX_API_KEY_BYTES {
            return Err(format!(
                "channel API key exceeds the {MAX_API_KEY_BYTES}-byte credential limit"
            ));
        }
        Entry::new(API_KEY_CREDENTIAL_SERVICE, &api_key_account(channel_code))
            .map_err(|_| "the operating-system credential store is unavailable".to_owned())?
            .set_secret(api_key.as_bytes())
            .map_err(|_| "the operating-system credential store is unavailable".to_owned())
    }

    fn delete(&self, channel_code: &str) -> Result<(), String> {
        match Entry::new(API_KEY_CREDENTIAL_SERVICE, &api_key_account(channel_code))
            .map_err(|_| "the operating-system credential store is unavailable".to_owned())?
            .delete_credential()
        {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err("the operating-system credential store is unavailable".to_owned()),
        }
    }
}

/// In-memory API key secret store for tests and non-OS environments.
#[derive(Debug, Default)]
pub struct InMemoryApiKeySecretStore {
    values: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl ApiKeySecretStore for InMemoryApiKeySecretStore {
    fn read(&self, channel_code: &str) -> Result<Option<String>, String> {
        Ok(self
            .values
            .lock()
            .map_err(|_| "in-memory secret store lock is poisoned".to_owned())?
            .get(channel_code)
            .cloned())
    }

    fn write(&self, channel_code: &str, api_key: &str) -> Result<(), String> {
        self.values
            .lock()
            .map_err(|_| "in-memory secret store lock is poisoned".to_owned())?
            .insert(channel_code.to_owned(), api_key.to_owned());
        Ok(())
    }

    fn delete(&self, channel_code: &str) -> Result<(), String> {
        self.values
            .lock()
            .map_err(|_| "in-memory secret store lock is poisoned".to_owned())?
            .remove(channel_code);
        Ok(())
    }
}

pub struct SqliteUserModelConfigStore {
    pool: SqlitePool,
    api_key_secret_store: Arc<dyn ApiKeySecretStore>,
}

impl SqliteUserModelConfigStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            api_key_secret_store: Arc::new(OsKeyringApiKeySecretStore),
        }
    }

    /// Constructs the store with an explicit API key secret store; used by
    /// tests and by hosts that provide their own credential-store binding.
    pub fn with_api_key_secret_store(
        pool: SqlitePool,
        api_key_secret_store: Arc<dyn ApiKeySecretStore>,
    ) -> Self {
        Self {
            pool,
            api_key_secret_store,
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Creates the client-local tables when they do not exist. Idempotent and
    /// safe to call on every startup.
    pub async fn initialize_schema(&self) -> Result<(), String> {
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(|error| format!("acquire user model config connection: {error}"))?;
        sqlx::raw_sql(USER_MODEL_CONFIG_SCHEMA_SQL)
            .execute(&mut *connection)
            .await
            .map_err(|error| format!("initialize user model config schema: {error}"))?;
        Ok(())
    }
}

fn now_rfc3339() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    // RFC3339-compatible timestamp without external chrono dependency.
    let days = seconds / 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    let seconds_of_day = seconds % 86_400;
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    // Howard Hinnant's civil_from_days algorithm.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[derive(Debug, Serialize, Deserialize)]
struct StringList(Vec<String>);

fn encode_string_list(values: &[String]) -> String {
    serde_json::to_string(&StringList(values.to_vec())).unwrap_or_else(|_| "[]".to_owned())
}

fn decode_string_list(value: &str) -> Vec<String> {
    serde_json::from_str::<StringList>(value)
        .map(|list| list.0)
        .unwrap_or_default()
}

struct ChannelRow {
    code: String,
    name: String,
    kind: String,
    base_url: String,
    description: String,
    default_vendor_code: String,
    default_model_id: String,
    api_key_configured: i64,
    sort_order: Option<i64>,
}

struct OfferingRow {
    id: i64,
    channel_code: String,
    vendor_code: String,
    vendor_name: String,
    sort_order: i64,
}

struct ModelRow {
    offering_id: i64,
    model_id: String,
    display_name: String,
    context_tokens: Option<i64>,
    max_output_tokens: Option<i64>,
    tool_call_rounds: Option<i64>,
    supports_multimodal: i64,
    sort_order: i64,
}

async fn load_channel(
    pool: &SqlitePool,
    code: &str,
) -> UserModelConfigStoreResult<Option<UserModelChannel>> {
    let row = sqlx::query(
        "SELECT code, name, kind, base_url, description, default_vendor_code, default_model_id, \
         api_key_configured, sort_order \
         FROM user_model_channel WHERE code = ?",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let channel = ChannelRow {
        code: row.get("code"),
        name: row.get("name"),
        kind: row.get("kind"),
        base_url: row.get("base_url"),
        description: row.get("description"),
        default_vendor_code: row.get("default_vendor_code"),
        default_model_id: row.get("default_model_id"),
        api_key_configured: row.get("api_key_configured"),
        sort_order: row.get("sort_order"),
    };
    Ok(Some(load_channel_offerings(pool, channel).await?))
}

async fn load_channel_offerings(
    pool: &SqlitePool,
    channel: ChannelRow,
) -> UserModelConfigStoreResult<UserModelChannel> {
    let offerings = sqlx::query(
        "SELECT id, channel_code, vendor_code, vendor_name, sort_order \
         FROM user_model_channel_offering WHERE channel_code = ? ORDER BY sort_order, id",
    )
    .bind(&channel.code)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| OfferingRow {
        id: row.get("id"),
        channel_code: row.get("channel_code"),
        vendor_code: row.get("vendor_code"),
        vendor_name: row.get("vendor_name"),
        sort_order: row.get("sort_order"),
    })
    .collect::<Vec<_>>();
    let models = sqlx::query(
        "SELECT offering_id, model_id, display_name, context_tokens, max_output_tokens, \
         tool_call_rounds, supports_multimodal, sort_order \
         FROM user_model_channel_model ORDER BY sort_order, id",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| ModelRow {
        offering_id: row.get("offering_id"),
        model_id: row.get("model_id"),
        display_name: row.get("display_name"),
        context_tokens: row.get("context_tokens"),
        max_output_tokens: row.get("max_output_tokens"),
        tool_call_rounds: row.get("tool_call_rounds"),
        supports_multimodal: row.get("supports_multimodal"),
        sort_order: row.get("sort_order"),
    })
    .collect::<Vec<_>>();
    let offerings = offerings
        .into_iter()
        .map(|offering| {
            let vendor_models = models
                .iter()
                .filter(|model| model.offering_id == offering.id)
                .map(|model| UserModelChannelModel {
                    model_id: model.model_id.clone(),
                    display_name: model.display_name.clone(),
                    context_tokens: model.context_tokens,
                    max_output_tokens: model.max_output_tokens,
                    tool_call_rounds: model.tool_call_rounds,
                    supports_multimodal: model.supports_multimodal != 0,
                })
                .collect::<Vec<_>>();
            UserModelChannelOffering {
                vendor_code: offering.vendor_code,
                vendor_name: offering.vendor_name,
                models: vendor_models,
            }
        })
        .collect::<Vec<_>>();
    Ok(UserModelChannel {
        code: channel.code,
        name: channel.name,
        kind: channel.kind,
        base_url: channel.base_url,
        description: channel.description,
        default_vendor_code: channel.default_vendor_code,
        default_model_id: channel.default_model_id,
        api_key_configured: channel.api_key_configured != 0,
        sort_order: channel.sort_order,
        offerings,
    })
}

async fn replace_offerings(
    transaction: &mut Transaction<'_, Sqlite>,
    channel_code: &str,
    offerings: &[UserModelChannelOffering],
) -> UserModelConfigStoreResult<()> {
    // Retire existing offering rows (and their models via cascade) first.
    sqlx::query(
        "UPDATE user_model_channel_offering SET vendor_name = vendor_name || '_retired_' || \
         CAST(id AS TEXT), sort_order = sort_order + 1000000 WHERE channel_code = ?",
    )
    .bind(channel_code)
    .execute(&mut **transaction)
    .await
    .map_err(|error| {
        UserModelConfigStoreError::Message(format!("retire channel offerings failed: {error}"))
    })?;
    for (offering_index, offering) in offerings.iter().enumerate() {
        let offering_id = sqlx::query(
            "INSERT INTO user_model_channel_offering \
             (channel_code, vendor_code, vendor_name, sort_order) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT(channel_code, vendor_code) DO UPDATE SET \
             vendor_name = excluded.vendor_name, sort_order = excluded.sort_order \
             RETURNING id",
        )
        .bind(channel_code)
        .bind(&offering.vendor_code)
        .bind(&offering.vendor_name)
        .bind(offering_index as i64)
        .fetch_one(&mut **transaction)
        .await?
        .get::<i64, _>("id");
        sqlx::query("DELETE FROM user_model_channel_model WHERE offering_id = ?")
            .bind(offering_id)
            .execute(&mut **transaction)
            .await?;
        for (model_index, model) in offering.models.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_model_channel_model \
                 (offering_id, model_id, display_name, context_tokens, max_output_tokens, \
                  tool_call_rounds, supports_multimodal, sort_order) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(offering_id, model_id) DO UPDATE SET \
                 display_name = excluded.display_name, \
                 context_tokens = excluded.context_tokens, \
                 max_output_tokens = excluded.max_output_tokens, \
                 tool_call_rounds = excluded.tool_call_rounds, \
                 supports_multimodal = excluded.supports_multimodal, \
                 sort_order = excluded.sort_order",
            )
            .bind(offering_id)
            .bind(&model.model_id)
            .bind(&model.display_name)
            .bind(model.context_tokens)
            .bind(model.max_output_tokens)
            .bind(model.tool_call_rounds)
            .bind(if model.supports_multimodal { 1 } else { 0 })
            .bind(model_index as i64)
            .execute(&mut **transaction)
            .await?;
        }
    }
    // Purge offerings that were retired above and no longer exist.
    sqlx::query(
        "DELETE FROM user_model_channel_offering \
         WHERE channel_code = ? AND vendor_name LIKE '%_retired_%'",
    )
    .bind(channel_code)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

impl UserModelConfigStore for SqliteUserModelConfigStore {
    async fn list_channels(&self) -> UserModelConfigStoreResult<Vec<UserModelChannel>> {
        let codes = sqlx::query("SELECT code FROM user_model_channel ORDER BY sort_order, code")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.get::<String, _>("code"))
            .collect::<Vec<_>>();
        let mut channels = Vec::with_capacity(codes.len());
        for code in codes {
            if let Some(channel) = load_channel(&self.pool, &code).await? {
                channels.push(channel);
            }
        }
        Ok(channels)
    }

    async fn get_channel(&self, code: &str) -> UserModelConfigStoreResult<Option<UserModelChannel>> {
        load_channel(&self.pool, code).await
    }

    async fn upsert_channel(&self, channel: &UserModelChannel) -> UserModelConfigStoreResult<()> {
        let timestamp = now_rfc3339();
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO user_model_channel \
             (code, name, kind, base_url, description, default_vendor_code, default_model_id, \
              api_key_configured, sort_order, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(code) DO UPDATE SET \
             name = excluded.name, kind = excluded.kind, base_url = excluded.base_url, \
             description = excluded.description, \
             default_vendor_code = excluded.default_vendor_code, \
             default_model_id = excluded.default_model_id, \
             api_key_configured = excluded.api_key_configured, \
             sort_order = excluded.sort_order, updated_at = excluded.updated_at, \
             version = user_model_channel.version + 1",
        )
        .bind(&channel.code)
        .bind(&channel.name)
        .bind(&channel.kind)
        .bind(&channel.base_url)
        .bind(&channel.description)
        .bind(&channel.default_vendor_code)
        .bind(&channel.default_model_id)
        .bind(if channel.api_key_configured { 1 } else { 0 })
        .bind(channel.sort_order)
        .bind(&timestamp)
        .bind(&timestamp)
        .execute(&mut *transaction)
        .await?;
        replace_offerings(&mut transaction, &channel.code, &channel.offerings).await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn delete_channel(&self, code: &str) -> UserModelConfigStoreResult<()> {
        // Remove the credential first: if the OS store is unavailable the
        // channel must not be half-deleted while its credential survives.
        self.api_key_secret_store
            .delete(code)
            .map_err(|error| UserModelConfigStoreError::SecretStore(error))?;
        sqlx::query("DELETE FROM user_model_channel WHERE code = ?")
            .bind(code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn upsert_api_key(&self, key: &crate::UserModelApiKey) -> UserModelConfigStoreResult<()> {
        // Raw credentials go to the OS credential store only; the SQLite file
        // records the configured flag so channel lists can render key state
        // without ever exposing the credential.
        self.api_key_secret_store
            .write(&key.channel_code, &key.api_key)
            .map_err(|error| UserModelConfigStoreError::SecretStore(error))?;
        let timestamp = now_rfc3339();
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO user_model_key (channel_code, api_key, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(channel_code) DO UPDATE SET \
             api_key = excluded.api_key, updated_at = excluded.updated_at",
        )
        .bind(&key.channel_code)
        .bind("__keyring__")
        .bind(&timestamp)
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            "UPDATE user_model_channel SET api_key_configured = 1, updated_at = ? \
             WHERE code = ?",
        )
        .bind(&timestamp)
        .bind(&key.channel_code)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn get_api_key(&self, channel_code: &str) -> UserModelConfigStoreResult<Option<String>> {
        self.api_key_secret_store
            .read(channel_code)
            .map_err(|error| UserModelConfigStoreError::SecretStore(error))
    }

    async fn list_engine_configs(
        &self,
        engine_id: Option<&str>,
    ) -> UserModelConfigStoreResult<Vec<UserModelEngineConfig>> {
        let rows = match engine_id {
            Some(engine_id) => {
                sqlx::query(
                    "SELECT engine_id, channel_code, vendor_code, base_url, default_model_id, \
                     supported_model_ids, supported_provider_ids, input_context_tokens, \
                     output_context_tokens, tool_call_rounds, supports_multimodal, \
                     api_key_configured, applied_at \
                     FROM user_model_engine_config WHERE engine_id = ? ORDER BY applied_at, id",
                )
                .bind(engine_id)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT engine_id, channel_code, vendor_code, base_url, default_model_id, \
                     supported_model_ids, supported_provider_ids, input_context_tokens, \
                     output_context_tokens, tool_call_rounds, supports_multimodal, \
                     api_key_configured, applied_at \
                     FROM user_model_engine_config ORDER BY applied_at, id",
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows
            .into_iter()
            .map(|row| UserModelEngineConfig {
                engine_id: row.get("engine_id"),
                channel_code: row.get("channel_code"),
                vendor_code: row.get("vendor_code"),
                base_url: row.get("base_url"),
                default_model_id: row.get("default_model_id"),
                supported_model_ids: decode_string_list(row.get("supported_model_ids")),
                supported_provider_ids: decode_string_list(row.get("supported_provider_ids")),
                input_context_tokens: row.get("input_context_tokens"),
                output_context_tokens: row.get("output_context_tokens"),
                tool_call_rounds: row.get("tool_call_rounds"),
                supports_multimodal: row.get::<i64, _>("supports_multimodal") != 0,
                api_key_configured: row.get::<i64, _>("api_key_configured") != 0,
                applied_at: row.get("applied_at"),
            })
            .collect())
    }

    async fn upsert_engine_config(
        &self,
        config: &UserModelEngineConfig,
    ) -> UserModelConfigStoreResult<()> {        sqlx::query(
            "INSERT INTO user_model_engine_config \
             (engine_id, channel_code, vendor_code, base_url, default_model_id, \
              supported_model_ids, supported_provider_ids, input_context_tokens, \
              output_context_tokens, tool_call_rounds, supports_multimodal, \
              api_key_configured, applied_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(engine_id, channel_code) DO UPDATE SET \
             vendor_code = excluded.vendor_code, base_url = excluded.base_url, \
             default_model_id = excluded.default_model_id, \
             supported_model_ids = excluded.supported_model_ids, \
             supported_provider_ids = excluded.supported_provider_ids, \
             input_context_tokens = excluded.input_context_tokens, \
             output_context_tokens = excluded.output_context_tokens, \
             tool_call_rounds = excluded.tool_call_rounds, \
             supports_multimodal = excluded.supports_multimodal, \
             api_key_configured = excluded.api_key_configured, \
             applied_at = excluded.applied_at",
        )
        .bind(&config.engine_id)
        .bind(&config.channel_code)
        .bind(&config.vendor_code)
        .bind(&config.base_url)
        .bind(&config.default_model_id)
        .bind(encode_string_list(&config.supported_model_ids))
        .bind(encode_string_list(&config.supported_provider_ids))
        .bind(config.input_context_tokens)
        .bind(config.output_context_tokens)
        .bind(config.tool_call_rounds)
        .bind(if config.supports_multimodal { 1 } else { 0 })
        .bind(if config.api_key_configured { 1 } else { 0 })
        .bind(&config.applied_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_engine_config(
        &self,
        engine_id: &str,
        channel_code: &str,
    ) -> UserModelConfigStoreResult<()> {
        sqlx::query(
            "DELETE FROM user_model_engine_config WHERE engine_id = ? AND channel_code = ?",
        )
        .bind(engine_id)
        .bind(channel_code)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_engine_selections(
        &self,
    ) -> UserModelConfigStoreResult<Vec<UserModelEngineSelection>> {
        let rows = sqlx::query(
            "SELECT engine_id, channel_code, model_id FROM user_model_engine_selection \
             ORDER BY engine_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| UserModelEngineSelection {
                engine_id: row.get("engine_id"),
                channel_code: row.get("channel_code"),
                model_id: row.get("model_id"),
            })
            .collect())
    }

    async fn get_engine_selection(
        &self,
        engine_id: &str,
    ) -> UserModelConfigStoreResult<Option<UserModelEngineSelection>> {
        let row = sqlx::query(
            "SELECT engine_id, channel_code, model_id FROM user_model_engine_selection \
             WHERE engine_id = ?",
        )
        .bind(engine_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| UserModelEngineSelection {
            engine_id: row.get("engine_id"),
            channel_code: row.get("channel_code"),
            model_id: row.get("model_id"),
        }))
    }

    async fn upsert_engine_selection(
        &self,
        selection: &UserModelEngineSelection,
    ) -> UserModelConfigStoreResult<()> {
        sqlx::query(
            "INSERT INTO user_model_engine_selection (engine_id, channel_code, model_id, updated_at) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT(engine_id) DO UPDATE SET \
             channel_code = excluded.channel_code, model_id = excluded.model_id, \
             updated_at = excluded.updated_at",
        )
        .bind(&selection.engine_id)
        .bind(&selection.channel_code)
        .bind(&selection.model_id)
        .bind(now_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_engine_selection(
        &self,
        engine_id: &str,
    ) -> UserModelConfigStoreResult<()> {
        sqlx::query("DELETE FROM user_model_engine_selection WHERE engine_id = ?")
            .bind(engine_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
