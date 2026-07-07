import type { ModelCapability, ModelInfo, ModelModality } from "./types.js";

/**
 * Protocol interaction features aligned with industry provider metadata:
 * - streaming: OpenAI stream / Gemini streamGenerateContent
 * - tool_call: OpenAI tools / Anthropic tools / Gemini function calling
 * - structured_output: OpenAI json_schema response_format / Gemini responseSchema
 */
export type ModelFeature = "streaming" | "tool_call" | "structured_output";

export const MODEL_FEATURES: readonly ModelFeature[] = [
  "streaming",
  "tool_call",
  "structured_output",
] as const;

export const MODEL_INPUT_MODALITIES: readonly ModelModality[] = [
  "text",
  "image",
  "audio",
  "music",
  "video",
] as const;

export const MODEL_OUTPUT_MODALITIES: readonly ModelModality[] = [
  "text",
  "image",
  "audio",
  "music",
  "video",
] as const;

export type ModelCapabilitySubject = Pick<
  ModelInfo,
  | "catalogKey"
  | "primaryCapability"
  | "capabilities"
  | "inputModalities"
  | "outputModalities"
  | "supportsStreaming"
  | "supportsTools"
  | "supportsJsonSchema"
>;

export interface ModelCapabilityProfile {
  catalogKey: string;
  primaryCapability: ModelCapability;
  capabilities: ModelCapability[];
  inputModalities: ModelModality[];
  outputModalities: ModelModality[];
  features: ModelFeature[];
  supportsStreaming: boolean;
  supportsToolCall: boolean;
  supportsStructuredOutput: boolean;
}

export function getModelCapabilityProfile(model: ModelCapabilitySubject): ModelCapabilityProfile {
  return {
    catalogKey: model.catalogKey,
    primaryCapability: model.primaryCapability,
    capabilities: [...model.capabilities],
    inputModalities: [...model.inputModalities],
    outputModalities: [...model.outputModalities],
    features: listModelFeatures(model),
    supportsStreaming: modelSupportsStreaming(model),
    supportsToolCall: modelSupportsToolCall(model),
    supportsStructuredOutput: modelSupportsStructuredOutput(model),
  };
}

export function modelSupportsInputModality(
  model: ModelCapabilitySubject,
  modality: ModelModality,
): boolean {
  return model.inputModalities.includes(modality);
}

export function modelSupportsOutputModality(
  model: ModelCapabilitySubject,
  modality: ModelModality,
): boolean {
  return model.outputModalities.includes(modality);
}

export function modelSupportsTextInput(model: ModelCapabilitySubject): boolean {
  return modelSupportsInputModality(model, "text");
}

export function modelSupportsImageInput(model: ModelCapabilitySubject): boolean {
  return modelSupportsInputModality(model, "image");
}

export function modelSupportsAudioInput(model: ModelCapabilitySubject): boolean {
  return modelSupportsInputModality(model, "audio");
}

export function modelSupportsVideoInput(model: ModelCapabilitySubject): boolean {
  return modelSupportsInputModality(model, "video");
}

export function modelSupportsTextOutput(model: ModelCapabilitySubject): boolean {
  return modelSupportsOutputModality(model, "text");
}

export function modelSupportsImageOutput(model: ModelCapabilitySubject): boolean {
  return modelSupportsOutputModality(model, "image");
}

export function modelSupportsAudioOutput(model: ModelCapabilitySubject): boolean {
  return modelSupportsOutputModality(model, "audio");
}

export function modelSupportsVideoOutput(model: ModelCapabilitySubject): boolean {
  return modelSupportsOutputModality(model, "video");
}

/** Industry alias for image input (vision). */
export function modelSupportsVision(model: ModelCapabilitySubject): boolean {
  return modelSupportsImageInput(model);
}

/** Industry alias for audio input (speech / voice). */
export function modelSupportsSpeechInput(model: ModelCapabilitySubject): boolean {
  return modelSupportsAudioInput(model);
}

/** Industry alias for audio output (TTS / speech synthesis). */
export function modelSupportsSpeechOutput(model: ModelCapabilitySubject): boolean {
  return modelSupportsAudioOutput(model);
}

export function modelSupportsFeature(model: ModelCapabilitySubject, feature: ModelFeature): boolean {
  switch (feature) {
    case "streaming":
      return modelSupportsStreaming(model);
    case "tool_call":
      return modelSupportsToolCall(model);
    case "structured_output":
      return modelSupportsStructuredOutput(model);
    default:
      return false;
  }
}

export function modelSupportsStreaming(model: ModelCapabilitySubject): boolean {
  return model.supportsStreaming === true;
}

export function modelSupportsToolCall(model: ModelCapabilitySubject): boolean {
  return model.supportsTools === true;
}

export function modelSupportsStructuredOutput(model: ModelCapabilitySubject): boolean {
  return model.supportsJsonSchema === true;
}

export function listModelFeatures(model: ModelCapabilitySubject): ModelFeature[] {
  return MODEL_FEATURES.filter((feature) => modelSupportsFeature(model, feature));
}
