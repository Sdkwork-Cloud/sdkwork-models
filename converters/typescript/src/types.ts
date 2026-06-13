/** 协议枚举 */
export enum Protocol {
  OpenAiResponses = 'openai_responses',
  OpenAiCompletions = 'openai_completions',
  AnthropicMessages = 'anthropic_messages',
  GoogleGemini = 'google_gemini',
  OpenAiCompatible = 'openai_compatible',
  VendorNative = 'vendor_native',
}

/** 能力枚举 */
export enum Capability {
  Stream = 'stream',
  Tools = 'tools',
  Vision = 'vision',
  Audio = 'audio',
  Video = 'video',
  Image = 'image',
  Code = 'code',
  Reasoning = 'reasoning',
}

/** 角色枚举 */
export enum Role {
  System = 'system',
  User = 'user',
  Assistant = 'assistant',
  Tool = 'tool',
}

/** 停止原因枚举 */
export enum StopReason {
  EndTurn = 'end_turn',
  StopSequence = 'stop_sequence',
  MaxTokens = 'max_tokens',
  ToolUse = 'tool_use',
  Stop = 'stop',
  Length = 'length',
  ContentFilter = 'content_filter',
}

/** 工具类型枚举 */
export enum ToolType {
  Function = 'function',
}

/** 转换请求 */
export interface ConversionRequest {
  protocol: Protocol;
  model: string;
  messages: Message[];
  maxTokens?: number;
  temperature?: number;
  topP?: number;
  stream: boolean;
  tools?: Tool[];
  system?: string;
  metadata: Record<string, unknown>;
}

/** 消息 */
export interface Message {
  role: Role;
  content: Content;
}

/** 内容类型 */
export type Content = string | ContentPart[];

/** 内容部分 */
export type ContentPart =
  | { type: 'text'; text: string }
  | { type: 'image_url'; image_url: { url: string; detail?: string } }
  | { type: 'image'; source: { type: string; media_type: string; data: string } }
  | { type: 'tool_use'; id: string; name: string; input: unknown }
  | { type: 'tool_result'; tool_use_id: string; content: string | ContentPart[] };

/** 工具定义 */
export interface Tool {
  type: ToolType;
  function: FunctionDefinition;
}

/** 函数定义 */
export interface FunctionDefinition {
  name: string;
  description?: string;
  parameters?: unknown;
  input_schema?: unknown;
}

/** 转换响应 */
export interface ConversionResponse {
  protocol: Protocol;
  id: string;
  model: string;
  content: ContentPart[];
  stopReason?: StopReason;
  usage: Usage;
  metadata: Record<string, unknown>;
}

/** Token使用统计 */
export interface Usage {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  cacheCreationInputTokens?: number;
  cacheReadInputTokens?: number;
}

/** 模型映射配置 */
export interface ModelMapping {
  mapping: Record<string, string>;
  wildcardRules?: { pattern: string; target: string }[];
}

/** 转换器配置 */
export interface ConverterConfig {
  name: string;
  sourceProtocol: Protocol;
  targetProtocol: Protocol;
  modelMapping: ModelMapping;
  capabilities: Capability[];
  options?: Record<string, unknown>;
}
