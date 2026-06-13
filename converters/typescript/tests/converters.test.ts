import {
  OpenAiResponsesToAnthropicMessagesConverter,
  OpenAiResponsesToOpenAiCompletionsConverter,
  OpenAiResponsesToGoogleGeminiConverter,
  OpenAiCompletionsToAnthropicMessagesConverter,
  OpenAiCompletionsToOpenAiResponsesConverter,
  OpenAiCompletionsToGoogleGeminiConverter,
  AnthropicMessagesToOpenAiResponsesConverter,
  AnthropicMessagesToOpenAiCompletionsConverter,
  AnthropicMessagesToGoogleGeminiConverter,
  GoogleGeminiToAnthropicMessagesConverter,
  GoogleGeminiToOpenAiResponsesConverter,
  GoogleGeminiToOpenAiCompletionsConverter,
  ModelMapper,
  ConverterRegistry,
  Protocol,
} from '../src/index';

describe('ModelMapper', () => {
  test('direct mapping', () => {
    const mapper = new ModelMapper();
    const mapping = { mapping: { 'gpt-5.5': 'claude-sonnet-4' } };
    expect(mapper.map('gpt-5.5', mapping)).toBe('claude-sonnet-4');
    expect(mapper.map('unknown', mapping)).toBe('unknown');
  });

  test('reverse mapping', () => {
    const mapper = new ModelMapper();
    const mapping = { mapping: { 'gpt-5.5': 'claude-sonnet-4' } };
    expect(mapper.reverseMap('claude-sonnet-4', mapping)).toBe('gpt-5.5');
  });
});

// Helper to create a basic request
function makeRequest(protocol: Protocol, model: string, content: string = 'Hello') {
  return {
    protocol,
    model,
    messages: [{ role: 'user' as const, content }],
    stream: false,
    metadata: {},
  };
}

// Helper to create a basic response
function makeResponse(protocol: Protocol, model: string, text: string = 'Hello!') {
  return {
    protocol,
    id: 'resp_123',
    model,
    content: [{ type: 'text' as const, text }],
    usage: { promptTokens: 10, completionTokens: 5, totalTokens: 15 },
    metadata: {},
  };
}

// Test all 12 converters
const converters = [
  { name: 'OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES', Class: OpenAiResponsesToAnthropicMessagesConverter, src: Protocol.OpenAiResponses, tgt: Protocol.AnthropicMessages },
  { name: 'OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS', Class: OpenAiResponsesToOpenAiCompletionsConverter, src: Protocol.OpenAiResponses, tgt: Protocol.OpenAiCompletions },
  { name: 'OPENAI_RESPONSES_TO_GOOGLE_GEMINI', Class: OpenAiResponsesToGoogleGeminiConverter, src: Protocol.OpenAiResponses, tgt: Protocol.GoogleGemini },
  { name: 'OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES', Class: OpenAiCompletionsToAnthropicMessagesConverter, src: Protocol.OpenAiCompletions, tgt: Protocol.AnthropicMessages },
  { name: 'OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES', Class: OpenAiCompletionsToOpenAiResponsesConverter, src: Protocol.OpenAiCompletions, tgt: Protocol.OpenAiResponses },
  { name: 'OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI', Class: OpenAiCompletionsToGoogleGeminiConverter, src: Protocol.OpenAiCompletions, tgt: Protocol.GoogleGemini },
  { name: 'ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES', Class: AnthropicMessagesToOpenAiResponsesConverter, src: Protocol.AnthropicMessages, tgt: Protocol.OpenAiResponses },
  { name: 'ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS', Class: AnthropicMessagesToOpenAiCompletionsConverter, src: Protocol.AnthropicMessages, tgt: Protocol.OpenAiCompletions },
  { name: 'ANTHROPIC_MESSAGES_TO_GOOGLE_GEMINI', Class: AnthropicMessagesToGoogleGeminiConverter, src: Protocol.AnthropicMessages, tgt: Protocol.GoogleGemini },
  { name: 'GOOGLE_GEMINI_TO_ANTHROPIC_MESSAGES', Class: GoogleGeminiToAnthropicMessagesConverter, src: Protocol.GoogleGemini, tgt: Protocol.AnthropicMessages },
  { name: 'GOOGLE_GEMINI_TO_OPENAI_RESPONSES', Class: GoogleGeminiToOpenAiResponsesConverter, src: Protocol.GoogleGemini, tgt: Protocol.OpenAiResponses },
  { name: 'GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS', Class: GoogleGeminiToOpenAiCompletionsConverter, src: Protocol.GoogleGemini, tgt: Protocol.OpenAiCompletions },
];

describe('All 12 Converters', () => {
  converters.forEach(({ name, Class, src, tgt }) => {
    describe(name, () => {
      const converter = new Class();

      test('metadata', () => {
        expect(converter.name()).toBe(name);
        expect(converter.sourceProtocol()).toBe(src);
        expect(converter.targetProtocol()).toBe(tgt);
        expect(converter.capabilities().length).toBeGreaterThan(0);
      });

      test('canConvert', () => {
        expect(converter.canConvert(src, tgt)).toBe(true);
        expect(converter.canConvert(tgt, src)).toBe(false);
      });

      test('convertRequest basic', async () => {
        const req = makeRequest(src, 'test-model');
        const result = await converter.convertRequest(req as any);
        expect(result.protocol).toBe(tgt);
        expect(result.model).toBe('test-model');
      });

      test('convertRequest with model mapping', async () => {
        const mapped = new Class({ mapping: { 'test-model': 'mapped-model' } });
        const req = makeRequest(src, 'test-model');
        const result = await mapped.convertRequest(req as any);
        expect(result.model).toBe('mapped-model');
      });

      test('convertResponse basic', async () => {
        const resp = makeResponse(tgt, 'test-model');
        const result = await converter.convertResponse(resp as any);
        expect(result.protocol).toBe(src);
        expect(result.model).toBe('test-model');
      });

      test('convertResponse with model mapping', async () => {
        const mapped = new Class({ mapping: { 'test-model': 'mapped-model' } });
        const resp = makeResponse(tgt, 'mapped-model');
        const result = await mapped.convertResponse(resp as any);
        expect(result.model).toBe('test-model');
      });
    });
  });
});

describe('ConverterRegistry', () => {
  test('register and get', () => {
    const registry = new ConverterRegistry();
    const converter = new OpenAiResponsesToAnthropicMessagesConverter();
    registry.register(converter as any);
    expect(registry.get('OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES')).toBe(converter);
  });

  test('find by protocol', () => {
    const registry = new ConverterRegistry();
    const converter = new OpenAiResponsesToAnthropicMessagesConverter();
    registry.register(converter as any);
    const found = registry.find(Protocol.OpenAiResponses, Protocol.AnthropicMessages);
    expect(found).toBe(converter);
  });

  test('registerDefaults registers all 12', () => {
    const registry = new ConverterRegistry();
    registry.registerDefaults();
    const list = registry.list();
    expect(list).toHaveLength(12);
    expect(list).toContain('OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES');
    expect(list).toContain('OPENAI_RESPONSES_TO_OPENAI_COMPLETIONS');
    expect(list).toContain('OPENAI_COMPLETIONS_TO_OPENAI_RESPONSES');
    expect(list).toContain('OPENAI_COMPLETIONS_TO_GOOGLE_GEMINI');
    expect(list).toContain('GOOGLE_GEMINI_TO_OPENAI_RESPONSES');
    expect(list).toContain('GOOGLE_GEMINI_TO_OPENAI_COMPLETIONS');
  });
});
