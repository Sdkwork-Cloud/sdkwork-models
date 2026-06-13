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

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::OpenAiResponses => write!(f, "openai_responses"),
            Protocol::OpenAiCompletions => write!(f, "openai_completions"),
            Protocol::AnthropicMessages => write!(f, "anthropic_messages"),
            Protocol::GoogleGemini => write!(f, "google_gemini"),
            Protocol::OpenAiCompatible => write!(f, "openai_compatible"),
            Protocol::VendorNative => write!(f, "vendor_native"),
        }
    }
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
    pub system: Option<SystemPrompt>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ConversionRequest {
    fn default() -> Self {
        Self {
            protocol: Protocol::OpenAiCompatible,
            model: String::new(),
            messages: Vec::new(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: None,
            stream: false,
            tools: None,
            system: None,
            metadata: HashMap::new(),
        }
    }
}

/// 系统提示
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Parts(Vec<SystemContentPart>),
}

/// 系统内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "cache_control")]
    CacheControl { cache_control: CacheControl },
}

/// 缓存控制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String,
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
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
    },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
}

/// 图片URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: Option<String>,
}

/// 图片源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// 工具结果内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// 工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: Function,
}

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

/// 函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub input_schema: Option<serde_json::Value>,
}

/// 转换响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResponse {
    pub protocol: Protocol,
    pub id: String,
    pub model: String,
    pub content: Vec<ContentPart>,
    pub stop_reason: Option<StopReason>,
    pub usage: Usage,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 停止原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopReason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(rename = "stop_sequence")]
    StopSequence,
    #[serde(rename = "max_tokens")]
    MaxTokens,
    #[serde(rename = "tool_use")]
    ToolUse,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "content_filter")]
    ContentFilter,
}

impl StopReason {
    /// 映射到 OpenAI 停止原因
    pub fn to_openai(&self) -> StopReason {
        match self {
            StopReason::EndTurn | StopReason::StopSequence => StopReason::Stop,
            StopReason::MaxTokens => StopReason::Length,
            StopReason::ToolUse => StopReason::Stop,
            _ => self.clone(),
        }
    }

    /// 映射到 Anthropic 停止原因
    pub fn to_anthropic(&self) -> StopReason {
        match self {
            StopReason::Stop => StopReason::EndTurn,
            StopReason::Length => StopReason::MaxTokens,
            StopReason::ContentFilter => StopReason::StopSequence,
            _ => self.clone(),
        }
    }

    /// 映射到 Gemini 停止原因 (与 OpenAI 类似)
    pub fn to_gemini(&self) -> StopReason {
        self.to_openai()
    }

    /// 从 OpenAI 停止原因映射
    pub fn from_openai(&self) -> StopReason {
        self.to_anthropic()
    }
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopReason::EndTurn => write!(f, "end_turn"),
            StopReason::StopSequence => write!(f, "stop_sequence"),
            StopReason::MaxTokens => write!(f, "max_tokens"),
            StopReason::ToolUse => write!(f, "tool_use"),
            StopReason::Stop => write!(f, "stop"),
            StopReason::Length => write!(f, "length"),
            StopReason::ContentFilter => write!(f, "content_filter"),
        }
    }
}

/// Token使用情况
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

/// 模型映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    pub mapping: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wildcard_rules: Option<Vec<WildcardRule>>,
}

impl Default for ModelMapping {
    fn default() -> Self {
        Self {
            mapping: HashMap::new(),
            wildcard_rules: None,
        }
    }
}

impl ModelMapping {
    pub fn new(mapping: HashMap<String, String>) -> Self {
        Self {
            mapping,
            wildcard_rules: None,
        }
    }

    pub fn resolve(&self, model: &str) -> String {
        if let Some(target) = self.mapping.get(model) {
            return target.clone();
        }
        model.to_string()
    }

    pub fn reverse_resolve(&self, model: &str) -> String {
        let reverse: HashMap<&str, &str> = self
            .mapping
            .iter()
            .map(|(k, v)| (v.as_str(), k.as_str()))
            .collect();
        if let Some(target) = reverse.get(model) {
            return target.to_string();
        }
        model.to_string()
    }
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
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}
