"""OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS 转换器"""

from ..traits import Converter
from ..types import Protocol, Capability, ConversionRequest, ConversionResponse, ModelMapping
from ..mappers import ModelMapper


class OpenAiResponsesToOpenAiCompletionsConverter(Converter):
    def __init__(self, model_mapping: ModelMapping | None = None) -> None:
        self.model_mapping = model_mapping or ModelMapping()
        self.mapper = ModelMapper()

    def name(self) -> str: return "OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS"
    def source_protocol(self) -> Protocol: return Protocol.OPENAI_RESPONSES
    def target_protocol(self) -> Protocol: return Protocol.OPENAI_COMPLETIONS
    def capabilities(self) -> list[Capability]: return [Capability.STREAM, Capability.TOOLS, Capability.VISION, Capability.CODE]

    async def convert_request(self, request: ConversionRequest) -> ConversionRequest:
        model = self.mapper.map(request.model, self.model_mapping)
        return request.model_copy(update={"protocol": Protocol.OPENAI_COMPLETIONS, "model": model})

    async def convert_response(self, response: ConversionResponse) -> ConversionResponse:
        model = self.mapper.reverse_map(response.model, self.model_mapping)
        return response.model_copy(update={"protocol": Protocol.OPENAI_RESPONSES, "model": model})
