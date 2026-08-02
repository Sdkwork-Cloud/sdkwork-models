use sdkwork_models_user_config_repository_sqlx::sqlite_store::SqliteUserModelConfigStore;
use sdkwork_models_user_config_repository_sqlx::{
    UserModelApiKey, UserModelChannel, UserModelChannelModel, UserModelChannelOffering,
    UserModelConfigStore, UserModelEngineConfig, UserModelEngineSelection,
};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

async fn test_pool() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect in-memory SQLite")
}

async fn test_store() -> SqliteUserModelConfigStore {
    let store = SqliteUserModelConfigStore::new(test_pool().await);
    store.initialize_schema().await.expect("initialize schema");
    store
}

fn sample_channel(code: &str) -> UserModelChannel {
    UserModelChannel {
        code: code.to_owned(),
        name: "Team Relay".to_owned(),
        kind: "relay".to_owned(),
        base_url: "https://relay.example.com/v1".to_owned(),
        description: "team gateway".to_owned(),
        default_vendor_code: "openai".to_owned(),
        default_model_id: "gpt-5.6-sol".to_owned(),
        api_key_configured: true,
        sort_order: Some(1),
        offerings: vec![
            UserModelChannelOffering {
                vendor_code: "openai".to_owned(),
                vendor_name: "OpenAI".to_owned(),
                models: vec![
                    UserModelChannelModel {
                        model_id: "gpt-5.6-sol".to_owned(),
                        display_name: "GPT-5.6 Sol".to_owned(),
                        context_tokens: Some(1_050_000),
                        max_output_tokens: Some(128_000),
                        tool_call_rounds: Some(32),
                        supports_multimodal: true,
                    },
                    UserModelChannelModel {
                        model_id: "my-custom-model".to_owned(),
                        display_name: "My Custom".to_owned(),
                        context_tokens: None,
                        max_output_tokens: None,
                        tool_call_rounds: None,
                        supports_multimodal: false,
                    },
                ],
            },
            UserModelChannelOffering {
                vendor_code: "anthropic".to_owned(),
                vendor_name: "Anthropic".to_owned(),
                models: vec![UserModelChannelModel {
                    model_id: "claude-opus-5".to_owned(),
                    display_name: "Claude Opus 5".to_owned(),
                    context_tokens: Some(1_000_000),
                    max_output_tokens: Some(128_000),
                    tool_call_rounds: Some(24),
                    supports_multimodal: true,
                }],
            },
        ],
    }
}

#[tokio::test]
async fn initialize_schema_creates_tables_idempotently() {
    let store = test_store().await;
    store.initialize_schema().await.expect("re-initialize schema");
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
    )
    .fetch_all(store.pool())
    .await
    .expect("list tables");
    for expected in [
        "user_model_channel",
        "user_model_channel_offering",
        "user_model_channel_model",
        "user_model_key",
        "user_model_engine_config",
        "user_model_engine_selection",
    ] {
        assert!(tables.contains(&expected.to_owned()), "missing {expected}: {tables:?}");
    }
}

#[tokio::test]
async fn channel_upsert_round_trip_preserves_offerings_and_models() {
    let store = test_store().await;
    store
        .upsert_channel(&sample_channel("model-access.relay.team"))
        .await
        .expect("upsert channel");
    let channel = store
        .get_channel("model-access.relay.team")
        .await
        .expect("get channel")
        .expect("channel exists");
    assert_eq!(channel.name, "Team Relay");
    assert_eq!(channel.kind, "relay");
    assert_eq!(channel.base_url, "https://relay.example.com/v1");
    assert_eq!(channel.default_model_id, "gpt-5.6-sol");
    assert!(channel.api_key_configured);
    assert_eq!(channel.offerings.len(), 2);
    assert_eq!(channel.offerings[0].vendor_code, "openai");
    assert_eq!(channel.offerings[0].models.len(), 2);
    assert_eq!(channel.offerings[0].models[1].model_id, "my-custom-model");
    assert_eq!(channel.offerings[1].models[0].model_id, "claude-opus-5");
    assert_eq!(
        channel.offerings[1].models[0].context_tokens,
        Some(1_000_000)
    );
}

#[tokio::test]
async fn channel_upsert_replaces_offerings_and_models() {
    let store = test_store().await;
    let mut channel = sample_channel("model-access.relay.team");
    store.upsert_channel(&channel).await.expect("upsert first");
    channel.offerings = vec![UserModelChannelOffering {
        vendor_code: "deepseek".to_owned(),
        vendor_name: "DeepSeek".to_owned(),
        models: vec![UserModelChannelModel {
            model_id: "deepseek-v4-pro".to_owned(),
            display_name: "DeepSeek V4 Pro".to_owned(),
            context_tokens: None,
            max_output_tokens: None,
            tool_call_rounds: None,
            supports_multimodal: false,
        }],
    }];
    store.upsert_channel(&channel).await.expect("upsert second");
    let reloaded = store
        .get_channel("model-access.relay.team")
        .await
        .expect("get channel")
        .expect("channel exists");
    assert_eq!(reloaded.offerings.len(), 1);
    assert_eq!(reloaded.offerings[0].vendor_code, "deepseek");
    assert_eq!(reloaded.offerings[0].models.len(), 1);
}

#[tokio::test]
async fn api_key_round_trip_and_cascade_delete() {
    let store = test_store().await;
    store
        .upsert_channel(&sample_channel("model-access.relay.team"))
        .await
        .expect("upsert channel");
    store
        .upsert_api_key(&UserModelApiKey {
            channel_code: "model-access.relay.team".to_owned(),
            api_key: "sk-local-secret".to_owned(),
        })
        .await
        .expect("upsert key");
    assert_eq!(
        store
            .get_api_key("model-access.relay.team")
            .await
            .expect("get key"),
        Some("sk-local-secret".to_owned())
    );
    // Replacing the key updates in place.
    store
        .upsert_api_key(&UserModelApiKey {
            channel_code: "model-access.relay.team".to_owned(),
            api_key: "sk-rotated".to_owned(),
        })
        .await
        .expect("rotate key");
    assert_eq!(
        store
            .get_api_key("model-access.relay.team")
            .await
            .expect("get key"),
        Some("sk-rotated".to_owned())
    );
    // Deleting the channel cascades to the key.
    store
        .delete_channel("model-access.relay.team")
        .await
        .expect("delete channel");
    assert_eq!(
        store
            .get_api_key("model-access.relay.team")
            .await
            .expect("get key"),
        None
    );
}

#[tokio::test]
async fn engine_configs_and_selections_are_per_tool() {
    let store = test_store().await;
    store
        .upsert_channel(&sample_channel("model-access.relay.team"))
        .await
        .expect("upsert channel");
    store
        .upsert_engine_config(&UserModelEngineConfig {
            engine_id: "codex".to_owned(),
            channel_code: "model-access.relay.team".to_owned(),
            vendor_code: "openai".to_owned(),
            base_url: "https://relay.example.com/v1".to_owned(),
            default_model_id: "gpt-5.6-sol".to_owned(),
            supported_model_ids: vec!["gpt-5.6-sol".to_owned(), "claude-opus-5".to_owned()],
            supported_provider_ids: vec!["codex".to_owned(), "claude-code".to_owned()],
            input_context_tokens: Some(1_050_000),
            output_context_tokens: Some(128_000),
            tool_call_rounds: Some(32),
            supports_multimodal: true,
            api_key_configured: true,
            applied_at: "2026-08-02T00:00:00Z".to_owned(),
        })
        .await
        .expect("upsert codex config");
    store
        .upsert_engine_config(&UserModelEngineConfig {
            engine_id: "claude-code".to_owned(),
            channel_code: "model-access.relay.team".to_owned(),
            vendor_code: "anthropic".to_owned(),
            base_url: "https://relay.example.com/v1".to_owned(),
            default_model_id: "claude-opus-5".to_owned(),
            supported_model_ids: vec!["claude-opus-5".to_owned()],
            supported_provider_ids: vec!["claude-code".to_owned()],
            input_context_tokens: None,
            output_context_tokens: None,
            tool_call_rounds: None,
            supports_multimodal: false,
            api_key_configured: false,
            applied_at: "2026-08-02T00:01:00Z".to_owned(),
        })
        .await
        .expect("upsert claude config");

    let codex_only = store
        .list_engine_configs(Some("codex"))
        .await
        .expect("list codex configs");
    assert_eq!(codex_only.len(), 1);
    assert_eq!(codex_only[0].engine_id, "codex");
    assert_eq!(codex_only[0].supported_model_ids.len(), 2);

    let all = store
        .list_engine_configs(None)
        .await
        .expect("list all configs");
    assert_eq!(all.len(), 2);

    store
        .upsert_engine_selection(&UserModelEngineSelection {
            engine_id: "codex".to_owned(),
            channel_code: "model-access.relay.team".to_owned(),
            model_id: "gpt-5.6-sol".to_owned(),
        })
        .await
        .expect("upsert selection");
    let selection = store
        .get_engine_selection("codex")
        .await
        .expect("get selection")
        .expect("selection exists");
    assert_eq!(selection.model_id, "gpt-5.6-sol");

    // Deleting the channel cascades to engine configs and selections.
    store
        .delete_channel("model-access.relay.team")
        .await
        .expect("delete channel");
    assert_eq!(
        store
            .list_engine_configs(None)
            .await
            .expect("list after delete")
            .len(),
        0
    );
    assert_eq!(
        store
            .get_engine_selection("codex")
            .await
            .expect("selection after delete"),
        None
    );
}

#[tokio::test]
async fn multiple_channels_list_independently() {
    let store = test_store().await;
    store
        .upsert_channel(&sample_channel("model-access.relay.team"))
        .await
        .expect("upsert relay");
    let mut custom = sample_channel("model-access.custom.local");
    custom.kind = "custom".to_owned();
    custom.name = "Local Custom".to_owned();
    store.upsert_channel(&custom).await.expect("upsert custom");
    let channels = store.list_channels().await.expect("list channels");
    assert_eq!(channels.len(), 2);
    let kinds = channels.iter().map(|c| c.kind.as_str()).collect::<Vec<_>>();
    assert!(kinds.contains(&"relay"));
    assert!(kinds.contains(&"custom"));
}
