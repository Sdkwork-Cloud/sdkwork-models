import type { Converter } from './traits';
import type { Protocol } from './types';

/** 转换器注册表 */
export class ConverterRegistry {
  private converters = new Map<string, Converter>();

  /** 注册转换器 */
  register(converter: Converter): void {
    this.converters.set(converter.name(), converter);
  }

  /** 注销转换器 */
  unregister(name: string): boolean {
    return this.converters.delete(name);
  }

  /** 获取转换器 */
  get(name: string): Converter | undefined {
    return this.converters.get(name);
  }

  /** 按协议对查找转换器 */
  find(source: Protocol, target: Protocol): Converter | undefined {
    for (const converter of this.converters.values()) {
      if (converter.canConvert(source, target)) {
        return converter;
      }
    }
    return undefined;
  }

  /** 列出所有已注册的转换器名称 */
  list(): string[] {
    return Array.from(this.converters.keys());
  }

  /** 按源协议筛选 */
  bySourceProtocol(protocol: Protocol): Converter[] {
    return Array.from(this.converters.values()).filter(
      (c) => c.sourceProtocol() === protocol
    );
  }

  /** 按目标协议筛选 */
  byTargetProtocol(protocol: Protocol): Converter[] {
    return Array.from(this.converters.values()).filter(
      (c) => c.targetProtocol() === protocol
    );
  }

  /** 注册所有内置转换器 */
  registerDefaults(): void {
    // OpenAI Responses →
    const { OpenAiResponsesToAnthropicMessagesConverter } = require('./openai-responses-to-anthropic-messages');
    const { OpenAiResponsesToOpenAiCompletionsConverter } = require('./openai-responses-to-openai-completions');
    const { OpenAiResponsesToGoogleGeminiConverter } = require('./openai-responses-to-google-gemini');

    // OpenAI Completions →
    const { OpenAiCompletionsToAnthropicMessagesConverter } = require('./openai-completions-to-anthropic-messages');
    const { OpenAiCompletionsToOpenAiResponsesConverter } = require('./openai-completions-to-openai-responses');
    const { OpenAiCompletionsToGoogleGeminiConverter } = require('./openai-completions-to-google-gemini');

    // Anthropic Messages →
    const { AnthropicMessagesToOpenAiResponsesConverter } = require('./anthropic-messages-to-openai-responses');
    const { AnthropicMessagesToOpenAiCompletionsConverter } = require('./anthropic-messages-to-openai-completions');
    const { AnthropicMessagesToGoogleGeminiConverter } = require('./anthropic-messages-to-google-gemini');

    // Google Gemini →
    const { GoogleGeminiToAnthropicMessagesConverter } = require('./google-gemini-to-anthropic-messages');
    const { GoogleGeminiToOpenAiResponsesConverter } = require('./google-gemini-to-openai-responses');
    const { GoogleGeminiToOpenAiCompletionsConverter } = require('./google-gemini-to-openai-completions');

    this.register(new OpenAiResponsesToAnthropicMessagesConverter());
    this.register(new OpenAiResponsesToOpenAiCompletionsConverter());
    this.register(new OpenAiResponsesToGoogleGeminiConverter());
    this.register(new OpenAiCompletionsToAnthropicMessagesConverter());
    this.register(new OpenAiCompletionsToOpenAiResponsesConverter());
    this.register(new OpenAiCompletionsToGoogleGeminiConverter());
    this.register(new AnthropicMessagesToOpenAiResponsesConverter());
    this.register(new AnthropicMessagesToOpenAiCompletionsConverter());
    this.register(new AnthropicMessagesToGoogleGeminiConverter());
    this.register(new GoogleGeminiToAnthropicMessagesConverter());
    this.register(new GoogleGeminiToOpenAiResponsesConverter());
    this.register(new GoogleGeminiToOpenAiCompletionsConverter());
  }
}
