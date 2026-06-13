use async_trait::async_trait;

use crate::error::ConverterError;
use crate::types::*;

/// 转换器核心trait
#[async_trait]
pub trait Converter: Send + Sync {
    /// 转换器名称（遵循命名规范：SOURCE_PROTOCOL_TO_TARGET_PROTOCOL）
    fn name(&self) -> &str;

    /// 源协议
    fn source_protocol(&self) -> Protocol;

    /// 目标协议
    fn target_protocol(&self) -> Protocol;

    /// 支持的能力
    fn capabilities(&self) -> Vec<Capability>;

    /// 是否支持该转换
    fn can_convert(&self, source: &Protocol, target: &Protocol) -> bool {
        *source == self.source_protocol() && *target == self.target_protocol()
    }

    /// 转换请求（将目标协议格式转为源协议格式，或反向取决于实现）
    async fn convert_request(
        &self,
        request: ConversionRequest,
    ) -> Result<ConversionRequest, ConverterError>;

    /// 转换响应（将源协议格式转为目标协议格式）
    async fn convert_response(
        &self,
        response: ConversionResponse,
    ) -> Result<ConversionResponse, ConverterError>;
}
