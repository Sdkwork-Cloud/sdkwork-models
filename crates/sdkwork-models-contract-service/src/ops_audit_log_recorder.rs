use std::future::Future;
use std::pin::Pin;

use crate::DomainResult;

pub struct OpsAuditLogEntry<'a> {
    pub audit_log_uuid: &'a str,
    pub request_id: &'a str,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub operator_id: i64,
    pub operator_type: i32,
    pub action: &'static str,
    pub target_type: i32,
    pub target_id: i64,
    pub change_summary: serde_json::Value,
}

pub type OpsAuditLogFuture<'a> = Pin<Box<dyn Future<Output = DomainResult<()>> + Send + 'a>>;

pub trait OpsAuditLogRecorder: Send + Sync {
    fn record_postgres_audit_log<'a>(
        &'a self,
        tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>,
        entry: OpsAuditLogEntry<'a>,
    ) -> OpsAuditLogFuture<'a>;
}

pub struct NoopOpsAuditLogRecorder;

impl OpsAuditLogRecorder for NoopOpsAuditLogRecorder {
    fn record_postgres_audit_log<'a>(
        &'a self,
        _tx: &'a mut sqlx::Transaction<'a, sqlx::Postgres>,
        _entry: OpsAuditLogEntry<'a>,
    ) -> OpsAuditLogFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}
