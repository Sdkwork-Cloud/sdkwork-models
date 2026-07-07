from __future__ import annotations

from typing import Any, TypedDict

MODEL_FEATURES = ("streaming", "tool_call", "structured_output")
MODEL_INPUT_MODALITIES = ("text", "image", "audio", "music", "video")
MODEL_OUTPUT_MODALITIES = ("text", "image", "audio", "music", "video")


class ModelCapabilityProfile(TypedDict):
    catalogKey: str
    primaryCapability: str
    capabilities: list[str]
    inputModalities: list[str]
    outputModalities: list[str]
    features: list[str]
    supportsStreaming: bool
    supportsToolCall: bool
    supportsStructuredOutput: bool


def get_model_capability_profile(model: dict[str, Any]) -> ModelCapabilityProfile:
    return {
        "catalogKey": str(model.get("catalogKey", "")),
        "primaryCapability": str(model.get("primaryCapability", "")),
        "capabilities": list(model.get("capabilities", [])),
        "inputModalities": list(model.get("inputModalities", [])),
        "outputModalities": list(model.get("outputModalities", [])),
        "features": list_model_features(model),
        "supportsStreaming": model_supports_streaming(model),
        "supportsToolCall": model_supports_tool_call(model),
        "supportsStructuredOutput": model_supports_structured_output(model),
    }


def model_supports_input_modality(model: dict[str, Any], modality: str) -> bool:
    return modality in model.get("inputModalities", [])


def model_supports_output_modality(model: dict[str, Any], modality: str) -> bool:
    return modality in model.get("outputModalities", [])


def model_supports_text_input(model: dict[str, Any]) -> bool:
    return model_supports_input_modality(model, "text")


def model_supports_image_input(model: dict[str, Any]) -> bool:
    return model_supports_input_modality(model, "image")


def model_supports_audio_input(model: dict[str, Any]) -> bool:
    return model_supports_input_modality(model, "audio")


def model_supports_video_input(model: dict[str, Any]) -> bool:
    return model_supports_input_modality(model, "video")


def model_supports_text_output(model: dict[str, Any]) -> bool:
    return model_supports_output_modality(model, "text")


def model_supports_image_output(model: dict[str, Any]) -> bool:
    return model_supports_output_modality(model, "image")


def model_supports_audio_output(model: dict[str, Any]) -> bool:
    return model_supports_output_modality(model, "audio")


def model_supports_video_output(model: dict[str, Any]) -> bool:
    return model_supports_output_modality(model, "video")


def model_supports_vision(model: dict[str, Any]) -> bool:
    return model_supports_image_input(model)


def model_supports_speech_input(model: dict[str, Any]) -> bool:
    return model_supports_audio_input(model)


def model_supports_speech_output(model: dict[str, Any]) -> bool:
    return model_supports_audio_output(model)


def model_supports_feature(model: dict[str, Any], feature: str) -> bool:
    if feature == "streaming":
        return model_supports_streaming(model)
    if feature == "tool_call":
        return model_supports_tool_call(model)
    if feature == "structured_output":
        return model_supports_structured_output(model)
    return False


def model_supports_streaming(model: dict[str, Any]) -> bool:
    return model.get("supportsStreaming") is True


def model_supports_tool_call(model: dict[str, Any]) -> bool:
    return model.get("supportsTools") is True


def model_supports_structured_output(model: dict[str, Any]) -> bool:
    return model.get("supportsJsonSchema") is True


def list_model_features(model: dict[str, Any]) -> list[str]:
    return [feature for feature in MODEL_FEATURES if model_supports_feature(model, feature)]
