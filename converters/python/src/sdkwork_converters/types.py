"""类型定义"""

from enum import Enum
from typing import Any, Union
from pydantic import BaseModel, Field


class Protocol(str, Enum):
    """协议枚举"""
    OPENAI_RESPONSES = "openai_responses"
    OPENAI_COMPLETIONS = "openai_completions"
    ANTHROPIC_MESSAGES = "anthropic_messages"
    GOOGLE_GEMINI = "google_gemini"
    OPENAI_COMPATIBLE = "openai_compatible"
    VENDOR_NATIVE = "vendor_native"


class Capability(str, Enum):
    """能力枚举"""
    STREAM = "stream"
    TOOLS = "tools"
    VISION = "vision"
    AUDIO = "audio"
    VIDEO = "video"
    IMAGE = "image"
    CODE = "code"
    REASONING = "reasoning"


class Role(str, Enum):
    """角色枚举"""
    SYSTEM = "system"
    USER = "user"
    ASSISTANT = "assistant"
    TOOL = "tool"


class StopReason(str, Enum):
    """停止原因枚举"""
    END_TURN = "end_turn"
    STOP_SEQUENCE = "stop_sequence"
    MAX_TOKENS = "max_tokens"
    TOOL_USE = "tool_use"
    STOP = "stop"
    LENGTH = "length"
    CONTENT_FILTER = "content_filter"


class ToolType(str, Enum):
    """工具类型枚举"""
    FUNCTION = "function"


class TextContent(BaseModel):
    """文本内容"""
    type: str = "text"
    text: str


class ImageUrlContent(BaseModel):
    """图片URL内容"""
    type: str = "image_url"
    image_url: dict[str, Any]


class ToolUseContent(BaseModel):
    """工具调用内容"""
    type: str = "tool_use"
    id: str
    name: str
    input: Any


class ToolResultContent(BaseModel):
    """工具结果内容"""
    type: str = "tool_result"
    tool_use_id: str
    content: Union[str, list[Any]]


ContentPart = Union[TextContent, ImageUrlContent, ToolUseContent, ToolResultContent]


class Message(BaseModel):
    """消息"""
    role: Role
    content: Union[str, list[ContentPart]]


class Function(BaseModel):
    """函数定义"""
    name: str
    description: str | None = None
    parameters: Any = None
    input_schema: Any = None


class Tool(BaseModel):
    """工具定义"""
    type: ToolType = ToolType.FUNCTION
    function: Function


class Usage(BaseModel):
    """Token使用统计"""
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0
    cache_creation_input_tokens: int | None = None
    cache_read_input_tokens: int | None = None


class ConversionRequest(BaseModel):
    """转换请求"""
    protocol: Protocol
    model: str
    messages: list[Message]
    max_tokens: int | None = None
    temperature: float | None = None
    top_p: float | None = None
    stream: bool = False
    tools: list[Tool] | None = None
    system: str | None = None
    metadata: dict[str, Any] = Field(default_factory=dict)


class ConversionResponse(BaseModel):
    """转换响应"""
    protocol: Protocol
    id: str
    model: str
    content: list[ContentPart]
    stop_reason: StopReason | None = None
    usage: Usage = Field(default_factory=Usage)
    metadata: dict[str, Any] = Field(default_factory=dict)


class ModelMapping(BaseModel):
    """模型映射配置"""
    mapping: dict[str, str] = Field(default_factory=dict)
    wildcard_rules: list[dict[str, str]] | None = None

    def resolve(self, model: str) -> str:
        """解析模型名称"""
        return self.mapping.get(model, model)

    def reverse_resolve(self, model: str) -> str:
        """反向解析模型名称"""
        reverse = {v: k for k, v in self.mapping.items()}
        return reverse.get(model, model)


class ConverterConfig(BaseModel):
    """转换器配置"""
    name: str
    source_protocol: Protocol
    target_protocol: Protocol
    model_mapping: ModelMapping = Field(default_factory=ModelMapping)
    capabilities: list[Capability] = Field(default_factory=list)
    options: dict[str, Any] = Field(default_factory=dict)
