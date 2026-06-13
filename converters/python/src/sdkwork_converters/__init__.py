"""SDKWork Client API Protocol Converters"""

from .types import Protocol, Capability, Role, StopReason, ToolType, ConversionRequest, ConversionResponse, Message, ContentPart, Tool, Function, Usage, ModelMapping
from .error import ConverterError
from .traits import Converter, Mapper
from .registry import ConverterRegistry
from .mappers import ModelMapper, PrefixMapper

# OpenAI Responses →
from .openai_responses_to_anthropic_messages import OpenAiResponsesToAnthropicMessagesConverter
from .openai_responses_to_openai_completions import OpenAiResponsesToOpenAiCompletionsConverter
from .openai_responses_to_google_gemini import OpenAiResponsesToGoogleGeminiConverter

# OpenAI Completions →
from .openai_completions_to_anthropic_messages import OpenAiCompletionsToAnthropicMessagesConverter
from .openai_completions_to_openai_responses import OpenAiCompletionsToOpenAiResponsesConverter
from .openai_completions_to_google_gemini import OpenAiCompletionsToGoogleGeminiConverter

# Anthropic Messages →
from .anthropic_messages_to_openai_responses import AnthropicMessagesToOpenAiResponsesConverter
from .anthropic_messages_to_openai_completions import AnthropicMessagesToOpenAiCompletionsConverter
from .anthropic_messages_to_google_gemini import AnthropicMessagesToGoogleGeminiConverter

# Google Gemini →
from .google_gemini_to_anthropic_messages import GoogleGeminiToAnthropicMessagesConverter
from .google_gemini_to_openai_responses import GoogleGeminiToOpenAiResponsesConverter
from .google_gemini_to_openai_completions import GoogleGeminiToOpenAiCompletionsConverter

__all__ = [
    "Protocol", "Capability", "Role", "StopReason", "ToolType",
    "ConversionRequest", "ConversionResponse", "Message", "ContentPart", "Tool", "Function", "Usage", "ModelMapping",
    "ConverterError", "Converter", "Mapper", "ConverterRegistry", "ModelMapper", "PrefixMapper",
    "OpenAiResponsesToAnthropicMessagesConverter", "OpenAiResponsesToOpenAiCompletionsConverter", "OpenAiResponsesToGoogleGeminiConverter",
    "OpenAiCompletionsToAnthropicMessagesConverter", "OpenAiCompletionsToOpenAiResponsesConverter", "OpenAiCompletionsToGoogleGeminiConverter",
    "AnthropicMessagesToOpenAiResponsesConverter", "AnthropicMessagesToOpenAiCompletionsConverter", "AnthropicMessagesToGoogleGeminiConverter",
    "GoogleGeminiToAnthropicMessagesConverter", "GoogleGeminiToOpenAiResponsesConverter", "GoogleGeminiToOpenAiCompletionsConverter",
]
