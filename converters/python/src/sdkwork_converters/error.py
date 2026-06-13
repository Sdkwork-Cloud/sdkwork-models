"""错误处理"""


class ConverterError(Exception):
    """转换器错误"""

    def __init__(self, message: str, code: str, details: dict | None = None):
        super().__init__(message)
        self.code = code
        self.details = details

    @staticmethod
    def unsupported_conversion(source: str, target: str) -> "ConverterError":
        return ConverterError(
            f"Unsupported conversion: {source} -> {target}",
            "UNSUPPORTED_CONVERSION",
        )

    @staticmethod
    def invalid_request(message: str) -> "ConverterError":
        return ConverterError(message, "INVALID_REQUEST")

    @staticmethod
    def invalid_response(message: str) -> "ConverterError":
        return ConverterError(message, "INVALID_RESPONSE")

    @staticmethod
    def model_mapping_not_found(model: str) -> "ConverterError":
        return ConverterError(
            f"Model mapping not found: {model}",
            "MODEL_MAPPING_NOT_FOUND",
        )

    @staticmethod
    def missing_field(field: str) -> "ConverterError":
        return ConverterError(
            f"Missing required field: {field}",
            "MISSING_FIELD",
        )
