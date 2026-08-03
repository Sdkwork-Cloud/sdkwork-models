//! Store-to-agents push orchestration for the client-local model
//! configuration bridge.

use sdkwork_models_user_config_repository_sqlx::{
    UserModelConfigStore, UserModelConfigStoreError, UserModelEngineConfig,
};

use crate::client::{
    ApplyModelConfigurationError, ApplyModelConfigurationRequest, ModelConfigBridgeClient,
    ModelSelectionApplyRequest,
};

/// Outcome of a single engine push.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushOutcome {
    /// The engine has no stored configuration/selection to push.
    NothingToPush,
    /// Applied on the agents side; the store record was updated.
    Applied { profile_id: String },
}

/// Errors raised while orchestrating a store → agents push.
#[derive(Debug)]
pub enum PushError {
    Store(UserModelConfigStoreError),
    Apply(ApplyModelConfigurationError),
}

impl std::fmt::Display for PushError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "client-local model config store: {error}"),
            Self::Apply(error) => write!(formatter, "agents Config SPI: {error}"),
        }
    }
}

impl std::error::Error for PushError {}

impl From<UserModelConfigStoreError> for PushError {
    fn from(error: UserModelConfigStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ApplyModelConfigurationError> for PushError {
    fn from(error: ApplyModelConfigurationError) -> Self {
        Self::Apply(error)
    }
}

/// Pushes the stored engine configuration (channel + model + capability
/// metadata) to the agents Config SPI and writes back the applied state.
///
/// The channel code is used as the configuration id, so re-pushing the same
/// channel converges on the same agents profile (the agents side persists
/// the profile and the credential in its secret surface).
pub async fn push_engine_configuration<S: UserModelConfigStore>(
    store: &S,
    client: &ModelConfigBridgeClient,
    engine_id: &str,
) -> Result<PushOutcome, PushError> {
    let configs = store.list_engine_configs(Some(engine_id)).await?;
    let Some(config) = configs.first() else {
        return Ok(PushOutcome::NothingToPush);
    };
    let api_key = store.get_api_key(&config.channel_code).await?;
    let request = ApplyModelConfigurationRequest::from_engine_config(
        &config.channel_code,
        engine_id,
        config,
        api_key,
    );
    let applied = client.apply_configuration(&request).await?;
    write_back_applied(store, config).await?;
    Ok(PushOutcome::Applied {
        profile_id: applied.profile_id,
    })
}

/// Pushes the stored engine model selection to the agents Config SPI.
pub async fn push_engine_selection<S: UserModelConfigStore>(
    store: &S,
    client: &ModelConfigBridgeClient,
    engine_id: &str,
) -> Result<PushOutcome, PushError> {
    let Some(selection) = store.get_engine_selection(engine_id).await? else {
        return Ok(PushOutcome::NothingToPush);
    };
    let request = ModelSelectionApplyRequest {
        configuration_id: Some(selection.channel_code.clone()),
        engine_id: engine_id.to_string(),
        model_id: selection.model_id.clone(),
    };
    let applied = client.apply_selection(&request).await?;
    Ok(PushOutcome::Applied {
        profile_id: applied.profile_id,
    })
}

async fn write_back_applied<S: UserModelConfigStore>(
    store: &S,
    config: &UserModelEngineConfig,
) -> Result<(), PushError> {
    let mut updated = config.clone();
    updated.applied_at = now_rfc3339();
    updated.api_key_configured = true;
    store.upsert_engine_config(&updated).await?;
    Ok(())
}

fn now_rfc3339() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    // RFC3339-compatible UTC timestamp without an external time dependency.
    let days = seconds / 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    let seconds_of_day = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",
        hour = seconds_of_day / 3600,
        minute = (seconds_of_day % 3600) / 60,
        second = seconds_of_day % 60,
    )
}

/// Howard Hinnant's `civil_from_days` algorithm (days since epoch → date).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(20_666), (2026, 8, 1));
    }
}
