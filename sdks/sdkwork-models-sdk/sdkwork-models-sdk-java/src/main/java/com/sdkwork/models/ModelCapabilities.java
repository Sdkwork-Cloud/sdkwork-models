package com.sdkwork.models;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

/**
 * Model capability predicates aligned with catalog modality arrays and supports* flags.
 *
 * <p>Industry alignment:
 * <ul>
 *   <li>Input/output modalities mirror OpenAI model modalities and Gemini input/output modalities.</li>
 *   <li>{@code tool_call} maps to OpenAI/Anthropic/Gemini function or tool calling ({@code supportsTools}).</li>
 *   <li>{@code structured_output} maps to JSON Schema constrained generation ({@code supportsJsonSchema}).</li>
 *   <li>{@code streaming} maps to streamed response delivery ({@code supportsStreaming}).</li>
 * </ul>
 */
public final class ModelCapabilities {
    public static final List<String> MODEL_FEATURES = List.of("streaming", "tool_call", "structured_output");
    public static final List<String> MODEL_INPUT_MODALITIES = List.of("text", "image", "audio", "music", "video");
    public static final List<String> MODEL_OUTPUT_MODALITIES = List.of("text", "image", "audio", "music", "video");

    private ModelCapabilities() {}

    public static Map<String, Object> getModelCapabilityProfile(Map<String, Object> model) {
        Map<String, Object> profile = new LinkedHashMap<>();
        profile.put("catalogKey", model.get("catalogKey"));
        profile.put("primaryCapability", model.get("primaryCapability"));
        profile.put("capabilities", copyStringList(model.get("capabilities")));
        profile.put("inputModalities", copyStringList(model.get("inputModalities")));
        profile.put("outputModalities", copyStringList(model.get("outputModalities")));
        profile.put("features", listModelFeatures(model));
        profile.put("supportsStreaming", modelSupportsStreaming(model));
        profile.put("supportsToolCall", modelSupportsToolCall(model));
        profile.put("supportsStructuredOutput", modelSupportsStructuredOutput(model));
        return profile;
    }

    public static boolean modelSupportsInputModality(Map<String, Object> model, String modality) {
        return containsString(model.get("inputModalities"), modality);
    }

    public static boolean modelSupportsOutputModality(Map<String, Object> model, String modality) {
        return containsString(model.get("outputModalities"), modality);
    }

    public static boolean modelSupportsTextInput(Map<String, Object> model) {
        return modelSupportsInputModality(model, "text");
    }

    public static boolean modelSupportsImageInput(Map<String, Object> model) {
        return modelSupportsInputModality(model, "image");
    }

    public static boolean modelSupportsAudioInput(Map<String, Object> model) {
        return modelSupportsInputModality(model, "audio");
    }

    public static boolean modelSupportsVideoInput(Map<String, Object> model) {
        return modelSupportsInputModality(model, "video");
    }

    public static boolean modelSupportsTextOutput(Map<String, Object> model) {
        return modelSupportsOutputModality(model, "text");
    }

    public static boolean modelSupportsImageOutput(Map<String, Object> model) {
        return modelSupportsOutputModality(model, "image");
    }

    public static boolean modelSupportsAudioOutput(Map<String, Object> model) {
        return modelSupportsOutputModality(model, "audio");
    }

    public static boolean modelSupportsVideoOutput(Map<String, Object> model) {
        return modelSupportsOutputModality(model, "video");
    }

    public static boolean modelSupportsVision(Map<String, Object> model) {
        return modelSupportsImageInput(model);
    }

    public static boolean modelSupportsSpeechInput(Map<String, Object> model) {
        return modelSupportsAudioInput(model);
    }

    public static boolean modelSupportsSpeechOutput(Map<String, Object> model) {
        return modelSupportsAudioOutput(model);
    }

    public static boolean modelSupportsFeature(Map<String, Object> model, String feature) {
        return switch (feature) {
            case "streaming" -> modelSupportsStreaming(model);
            case "tool_call" -> modelSupportsToolCall(model);
            case "structured_output" -> modelSupportsStructuredOutput(model);
            default -> false;
        };
    }

    public static boolean modelSupportsStreaming(Map<String, Object> model) {
        return Objects.equals(model.get("supportsStreaming"), Boolean.TRUE);
    }

    public static boolean modelSupportsToolCall(Map<String, Object> model) {
        return Objects.equals(model.get("supportsTools"), Boolean.TRUE);
    }

    public static boolean modelSupportsStructuredOutput(Map<String, Object> model) {
        return Objects.equals(model.get("supportsJsonSchema"), Boolean.TRUE);
    }

    public static List<String> listModelFeatures(Map<String, Object> model) {
        List<String> features = new ArrayList<>();
        for (String feature : MODEL_FEATURES) {
            if (modelSupportsFeature(model, feature)) {
                features.add(feature);
            }
        }
        return features;
    }

    private static boolean containsString(Object value, String expected) {
        if (!(value instanceof List<?> list)) {
            return false;
        }
        return list.stream().anyMatch(item -> Objects.equals(item, expected));
    }

    @SuppressWarnings("unchecked")
    private static List<String> copyStringList(Object value) {
        if (!(value instanceof List<?> list)) {
            return List.of();
        }
        List<String> copied = new ArrayList<>();
        for (Object item : list) {
            if (item instanceof String string) {
                copied.add(string);
            }
        }
        return copied;
    }
}
