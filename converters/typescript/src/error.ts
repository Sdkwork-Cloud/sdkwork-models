/** 转换器错误 */
export class ConverterError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: unknown
  ) {
    super(message);
    this.name = 'ConverterError';
  }

  static unsupportedConversion(from: string, to: string): ConverterError {
    return new ConverterError(
      `Unsupported conversion: ${from} -> ${to}`,
      'UNSUPPORTED_CONVERSION'
    );
  }

  static invalidRequest(message: string): ConverterError {
    return new ConverterError(message, 'INVALID_REQUEST');
  }

  static invalidResponse(message: string): ConverterError {
    return new ConverterError(message, 'INVALID_RESPONSE');
  }

  static modelMappingNotFound(model: string): ConverterError {
    return new ConverterError(
      `Model mapping not found: ${model}`,
      'MODEL_MAPPING_NOT_FOUND'
    );
  }

  static missingField(field: string): ConverterError {
    return new ConverterError(
      `Missing required field: ${field}`,
      'MISSING_FIELD'
    );
  }
}
