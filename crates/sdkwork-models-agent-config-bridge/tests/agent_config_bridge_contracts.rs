//! Integration contract: the bridge pushes client-local engine configs to
//! the agents Config SPI surface and writes back the applied state.

use sdkwork_models_agent_config_bridge::{
    push_engine_configuration, push_engine_selection, ModelConfigBridgeClient, PushOutcome,
};
use sdkwork_models_user_config_repository_sqlx::sqlite_store::SqliteUserModelConfigStore;
use sdkwork_models_user_config_repository_sqlx::{
    UserModelApiKey, UserModelChannel, UserModelConfigStore, UserModelEngineConfig,
    UserModelEngineSelection,
};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

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

async fn seed_engine_config(
    store: &SqliteUserModelConfigStore,
    engine_id: &str,
    channel_code: &str,
) -> UserModelEngineConfig {
    let channel = UserModelChannel {
        code: channel_code.to_string(),
        name: "Team Relay".to_string(),
        kind: "relay".to_string(),
        base_url: "https://relay.example.com/v1".to_string(),
        description: "team gateway".to_string(),
        default_vendor_code: "openai".to_string(),
        default_model_id: "gpt-5.6-sol".to_string(),
        api_key_configured: true,
        sort_order: Some(1),
        offerings: Vec::new(),
    };
    store.upsert_channel(&channel).await.expect("upsert channel");
    store
        .upsert_api_key(&UserModelApiKey {
            channel_code: channel_code.to_string(),
            api_key: "relay-secret".to_string(),
        })
        .await
        .expect("upsert key");
    let config = UserModelEngineConfig {
        engine_id: engine_id.to_string(),
        channel_code: channel_code.to_string(),
        vendor_code: "openai".to_string(),
        base_url: "https://relay.example.com/v1".to_string(),
        default_model_id: "gpt-5.6-sol".to_string(),
        supported_model_ids: vec!["gpt-5.6-sol".to_string()],
        supported_provider_ids: vec![engine_id.to_string()],
        input_context_tokens: Some(1_050_000),
        output_context_tokens: Some(128_000),
        tool_call_rounds: Some(32),
        supports_multimodal: true,
        api_key_configured: false,
        applied_at: "1970-01-01T00:00:00Z".to_string(),
    };
    store
        .upsert_engine_config(&config)
        .await
        .expect("upsert engine config");
    config
}

/// Serves one HTTP request on a loopback listener and returns
/// `(method, path, request_body, response_body)`.
fn stub_agents_server(
    response_body: &'static str,
) -> (String, std::thread::JoinHandle<()>, Arc<std::sync::Mutex<Vec<u8>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub addr");
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_thread = captured.clone();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 16_384];
        let read = stream.read(&mut buffer).expect("read request");
        *captured_thread.lock().unwrap() = buffer[..read].to_vec();
        let body = response_body;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).expect("write response");
    });
    (addr.to_string(), handle, captured)
}

#[tokio::test]
async fn push_engine_configuration_sends_agents_dto_and_writes_back() {
    let store = test_store().await;
    let config = seed_engine_config(&store, "codex", "team-relay").await;
    let (addr, handle, captured) = stub_agents_server(
        r#"{"code":0,"data":{"item":{"profileId":"profile.model_configuration.abc","engineId":"codex","apiKeyConfigured":true}},"traceId":"trace-1"}"#,
    );
    let client = ModelConfigBridgeClient::new(format!("http://{addr}"));

    let outcome = push_engine_configuration(&store, &client, "codex")
        .await
        .expect("push succeeds");
    handle.join().expect("stub server joins");

    assert_eq!(
        outcome,
        PushOutcome::Applied {
            profile_id: "profile.model_configuration.abc".to_string()
        }
    );

    // The request body matches the agents Config SPI DTO exactly.
    let request = String::from_utf8(captured.lock().unwrap().clone()).expect("utf8");
    assert!(request.starts_with("POST /app/v3/api/ai/model_configurations/apply HTTP/1.1"));
    assert!(request.contains(r#""configurationId":"team-relay""#));
    assert!(request.contains(r#""engineId":"codex""#));
    assert!(request.contains(r#""apiKey":"relay-secret""#));
    assert!(request.contains(r#""inputContextTokens":"1050000""#));
    assert!(request.contains(r#""toolCallRounds":"32""#));

    // The store record reflects the successful push.
    let reloaded = store
        .list_engine_configs(Some("codex"))
        .await
        .expect("reload")
        .into_iter()
        .next()
        .expect("config exists");
    assert!(reloaded.api_key_configured);
    assert_ne!(reloaded.applied_at, config.applied_at);
}

#[tokio::test]
async fn push_engine_selection_sends_agents_dto() {
    let store = test_store().await;
    seed_engine_config(&store, "codex", "team-relay").await;
    store
        .upsert_engine_selection(&UserModelEngineSelection {
            engine_id: "codex".to_string(),
            channel_code: "team-relay".to_string(),
            model_id: "gpt-5.6-reasoning".to_string(),
        })
        .await
        .expect("upsert selection");
    let (addr, handle, captured) = stub_agents_server(
        r#"{"code":0,"data":{"item":{"profileId":"profile.model_configuration.abc","engineId":"codex","modelId":"gpt-5.6-reasoning"}},"traceId":"trace-2"}"#,
    );
    let client = ModelConfigBridgeClient::new(format!("http://{addr}"));

    let outcome = push_engine_selection(&store, &client, "codex")
        .await
        .expect("push selection succeeds");
    handle.join().expect("stub server joins");

    assert_eq!(
        outcome,
        PushOutcome::Applied {
            profile_id: "profile.model_configuration.abc".to_string()
        }
    );
    let request = String::from_utf8(captured.lock().unwrap().clone()).expect("utf8");
    assert!(request.starts_with("POST /app/v3/api/ai/model_selections/apply HTTP/1.1"));
    assert!(request.contains(r#""configurationId":"team-relay""#));
    assert!(request.contains(r#""modelId":"gpt-5.6-reasoning""#));
}

#[tokio::test]
async fn push_without_stored_config_reports_nothing_to_push() {
    let store = test_store().await;
    let (addr, _handle, _captured) = stub_agents_server(r#"{"code":0,"data":{"item":{}}}"#);
    let client = ModelConfigBridgeClient::new(format!("http://{addr}"));

    let outcome = push_engine_configuration(&store, &client, "hermes")
        .await
        .expect("no-op push succeeds");
    assert_eq!(outcome, PushOutcome::NothingToPush);
}

#[tokio::test]
async fn agents_rejection_surfaces_the_response_problem() {
    let store = test_store().await;
    seed_engine_config(&store, "codex", "team-relay").await;
    // A 400 problem+json from the agents surface must propagate as a push error.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub addr");
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 16_384];
        let _ = stream.read(&mut buffer).expect("read request");
        let body = r#"{"code":400,"message":"apiKey is required"}"#;
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/problem+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).expect("write response");
    });
    let client = ModelConfigBridgeClient::new(format!("http://{addr}"));

    let error = push_engine_configuration(&store, &client, "codex")
        .await
        .expect_err("agents rejection must fail the push");
    handle.join().expect("stub server joins");
    let message = error.to_string();
    assert!(message.contains("400"), "message: {message}");
    assert!(message.contains("apiKey is required"), "message: {message}");
}
