use std::sync::OnceLock;

use sdkwork_database_id::SnowflakeIdGenerator;

use sdkwork_models_contract_service::{DomainError, DomainResult};

const DEFAULT_MODELS_RUNTIME_NODE_ID: u16 = 23;
const MODELS_RUNTIME_NODE_ID_ENV: &str = "SDKWORK_MODELS_SNOWFLAKE_NODE_ID";

static MODELS_RUNTIME_ID_GENERATOR: OnceLock<Result<SnowflakeIdGenerator, String>> =
    OnceLock::new();

pub(crate) fn next_claw_runtime_id(context: &str) -> DomainResult<i64> {
    let generator = models_runtime_id_generator()?;
    generator
        .generate()
        .map_err(|error| DomainError::new(format!("failed to generate {context} id: {error:?}")))
}

fn models_runtime_id_generator() -> DomainResult<&'static SnowflakeIdGenerator> {
    match MODELS_RUNTIME_ID_GENERATOR.get_or_init(build_models_runtime_id_generator) {
        Ok(generator) => Ok(generator),
        Err(message) => Err(DomainError::new(message.clone())),
    }
}

fn build_models_runtime_id_generator() -> Result<SnowflakeIdGenerator, String> {
    let node_id = match std::env::var(MODELS_RUNTIME_NODE_ID_ENV) {
        Ok(value) if !value.trim().is_empty() => value.trim().parse::<u16>().map_err(|_| {
            format!("{MODELS_RUNTIME_NODE_ID_ENV} must be an integer between 0 and 1023")
        })?,
        Ok(_) => {
            return Err(format!(
                "{MODELS_RUNTIME_NODE_ID_ENV} must be an integer between 0 and 1023"
            ));
        }
        Err(_) => DEFAULT_MODELS_RUNTIME_NODE_ID,
    };

    SnowflakeIdGenerator::new(node_id).map_err(|error| {
        format!("{MODELS_RUNTIME_NODE_ID_ENV} is invalid for models runtime IDs: {error:?}")
    })
}
