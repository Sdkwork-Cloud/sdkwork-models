"""转换器注册表"""

from __future__ import annotations
from typing import Any

from .traits import Converter
from .types import Protocol


class ConverterRegistry:
    """转换器注册表"""

    def __init__(self) -> None:
        self._converters: dict[str, Converter] = {}

    def register(self, converter: Converter) -> None:
        """注册转换器"""
        self._converters[converter.name()] = converter

    def unregister(self, name: str) -> bool:
        """注销转换器"""
        if name in self._converters:
            del self._converters[name]
            return True
        return False

    def get(self, name: str) -> Converter | None:
        """获取转换器"""
        return self._converters.get(name)

    def find(self, source: Protocol, target: Protocol) -> Converter | None:
        """按协议对查找转换器"""
        for converter in self._converters.values():
            if converter.can_convert(source, target):
                return converter
        return None

    def list(self) -> list[str]:
        """列出所有已注册的转换器名称"""
        return list(self._converters.keys())

    def by_source_protocol(self, protocol: Protocol) -> list[Converter]:
        """按源协议筛选"""
        return [c for c in self._converters.values() if c.source_protocol() == protocol]

    def by_target_protocol(self, protocol: Protocol) -> list[Converter]:
        """按目标协议筛选"""
        return [c for c in self._converters.values() if c.target_protocol() == protocol]

    def register_defaults(self) -> None:
        """注册所有内置转换器"""
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

        # Register all 12 converters
        self.register(OpenAiResponsesToAnthropicMessagesConverter())
        self.register(OpenAiResponsesToOpenAiCompletionsConverter())
        self.register(OpenAiResponsesToGoogleGeminiConverter())
        self.register(OpenAiCompletionsToAnthropicMessagesConverter())
        self.register(OpenAiCompletionsToOpenAiResponsesConverter())
        self.register(OpenAiCompletionsToGoogleGeminiConverter())
        self.register(AnthropicMessagesToOpenAiResponsesConverter())
        self.register(AnthropicMessagesToOpenAiCompletionsConverter())
        self.register(AnthropicMessagesToGoogleGeminiConverter())
        self.register(GoogleGeminiToAnthropicMessagesConverter())
        self.register(GoogleGeminiToOpenAiResponsesConverter())
        self.register(GoogleGeminiToOpenAiCompletionsConverter())
