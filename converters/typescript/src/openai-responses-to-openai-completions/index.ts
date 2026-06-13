import type { Converter } from '../traits';
import type { Protocol, Capability, ConversionRequest, ConversionResponse, ModelMapping } from '../types';
import { ModelMapper } from '../mappers';

export class OpenAiResponsesToOpenAiCompletionsConverter implements Converter {
  private mapper = new ModelMapper();
  private modelMapping: ModelMapping;

  constructor(modelMapping?: ModelMapping) {
    this.modelMapping = modelMapping ?? { mapping: {} };
  }

  name(): string { return 'OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS'; }
  sourceProtocol(): Protocol { return 'openai_responses' as Protocol; }
  targetProtocol(): Protocol { return 'openai_completions' as Protocol; }
  capabilities(): Capability[] { return ['stream', 'tools', 'vision', 'code'] as Capability[]; }

  canConvert(source: Protocol, target: Protocol): boolean {
    return source === this.sourceProtocol() && target === this.targetProtocol();
  }

  async convertRequest(request: ConversionRequest): Promise<ConversionRequest> {
    const model = this.mapper.map(request.model, this.modelMapping);
    return { ...request, protocol: this.targetProtocol(), model };
  }

  async convertResponse(response: ConversionResponse): Promise<ConversionResponse> {
    const model = this.mapper.reverseMap(response.model, this.modelMapping);
    return { ...response, protocol: this.sourceProtocol(), model };
  }
}
