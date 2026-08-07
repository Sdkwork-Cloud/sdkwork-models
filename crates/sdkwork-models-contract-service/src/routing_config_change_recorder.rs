use std::future::Future;
use std::pin::Pin;

use crate::DomainResult;

pub struct AiRoutingConfigChange<'a> {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub operator_id: i64,
    pub request_id: &'a str,
    pub requested_at: &'a str,
    pub changed_object_type: &'a str,
    pub changed_object_id: i64,
    pub action: &'a str,
    pub event_payload: serde_json::Value,
}

pub type AiRoutingConfigChangeFuture<'a> =
    Pin<Box<dyn Future<Output = DomainResult<i64>> + Send + 'a>>;

pub trait AiRoutingConfigChangeRecorder: Send + Sync {
    fn record_postgres_change<'a>(
        &'a self,
        tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>,
        change: AiRoutingConfigChange<'a>,
    ) -> AiRoutingConfigChangeFuture<'a>;
}

pub struct NoopAiRoutingConfigChangeRecorder;

impl AiRoutingConfigChangeRecorder for NoopAiRoutingConfigChangeRecorder {
    fn record_postgres_change<'a>(
        &'a self,
        _tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>,
        _change: AiRoutingConfigChange<'a>,
    ) -> AiRoutingConfigChangeFuture<'a> {
        Box::pin(async { Ok(0) })
    }
}
