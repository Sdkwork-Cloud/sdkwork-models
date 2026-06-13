"""ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS 转换器"""

from ..traits import Converter
from ..types import Protocol, Capability, ConversionRequest, ConversionResponse, ModelMapping
from ..mappers import ModelMapper


class AnthropicMessagesToOpenAiCompletionsConverter(Converter):
    def __init__(self, model_mapping: ModelMapping | None = None) -> None:
        self.model_mapping = model_mapping or ModelMapping()
        self.mapper = ModelMapper()

    def name(self) -> str: return "ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS"
    def source_protocol(self) -> Protocol: return Protocol.ANTHROPIC_MESSAGES
    def target_protocol(self) -> Protocol: return Protocol.OPENAI_COMPLETIONS
    def capabilities(self) -> list[Capability]: return [Capability.STREAM, Capability.TOOLS, Capability.CODE, Capability.REASONING]

    async def convert_request(self, request: ConversionRequest) -> ConversionRequest:
        model = self.mapper.map(request.model, self.model_mapping)
        messages = [m for m in request.messages if m.role.value != "system"]
        return request.model_copy(update={"protocol": Protocol.OPENAI_COMPLETIONS, "model": model, "messages": messages, "max_tokens": request.max_tokens or 4096})

    async def convert_response(self, response: ConversionResponse) -> ConversionResponse:
        model = self.mapper.reverse_map(response.model, self.model_mapping)
        return response.model_copy(update={"protocol": Protocol.ANTHROPIC_MESSAGES, "model": model})
