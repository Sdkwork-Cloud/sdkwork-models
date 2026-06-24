> Migrated from `docs/converter-naming-standard.md` on 2026-06-24.
> Owner: SDKWork maintainers

## 命名规范

### 格式

```
<SOURCE_PROTOCOL>_TO_<TARGET_PROTOCOL>
```

### 协议代码表

| 代码 | 协议全称 | 说明 |
|------|----------|------|
| OPENAI_RESPONSES | OpenAI Responses API | /v1/responses |
| OPENAI_COMPLETIONS | OpenAI Chat Completions | /v1/chat/completions |
| ANTHROPIC_MESSAGES | Anthropic Messages API | /v1/messages |
| GOOGLE_GEMINI | Google Gemini API | /v1/models/{model}:generateContent |
| OPENAI_COMPATIBLE | OpenAI兼容格式 | 通用兼容层 |

### 标准命名示例

| 转换器名称 | 源协议 | 目标协议 | 说明 |
|------------|--------|----------|------|
| OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES | OpenAI Responses | Anthropic Messages | Codex → Claude Code |
| ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES | Anthropic Messages | OpenAI Responses | Claude Code → Codex |
| OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES | OpenAI Completions | Anthropic Messages | DeepSeek → Claude Code |
| ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS | Anthropic Messages | OpenAI Completions | Claude Code → DeepSeek |
| OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES | OpenAI Completions | OpenAI Responses | 通用OpenAI → Codex |
| OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS | OpenAI Responses | OpenAI Completions | Codex → 通用OpenAI |
| GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS | Google Gemini | OpenAI Completions | Gemini → 通用OpenAI |
| OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI | OpenAI Completions | Google Gemini | 通用OpenAI → Gemini |
| GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES | Google Gemini | Anthropic Messages | Gemini → Claude Code |
| ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI | Anthropic Messages | Google Gemini | Claude Code → Gemini |

## 目录结构

```
converters/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── traits.rs                    # 核心trait定义
│   ├── types.rs                     # 类型定义
│   ├── error.rs                     # 错误处理
│   ├── registry.rs                  # 转换器注册表
│   ├── openai_responses_to_anthropic_messages/
│   │   ├── mod.rs
│   │   ├── converter.rs
│   │   ├── request.rs
│   │   ├── response.rs
│   │   └── tests.rs
│   ├── anthropic_messages_to_openai_responses/
│   │   ├── mod.rs
│   │   ├── converter.rs
│   │   ├── request.rs
│   │   ├── response.rs
│   │   └── tests.rs
│   ├── openai_completions_to_anthropic_messages/
│   │   ├── mod.rs
│   │   ├── converter.rs
│   │   ├── request.rs
│   │   ├── response.rs
│   │   └── tests.rs
│   └── ... (其他转换器)
├── tests/
│   ├── integration_tests.rs
│   └── common/
└── benches/
    └── converter_benchmarks.rs
```

## 核心Trait定义

### traits.rs

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::ConverterError;
use crate::types::*;

/// 转换器核心trait
#[async_trait]
pub trait Converter: Send + Sync {
    /// 转换器名称（遵循命名规范）
    fn name(&self) -> &str;
    
    /// 源协议
    fn source_protocol(&self) -> Protocol;
    
    /// 目标协议
    fn target_protocol(&self) -> Protocol;
    
    /// 支持的能力
    fn capabilities(&self) -> Vec<Capability>;
    
    /// 是否支持该转换
    fn can_convert(&self, source: &Protocol, target: &Protocol) -> bool {
        self.source_protocol() == *source && self.target_protocol() == *target
    }
    
    /// 转换请求
    async fn convert_request(&self, request: ConversionRequest) -> Result<ConversionRequest, ConverterError>;
    
    /// 转换响应
    async fn convert_response(&self, response: ConversionResponse) -> Result<ConversionResponse, ConverterError>;
    
    /// 转换流式响应
    async fn convert_stream(&self, stream: ConversionStream) -> Result<ConversionStream, ConverterError> {
        Err(ConverterError::UnsupportedOperation("streaming".to_string()))
    }
}

/// 映射器trait
pub trait Mapper: Send + Sync {
    /// 映射器名称
    fn name(&self) -> &str;
    
    /// 映射单个模型
    fn map(&self, source_model: &str, mapping: &ModelMapping) -> Result<String, ConverterError>;
    
    /// 批量映射模型
    fn map_batch(&self, models: &[String], mapping: &ModelMapping) -> Result<Vec<String>, ConverterError> {
        models.iter().map(|m| self.map(m, mapping)).collect()
    }
    
    /// 反向映射
    fn reverse_map(&self, target_model: &str, mapping: &ModelMapping) -> Result<String, ConverterError>;
}

/// 转换器工厂trait
pub trait ConverterFactory: Send + Sync {
    /// 创建转换器
    fn create(&self, config: &ConverterConfig) -> Result<Box<dyn Converter>, ConverterError>;
    
    /// 支持的协议对
    fn supported_pairs(&self) -> Vec<(Protocol, Protocol)>;
}
```

### types.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 协议枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    #[serde(rename = "openai_responses")]
    OpenAiResponses,
    #[serde(rename = "openai_completions")]
    OpenAiCompletions,
    #[serde(rename = "anthropic_messages")]
    AnthropicMessages,
    #[serde(rename = "google_gemini")]
    GoogleGemini,
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
    #[serde(rename = "vendor_native")]
    VendorNative,
}

/// 能力枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "stream")]
    Stream,
    #[serde(rename = "tools")]
    Tools,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "music")]
    Music,
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "reasoning")]
    Reasoning,
}

/// 转换请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    pub protocol: Protocol,
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stream: bool,
    pub tools: Option<Vec<Tool>>,
    pub system: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
}

/// 角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

/// 内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// 内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: serde_json::Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool_call_id: String, content: String },
}

/// 图片URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: Option<String>,
}

/// 工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function,
}

/// 函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

/// 转换响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResponse {
    pub protocol: Protocol,
    pub id: String,
    pub model: String,
    pub content: Vec<ContentPart>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 转换流
pub type ConversionStream = tokio::sync::mpsc::Receiver<ConversionResponse>;

/// 模型映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    pub mapping: HashMap<String, String>,
    pub wildcard_rules: Option<Vec<WildcardRule>>,
}

/// 通配符规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WildcardRule {
    pub pattern: String,
    pub target: String,
}

/// 转换器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConverterConfig {
    pub name: String,
    pub source_protocol: Protocol,
    pub target_protocol: Protocol,
    pub model_mapping: ModelMapping,
    pub capabilities: Vec<Capability>,
    pub options: HashMap<String, serde_json::Value>,
}
```

### error.rs

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConverterError {
    #[error("Unsupported conversion: {source} -> {target}")]
    UnsupportedConversion { source: String, target: String },
    
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Model mapping not found: {0}")]
    ModelMappingNotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}
```

## 转换器实现示例

### openai_responses_to_anthropic_messages/converter.rs

```rust
use async_trait::async_trait;
use crate::error::ConverterError;
use crate::traits::Converter;
use crate::types::*;
use super::request::convert_request;
use super::response::convert_response;

/// OpenAI Responses → Anthropic Messages 转换器
pub struct OpenAiResponsesToAnthropicMessagesConverter {
    model_mapping: ModelMapping,
}

impl OpenAiResponsesToAnthropicMessagesConverter {
    pub fn new(model_mapping: ModelMapping) -> Self {
        Self { model_mapping }
    }
}

#[async_trait]
impl Converter for OpenAiResponsesToAnthropicMessagesConverter {
    fn name(&self) -> &str {
        "OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES"
    }
    
    fn source_protocol(&self) -> Protocol {
        Protocol::OpenAiResponses
    }
    
    fn target_protocol(&self) -> Protocol {
        Protocol::AnthropicMessages
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Stream,
            Capability::Tools,
            Capability::Vision,
        ]
    }
    
    async fn convert_request(&self, request: ConversionRequest) -> Result<ConversionRequest, ConverterError> {
        convert_request(request, &self.model_mapping)
    }
    
    async fn convert_response(&self, response: ConversionResponse) -> Result<ConversionResponse, ConverterError> {
        convert_response(response, &self.model_mapping)
    }
    
    async fn convert_stream(&self, stream: ConversionStream) -> Result<ConversionStream, ConverterError> {
        // 实现流式转换逻辑
        todo!()
    }
}
```

### openai_responses_to_anthropic_messages/request.rs

```rust
use crate::error::ConverterError;
use crate::types::*;
use std::collections::HashMap;

/// 转换OpenAI请求为Anthropic格式
pub fn convert_request(
    request: ConversionRequest,
    model_mapping: &ModelMapping,
) -> Result<ConversionRequest, ConverterError> {
    // 映射模型名称
    let model = map_model(&request.model, model_mapping)?;
    
    // 提取系统消息
    let system = extract_system_message(&request.messages);
    
    // 转换消息格式
    let messages = convert_messages(&request.messages)?;
    
    // 转换工具格式
    let tools = request.tools.map(|t| convert_tools(&t)).transpose()?;
    
    Ok(ConversionRequest {
        protocol: Protocol::AnthropicMessages,
        model,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: request.stream,
        tools,
        system,
        metadata: request.metadata,
    })
}

fn map_model(model: &str, mapping: &ModelMapping) -> Result<String, ConverterError> {
    // 精确匹配
    if let Some(target) = mapping.mapping.get(model) {
        return Ok(target.clone());
    }
    
    // 通配符匹配
    if let Some(rules) = &mapping.wildcard_rules {
        for rule in rules {
            if matches_pattern(model, &rule.pattern) {
                return Ok(rule.target.replace("*", model));
            }
        }
    }
    
    // 默认返回原模型名
    Ok(model.to_string())
}

fn matches_pattern(model: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let regex_pattern = pattern.replace('*', ".*");
        regex::Regex::new(&format!("^{}$", regex_pattern))
            .map(|re| re.is_match(model))
            .unwrap_or(false)
    } else {
        model == pattern
    }
}

fn extract_system_message(messages: &[Message]) -> Option<String> {
    messages
        .iter()
        .find(|m| m.role == Role::System)
        .and_then(|m| match &m.content {
            Content::Text(text) => Some(text.clone()),
            Content::Parts(parts) => {
                let text: String = parts
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");
                if text.is_empty() { None } else { Some(text) }
            }
        })
}

fn convert_messages(messages: &[Message]) -> Result<Vec<Message>, ConverterError> {
    messages
        .iter()
        .filter(|m| m.role != Role::System)
        .map(|m| {
            Ok(Message {
                role: m.role.clone(),
                content: convert_content(&m.content)?,
            })
        })
        .collect()
}

fn convert_content(content: &Content) -> Result<Content, ConverterError> {
    match content {
        Content::Text(text) => Ok(Content::Text(text.clone())),
        Content::Parts(parts) => {
            let converted: Vec<ContentPart> = parts
                .iter()
                .map(convert_content_part)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Content::Parts(converted))
        }
    }
}

fn convert_content_part(part: &ContentPart) -> Result<ContentPart, ConverterError> {
    match part {
        ContentPart::Text { text } => Ok(ContentPart::Text { text: text.clone() }),
        ContentPart::ImageUrl { image_url } => Ok(ContentPart::ImageUrl {
            image_url: image_url.clone(),
        }),
        ContentPart::ToolUse { id, name, input } => Ok(ContentPart::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        }),
        ContentPart::ToolResult {
            tool_call_id,
            content,
        } => Ok(ContentPart::ToolResult {
            tool_call_id: tool_call_id.clone(),
            content: content.clone(),
        }),
    }
}

fn convert_tools(tools: &[Tool]) -> Result<Vec<Tool>, ConverterError> {
    // Anthropic和OpenAI的工具格式基本相同
    Ok(tools.to_vec())
}
```

### openai_responses_to_anthropic_messages/response.rs

```rust
use crate::error::ConverterError;
use crate::types::*;
use std::collections::HashMap;

/// 转换Anthropic响应为OpenAI格式
pub fn convert_response(
    response: ConversionResponse,
    model_mapping: &ModelMapping,
) -> Result<ConversionResponse, ConverterError> {
    // 反向映射模型名称
    let model = reverse_map_model(&response.model, model_mapping)?;
    
    // 转换内容格式
    let content = convert_content(&response.content)?;
    
    Ok(ConversionResponse {
        protocol: Protocol::OpenAiResponses,
        id: response.id,
        model,
        content,
        stop_reason: response.stop_reason,
        usage: response.usage,
        metadata: response.metadata,
    })
}

fn reverse_map_model(model: &str, mapping: &ModelMapping) -> Result<String, ConverterError> {
    // 创建反向映射
    let reverse_mapping: HashMap<&str, &str> = mapping
        .mapping
        .iter()
        .map(|(k, v)| (v.as_str(), k.as_str()))
        .collect();
    
    if let Some(target) = reverse_mapping.get(model) {
        return Ok(target.to_string());
    }
    
    Ok(model.to_string())
}

fn convert_content(content: &[ContentPart]) -> Result<Vec<ContentPart>, ConverterError> {
    content
        .iter()
        .map(convert_content_part)
        .collect()
}

fn convert_content_part(part: &ContentPart) -> Result<ContentPart, ConverterError> {
    match part {
        ContentPart::Text { text } => Ok(ContentPart::Text { text: text.clone() }),
        ContentPart::ImageUrl { image_url } => Ok(ContentPart::ImageUrl {
            image_url: image_url.clone(),
        }),
        ContentPart::ToolUse { id, name, input } => Ok(ContentPart::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        }),
        ContentPart::ToolResult {
            tool_call_id,
            content,
        } => Ok(ContentPart::ToolResult {
            tool_call_id: tool_call_id.clone(),
            content: content.clone(),
        }),
    }
}
```

## Vendor插件配置

### alibaba/plugin.json

```json
{
  "name": "alibaba-claude-code",
  "version": "1.0.0",
  "vendorCode": "alibaba",
  "apiCode": "claude_code",
  "converter": "ANTHROPIC_MESSAGES_TO_ANTHROPIC_MESSAGES",
  "config": {
    "sourceProtocol": "anthropic_messages",
    "targetProtocol": "anthropic_messages",
    "modelMapping": {
      "qwen3.7-max": "claude-sonnet-4",
      "qwen3.7-turbo": "claude-haiku-4"
    },
    "capabilities": ["stream", "tools"],
    "regions": ["global"]
  }
}
```

### deepseek/plugin.json

```json
{
  "name": "deepseek-claude-code",
  "version": "1.0.0",
  "vendorCode": "deepseek",
  "apiCode": "claude_code",
  "converter": "OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES",
  "config": {
    "sourceProtocol": "openai_compatible",
    "targetProtocol": "anthropic_messages",
    "modelMapping": {
      "deepseek-v4-pro": "claude-sonnet-4",
      "deepseek-v4-flash": "claude-haiku-4"
    },
    "capabilities": ["stream", "tools", "code"],
    "regions": ["cn", "global"]
  }
}
```

## 测试规范

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_convert_request() {
        let mapping = ModelMapping {
            mapping: vec![
                ("gpt-4".to_string(), "claude-sonnet-4".to_string()),
            ].into_iter().collect(),
            wildcard_rules: None,
        };
        
        let converter = OpenAiResponsesToAnthropicMessagesConverter::new(mapping);
        
        let request = ConversionRequest {
            protocol: Protocol::OpenAiResponses,
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: Role::User,
                    content: Content::Text("Hello!".to_string()),
                },
            ],
            max_tokens: Some(1024),
            temperature: Some(0.7),
            top_p: None,
            stream: false,
            tools: None,
            system: None,
            metadata: HashMap::new(),
        };
        
        let result = converter.convert_request(request).await.unwrap();
        
        assert_eq!(result.model, "claude-sonnet-4");
        assert_eq!(result.protocol, Protocol::AnthropicMessages);
    }
}
```

## 部署

### Cargo.toml

```toml
[package]
name = "sdkwork-converters"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
thiserror = "1.0"
regex = "1.0"
reqwest = { version = "0.11", features = ["json"] }

[dev-dependencies]
tokio-test = "0.4"
```

### 构建

```bash
# 构建库
cargo build --release

# 运行测试
cargo test

# 运行基准测试
cargo bench

# 生成文档
cargo doc --open
```

