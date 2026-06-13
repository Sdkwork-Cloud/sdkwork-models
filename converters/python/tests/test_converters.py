"""Python SDKWork Converters 测试 - 12个转换器"""

import pytest
import asyncio
from sdkwork_converters import (
    Protocol,
    Capability,
    Role,
    ConversionRequest,
    ConversionResponse,
    Message,
    Usage,
    ModelMapping,
    ConverterRegistry,
    ModelMapper,
    OpenAiResponsesToAnthropicMessagesConverter,
    OpenAiCompletionsToAnthropicMessagesConverter,
    OpenAiResponsesToOpenAiCompletionsConverter,
    OpenAiCompletionsToOpenAiResponsesConverter,
    OpenAiResponsesToGoogleGeminiConverter,
    OpenAiCompletionsToGoogleGeminiConverter,
    AnthropicMessagesToOpenAiResponsesConverter,
    AnthropicMessagesToOpenAiCompletionsConverter,
    AnthropicMessagesToGoogleGeminiConverter,
    GoogleGeminiToAnthropicMessagesConverter,
    GoogleGeminiToOpenAiResponsesConverter,
    GoogleGeminiToOpenAiCompletionsConverter,
)


def make_request(protocol: Protocol, model: str, user_msg: str = "Hello") -> ConversionRequest:
    return ConversionRequest(
        protocol=protocol,
        model=model,
        messages=[Message(role=Role.USER, content=user_msg)],
        max_tokens=1024,
        temperature=0.7,
        stream=False,
    )


def make_response(protocol: Protocol, model: str, text: str = "Hello!") -> ConversionResponse:
    return ConversionResponse(
        protocol=protocol,
        id="resp_123",
        model=model,
        content=[],
        usage=Usage(prompt_tokens=10, completion_tokens=5, total_tokens=15),
    )


# All 12 converters with their metadata
CONVERTERS = [
    ("OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES", OpenAiResponsesToAnthropicMessagesConverter, Protocol.OPENAI_RESPONSES, Protocol.ANTHROPIC_MESSAGES),
    ("OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS", OpenAiResponsesToOpenAiCompletionsConverter, Protocol.OPENAI_RESPONSES, Protocol.OPENAI_COMPLETIONS),
    ("OPENAI_RESPONSES_TO_GOOGLE_GEMINI", OpenAiResponsesToGoogleGeminiConverter, Protocol.OPENAI_RESPONSES, Protocol.GOOGLE_GEMINI),
    ("OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES", OpenAiCompletionsToAnthropicMessagesConverter, Protocol.OPENAI_COMPLETIONS, Protocol.ANTHROPIC_MESSAGES),
    ("OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES", OpenAiCompletionsToOpenAiResponsesConverter, Protocol.OPENAI_COMPLETIONS, Protocol.OPENAI_RESPONSES),
    ("OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI", OpenAiCompletionsToGoogleGeminiConverter, Protocol.OPENAI_COMPLETIONS, Protocol.GOOGLE_GEMINI),
    ("ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES", AnthropicMessagesToOpenAiResponsesConverter, Protocol.ANTHROPIC_MESSAGES, Protocol.OPENAI_RESPONSES),
    ("ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS", AnthropicMessagesToOpenAiCompletionsConverter, Protocol.ANTHROPIC_MESSAGES, Protocol.OPENAI_COMPLETIONS),
    ("ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI", AnthropicMessagesToGoogleGeminiConverter, Protocol.ANTHROPIC_MESSAGES, Protocol.GOOGLE_GEMINI),
    ("GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES", GoogleGeminiToAnthropicMessagesConverter, Protocol.GOOGLE_GEMINI, Protocol.ANTHROPIC_MESSAGES),
    ("GOOGLE_GEMINI_TO_OPENAI_RESPONSES", GoogleGeminiToOpenAiResponsesConverter, Protocol.GOOGLE_GEMINI, Protocol.OPENAI_RESPONSES),
    ("GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS", GoogleGeminiToOpenAiCompletionsConverter, Protocol.GOOGLE_GEMINI, Protocol.OPENAI_COMPLETIONS),
]


class TestModelMapper:
    def test_direct_mapping(self):
        mapper = ModelMapper()
        mapping = ModelMapping(mapping={"gpt-5.5": "claude-sonnet-4"})
        assert mapper.map("gpt-5.5", mapping) == "claude-sonnet-4"
        assert mapper.map("unknown", mapping) == "unknown"

    def test_reverse_mapping(self):
        mapper = ModelMapper()
        mapping = ModelMapping(mapping={"gpt-5.5": "claude-sonnet-4"})
        assert mapper.reverse_map("claude-sonnet-4", mapping) == "gpt-5.5"


class TestAllConverters:
    @pytest.mark.asyncio
    @pytest.mark.parametrize("name,cls,src,tgt", CONVERTERS)
    async def test_metadata(self, name, cls, src, tgt):
        converter = cls()
        assert converter.name() == name
        assert converter.source_protocol() == src
        assert converter.target_protocol() == tgt
        assert len(converter.capabilities()) > 0

    @pytest.mark.asyncio
    @pytest.mark.parametrize("name,cls,src,tgt", CONVERTERS)
    async def test_can_convert(self, name, cls, src, tgt):
        converter = cls()
        assert converter.can_convert(src, tgt) is True
        assert converter.can_convert(tgt, src) is False

    @pytest.mark.asyncio
    @pytest.mark.parametrize("name,cls,src,tgt", CONVERTERS)
    async def test_convert_request_basic(self, name, cls, src, tgt):
        converter = cls()
        req = make_request(src, "test-model")
        result = await converter.convert_request(req)
        assert result.protocol == tgt
        assert result.model == "test-model"

    @pytest.mark.asyncio
    @pytest.mark.parametrize("name,cls,src,tgt", CONVERTERS)
    async def test_convert_request_with_model_mapping(self, name, cls, src, tgt):
        mapping = ModelMapping(mapping={"test-model": "mapped-model"})
        converter = cls(model_mapping=mapping)
        req = make_request(src, "test-model")
        result = await converter.convert_request(req)
        assert result.model == "mapped-model"

    @pytest.mark.asyncio
    @pytest.mark.parametrize("name,cls,src,tgt", CONVERTERS)
    async def test_convert_response_basic(self, name, cls, src, tgt):
        converter = cls()
        resp = make_response(tgt, "test-model")
        result = await converter.convert_response(resp)
        assert result.protocol == src
        assert result.model == "test-model"

    @pytest.mark.asyncio
    @pytest.mark.parametrize("name,cls,src,tgt", CONVERTERS)
    async def test_convert_response_with_model_mapping(self, name, cls, src, tgt):
        mapping = ModelMapping(mapping={"test-model": "mapped-model"})
        converter = cls(model_mapping=mapping)
        resp = make_response(tgt, "mapped-model")
        result = await converter.convert_response(resp)
        assert result.model == "test-model"


class TestConverterRegistry:
    def test_register_and_get(self):
        registry = ConverterRegistry()
        converter = OpenAiResponsesToAnthropicMessagesConverter()
        registry.register(converter)
        assert registry.get("OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES") is converter

    def test_find_by_protocol(self):
        registry = ConverterRegistry()
        converter = OpenAiResponsesToAnthropicMessagesConverter()
        registry.register(converter)
        found = registry.find(Protocol.OPENAI_RESPONSES, Protocol.ANTHROPIC_MESSAGES)
        assert found is converter

    def test_register_defaults(self):
        registry = ConverterRegistry()
        registry.register_defaults()
        assert len(registry.list()) == 12
        assert "OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES" in registry.list()
        assert "OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS" in registry.list()
        assert "OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES" in registry.list()
        assert "GOOGLE_GEMINI_TO_OPENAI_RESPONSES" in registry.list()
        assert "GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS" in registry.list()
