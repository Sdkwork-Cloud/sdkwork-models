"""模型映射器"""

from ..traits import Mapper
from ..types import ModelMapping


class ModelMapper(Mapper):
    """标准模型映射器"""

    def name(self) -> str:
        return "model_mapper"

    def map(self, source_model: str, mapping: ModelMapping) -> str:
        return mapping.resolve(source_model)

    def reverse_map(self, target_model: str, mapping: ModelMapping) -> str:
        return mapping.reverse_resolve(target_model)


class PrefixMapper(Mapper):
    """前缀映射器"""

    def __init__(self, prefix: str) -> None:
        self.prefix = prefix

    def name(self) -> str:
        return "prefix_mapper"

    def map(self, source_model: str, mapping: ModelMapping) -> str:
        if source_model.startswith(self.prefix):
            return source_model
        return f"{self.prefix}{source_model}"

    def reverse_map(self, target_model: str, mapping: ModelMapping) -> str:
        if target_model.startswith(self.prefix):
            return target_model[len(self.prefix):]
        return target_model
