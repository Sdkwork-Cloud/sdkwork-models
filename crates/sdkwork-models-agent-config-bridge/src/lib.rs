//! Client-local model configuration bridge.
//!
//! The desktop client keeps the unified model management in its local SQLite
//! (`user_model_*` tables). This bridge pushes those engine configurations to
//! the sdkwork-agents Config SPI (app-api `model_configurations/apply` and
//! `model_selections/apply`) so the agent provider CLI configs are
//! materialized through the kernel runtime, and then writes back
//! `applied_at` + `api_key_configured` on the client-local record.
//!
//! The bridge is HTTP-only: it never depends on sdkwork-agents crates. API
//! keys are transported in the apply request exactly once (the established
//! credential boundary: the agents side stores them in its secret surface
//! and never persists them inside configuration profiles).

mod client;
mod sync;

pub use client::{
    AppliedModelConfiguration, ApplyModelConfigurationError, ApplyModelConfigurationRequest,
    ModelConfigBridgeClient, ModelSelectionApplyRequest, ModelSelectionApplyResponse,
};
pub use sync::{push_engine_configuration, push_engine_selection, PushError, PushOutcome};
