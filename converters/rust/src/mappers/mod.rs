use crate::error::ConverterError;
use crate::types::ModelMapping;

/// 模型映射器trait
pub trait Mapper: Send + Sync {
    fn name(&self) -> &str;

    fn map(&self, source_model: &str, mapping: &ModelMapping) -> Result<String, ConverterError>;

    fn reverse_map(
        &self,
        target_model: &str,
        mapping: &ModelMapping,
    ) -> Result<String, ConverterError>;

    fn map_batch(
        &self,
        models: &[String],
        mapping: &ModelMapping,
    ) -> Result<Vec<String>, ConverterError> {
        models.iter().map(|m| self.map(m, mapping)).collect()
    }
}

/// 标准模型映射器
pub struct ModelMapper;

impl ModelMapper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModelMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl Mapper for ModelMapper {
    fn name(&self) -> &str {
        "model_mapper"
    }

    fn map(&self, source_model: &str, mapping: &ModelMapping) -> Result<String, ConverterError> {
        Ok(mapping.resolve(source_model))
    }

    fn reverse_map(
        &self,
        target_model: &str,
        mapping: &ModelMapping,
    ) -> Result<String, ConverterError> {
        Ok(mapping.reverse_resolve(target_model))
    }
}

/// 前缀映射器
pub struct PrefixMapper {
    prefix: String,
}

impl PrefixMapper {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Mapper for PrefixMapper {
    fn name(&self) -> &str {
        "prefix_mapper"
    }

    fn map(&self, source_model: &str, _mapping: &ModelMapping) -> Result<String, ConverterError> {
        if source_model.starts_with(&self.prefix) {
            Ok(source_model.to_string())
        } else {
            Ok(format!("{}{}", self.prefix, source_model))
        }
    }

    fn reverse_map(
        &self,
        target_model: &str,
        _mapping: &ModelMapping,
    ) -> Result<String, ConverterError> {
        if target_model.starts_with(&self.prefix) {
            Ok(target_model[self.prefix.len()..].to_string())
        } else {
            Ok(target_model.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_mapper_direct() {
        let mapper = ModelMapper::new();
        let mut mapping = ModelMapping::default();
        mapping
            .mapping
            .insert("qwen3.7-max".to_string(), "claude-sonnet-4".to_string());

        assert_eq!(mapper.map("qwen3.7-max", &mapping).unwrap(), "claude-sonnet-4");
        assert_eq!(mapper.map("unknown", &mapping).unwrap(), "unknown");
    }

    #[test]
    fn test_model_mapper_reverse() {
        let mapper = ModelMapper::new();
        let mut mapping = ModelMapping::default();
        mapping
            .mapping
            .insert("qwen3.7-max".to_string(), "claude-sonnet-4".to_string());

        assert_eq!(
            mapper.reverse_map("claude-sonnet-4", &mapping).unwrap(),
            "qwen3.7-max"
        );
    }

    #[test]
    fn test_prefix_mapper() {
        let mapper = PrefixMapper::new("deepseek-");
        let mapping = ModelMapping::default();

        assert_eq!(mapper.map("v4-pro", &mapping).unwrap(), "deepseek-v4-pro");
        assert_eq!(
            mapper.map("deepseek-v4-pro", &mapping).unwrap(),
            "deepseek-v4-pro"
        );
        assert_eq!(
            mapper.reverse_map("deepseek-v4-pro", &mapping).unwrap(),
            "v4-pro"
        );
    }
}
