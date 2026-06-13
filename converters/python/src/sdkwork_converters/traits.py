"""核心接口定义"""

from abc import ABC, abstractmethod

from .types import (
    Protocol,
    Capability,
    ConversionRequest,
    ConversionResponse,
    ModelMapping,
)


class Converter(ABC):
    """转换器接口"""

    @abstractmethod
    def name(self) -> str:
        """转换器名称"""
        ...

    @abstractmethod
    def source_protocol(self) -> Protocol:
        """源协议"""
        ...

    @abstractmethod
    def target_protocol(self) -> Protocol:
        """目标协议"""
        ...

    @abstractmethod
    def capabilities(self) -> list[Capability]:
        """支持的能力"""
        ...

    def can_convert(self, source: Protocol, target: Protocol) -> bool:
        """是否支持该转换"""
        return self.source_protocol() == source and self.target_protocol() == target

    @abstractmethod
    async def convert_request(self, request: ConversionRequest) -> ConversionRequest:
        """转换请求"""
        ...

    @abstractmethod
    async def convert_response(self, response: ConversionResponse) -> ConversionResponse:
        """转换响应"""
        ...


class Mapper(ABC):
    """映射器接口"""

    @abstractmethod
    def name(self) -> str:
        """映射器名称"""
        ...

    @abstractmethod
    def map(self, source_model: str, mapping: ModelMapping) -> str:
        """映射单个模型"""
        ...

    @abstractmethod
    def reverse_map(self, target_model: str, mapping: ModelMapping) -> str:
        """反向映射"""
        ...

    def map_batch(self, models: list[str], mapping: ModelMapping) -> list[str]:
        """批量映射"""
        return [self.map(m, mapping) for m in models]
