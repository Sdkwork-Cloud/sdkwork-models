import 'types.dart';

const modelFeatures = ['streaming', 'tool_call', 'structured_output'];
const modelInputModalities = ['text', 'image', 'audio', 'music', 'video'];
const modelOutputModalities = ['text', 'image', 'audio', 'music', 'video'];

JsonObject getModelCapabilityProfile(JsonObject model) {
  return {
    'catalogKey': model['catalogKey'],
    'primaryCapability': model['primaryCapability'],
    'capabilities': _copyStringList(model['capabilities']),
    'inputModalities': _copyStringList(model['inputModalities']),
    'outputModalities': _copyStringList(model['outputModalities']),
    'features': listModelFeatures(model),
    'supportsStreaming': modelSupportsStreaming(model),
    'supportsToolCall': modelSupportsToolCall(model),
    'supportsStructuredOutput': modelSupportsStructuredOutput(model),
  };
}

bool modelSupportsInputModality(JsonObject model, String modality) {
  return _containsString(model['inputModalities'], modality);
}

bool modelSupportsOutputModality(JsonObject model, String modality) {
  return _containsString(model['outputModalities'], modality);
}

bool modelSupportsTextInput(JsonObject model) =>
    modelSupportsInputModality(model, 'text');

bool modelSupportsImageInput(JsonObject model) =>
    modelSupportsInputModality(model, 'image');

bool modelSupportsAudioInput(JsonObject model) =>
    modelSupportsInputModality(model, 'audio');

bool modelSupportsVideoInput(JsonObject model) =>
    modelSupportsInputModality(model, 'video');

bool modelSupportsTextOutput(JsonObject model) =>
    modelSupportsOutputModality(model, 'text');

bool modelSupportsImageOutput(JsonObject model) =>
    modelSupportsOutputModality(model, 'image');

bool modelSupportsAudioOutput(JsonObject model) =>
    modelSupportsOutputModality(model, 'audio');

bool modelSupportsVideoOutput(JsonObject model) =>
    modelSupportsOutputModality(model, 'video');

bool modelSupportsVision(JsonObject model) => modelSupportsImageInput(model);

bool modelSupportsSpeechInput(JsonObject model) => modelSupportsAudioInput(model);

bool modelSupportsSpeechOutput(JsonObject model) =>
    modelSupportsAudioOutput(model);

bool modelSupportsFeature(JsonObject model, String feature) {
  switch (feature) {
    case 'streaming':
      return modelSupportsStreaming(model);
    case 'tool_call':
      return modelSupportsToolCall(model);
    case 'structured_output':
      return modelSupportsStructuredOutput(model);
    default:
      return false;
  }
}

bool modelSupportsStreaming(JsonObject model) =>
    model['supportsStreaming'] == true;

bool modelSupportsToolCall(JsonObject model) => model['supportsTools'] == true;

bool modelSupportsStructuredOutput(JsonObject model) =>
    model['supportsJsonSchema'] == true;

List<String> listModelFeatures(JsonObject model) {
  return [
    for (final feature in modelFeatures)
      if (modelSupportsFeature(model, feature)) feature,
  ];
}

List<String> _copyStringList(Object? value) {
  if (value is! List) {
    return const [];
  }
  return [
    for (final item in value)
      if (item is String) item,
  ];
}

bool _containsString(Object? value, String expected) {
  if (value is! List) {
    return false;
  }
  return value.any((item) => item == expected);
}
