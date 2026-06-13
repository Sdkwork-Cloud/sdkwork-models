import type { Converter } from '../traits';
import type {
  Protocol,
  Capability,
  ConversionRequest,
  ConversionResponse,
  ModelMapping,
} from '../types';
import { ConverterError } from '../error';
import { ModelMapper } from '../mappers';

/** OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES 转换器 */
export class OpenAiResponsesToAnthropicMessagesConverter implements Converter {
  private mapper = new ModelMapper();
  private modelMapping: ModelMapping;

  constructor(modelMapping?: ModelMapping) {
    this.modelMapping = modelMapping ?? { mapping: {} };
  }

  name(): string {
    return 'OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES';
  }

  sourceProtocol(): Protocol {
    return 'openai_responses' as Protocol;
  }

  targetProtocol(): Protocol {
    return 'anthropic_messages' as Protocol;
  }

  capabilities(): Capability[] {
    return ['stream', 'tools', 'vision', 'code', 'reasoning'] as Capability[];
  }

  canConvert(source: Protocol, target: Protocol): boolean {
    return source === this.sourceProtocol() && target === this.targetProtocol();
  }

  async convertRequest(request: ConversionRequest): Promise<ConversionRequest> {
    const model = this.mapper.map(request.model, this.modelMapping);

    const messages = request.messages.filter((m) => m.role !== 'system');

    return {
      ...request,
      protocol: this.targetProtocol(),
      model,
      messages,
    };
  }

  async convertResponse(response: ConversionResponse): Promise<ConversionResponse> {
    const model = this.mapper.reverseMap(response.model, this.modelMapping);

    return {
      ...response,
      protocol: this.sourceProtocol(),
      model,
    };
  }
}
