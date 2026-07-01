> Owner: SDKWork maintainers

## �?述

Plugins�?系�?��?�?�?�不�?客�?�端工�?��?�?��??协议转换�??模�??�?��?�??每个vendor可以�?�?�?�己�??转换�?�plugin�?�?��?��?vendor�??�??�??API转换为�?种客�?�端工�?��??�??�??格式�??
## �?��?�?�??

```
plugins/
�??�??�?? README.md                    # �?件�?系说�??
�??�??�?? manifest.json                # �?件注�??�?�??�??�?? core/                        # 核�?�?件�?�?�
�??  �??�??�?? base-converter.ts       # 转换�?��?��?�??  �??�??�?? base-mapper.ts          # �?��?�?��?��?�??  �??�??�?? types.ts                # 类�??�?�?
�??  �??�??�?? registry.ts             # �?件注�??�?�??�??�?? converters/                  # 协议转换�??�??  �??�??�?? anthropic-to-openai/    # Anthropic �??OpenAI
�??  �??  �??�??�?? index.ts
�??  �??  �??�??�?? request.ts
�??  �??  �??�??�?? response.ts
�??  �??  �??�??�?? tests/
�??  �??�??�?? openai-to-anthropic/    # OpenAI �??Anthropic
�??  �??  �??�??�?? index.ts
�??  �??  �??�??�?? request.ts
�??  �??  �??�??�?? response.ts
�??  �??  �??�??�?? tests/
�??  �??�??�?? google-to-openai/       # Google �??OpenAI
�??  �??  �??�??�?? index.ts
�??  �??  �??�??�?? request.ts
�??  �??  �??�??�?? response.ts
�??  �??  �??�??�?? tests/
�??  �??�??�?? custom/                 # �?��?�?转换�?�
�??      �??�??�?? vendor-specific/
�??�??�?? mappers/                     # 模�??�?��?�??�??  �??�??�?? direct-mapper.ts        # �?��?��?��?
�??  �??�??�?? prefix-mapper.ts        # �?��?�?��?
�??  �??�??�?? suffix-mapper.ts        # �?�?�?��?
�??  �??�??�?? custom-mapper.ts        # �?��?�?�?��?�??�??�?? vendors/                     # Vendor�?件
�??  �??�??�?? openai/
�??  �??  �??�??�?? plugin.json
�??  �??  �??�??�?? converter.ts
�??  �??  �??�??�?? mapper.ts
�??  �??  �??�??�?? tests/
�??  �??�??�?? anthropic/
�??  �??  �??�??�?? plugin.json
�??  �??  �??�??�?? converter.ts
�??  �??  �??�??�?? mapper.ts
�??  �??  �??�??�?? tests/
�??  �??�??�?? alibaba/
�??  �??  �??�??�?? plugin.json
�??  �??  �??�??�?? converter.ts
�??  �??  �??�??�?? mapper.ts
�??  �??  �??�??�?? tests/
�??  �??�??�?? deepseek/
�??      �??�??�?? plugin.json
�??      �??�??�?? converter.ts
�??      �??�??�?? mapper.ts
�??      �??�??�?? tests/
�??�??�?? tools/                       # 客�?�端工�?��?�?    �??�??�?? claude-code/
    �??  �??�??�?? plugin.json
    �??  �??�??�?? adapter.ts
    �??  �??�??�?? tests/
    �??�??�?? codex/
    �??  �??�??�?? plugin.json
    �??  �??�??�?? adapter.ts
    �??  �??�??�?? tests/
    �??�??�?? gemini-cli/
    �??  �??�??�?? plugin.json
    �??  �??�??�?? adapter.ts
    �??  �??�??�?? tests/
    �??�??�?? cursor/
        �??�??�?? plugin.json
        �??�??�?? adapter.ts
        �??�??�?? tests/
```

## 核�?类�??�?�?

### types.ts

```typescript
// �?��?��?��??export type SupportStatus = 'native' | 'convert' | 'none';

// 协议代码
export type ProtocolCode = 
  | 'anthropic_messages'
  | 'openai_responses'
  | 'openai_compatible'
  | 'google_gemini'
  | 'vendor_native';

// �?��??代码
export type CapabilityCode = 
  | 'stream'
  | 'tools'
  | 'vision'
  | 'audio'
  | 'video'
  | 'image'
  | 'music'
  | 'code'
  | 'reasoning';

// API代码
export type ApiCode = 
  | 'claude_code'
  | 'codex'
  | 'gemini_cli'
  | 'cursor'
  | 'copilot'
  | 'cline'
  | 'aider';

// �?��??代码
export type RegionCode = 
  | 'global'
  | 'cn'
  | 'us'
  | 'eu'
  | 'asia';

// 模�??�?��?�?�??
export interface ModelMapping {
  source: string;
  target: string;
  transform?: 'none' | 'prefix' | 'suffix' | 'custom';
  config?: Record<string, any>;
}

// 转换�?�置
export interface ConvertConfig {
  from: ProtocolCode;
  map: Record<string, string>;
  rules?: TransformRule[];
}

// 转换�?�??
export interface TransformRule {
  type: 'request' | 'response';
  field: string;
  action: 'rename' | 'transform' | 'remove' | 'add';
  source?: string;
  target?: string;
  transform?: (value: any) => any;
}

// 客�?�端API�?�置
export interface ClientApiConfig {
  status: SupportStatus;
  version?: string;
  protocol: ProtocolCode;
  endpoint?: string;
  convert?: ConvertConfig;
  caps?: CapabilityCode[];
  regions?: RegionCode[];
  note?: string;
  meta?: Record<string, any>;
}

// Vendor�?�置
export interface VendorConfig {
  vendorCode: string;
  displayName: string;
  clientApi: Record<ApiCode, ClientApiConfig>;
}

// 转换�?��?��?export interface IConverter {
  readonly name: string;
  readonly sourceProtocol: ProtocolCode;
  readonly targetProtocol: ProtocolCode;
  
  canConvert(source: ProtocolCode, target: ProtocolCode): boolean;
  convertRequest(request: any): any;
  convertResponse(response: any): any;
}

// �?��?�?��?��?export interface IMapper {
  readonly name: string;
  
  map(sourceModel: string, mapping: Record<string, string>): string;
  mapBatch(models: string[], mapping: Record<string, string>): string[];
}

// �?件�?�口
export interface IPlugin {
  readonly name: string;
  readonly version: string;
  readonly vendorCode?: string;
  readonly apiCode?: ApiCode;
  
  initialize(): Promise<void>;
  getConverter(): IConverter;
  getMapper(): IMapper;
  getConfig(): ClientApiConfig;
}

// �?件注�??�?export interface PluginRegistry {
  register(plugin: IPlugin): void;
  unregister(name: string): void;
  get(name: string): IPlugin | undefined;
  getByVendor(vendorCode: string): IPlugin[];
  getByApi(apiCode: ApiCode): IPlugin[];
  list(): IPlugin[];
}
```

## �?件�?�置格式

### plugin.json

```json
{
  "name": "alibaba-claude-code",
  "version": "1.0.0",
  "description": "Alibaba Cloud Claude Code转换�??,
  "vendorCode": "alibaba",
  "apiCode": "claude_code",
  "author": "SDKWork",
  "license": "MIT",
  "main": "./converter.ts",
  "dependencies": {
    "core": "^1.0.0"
  },
  "config": {
    "sourceProtocol": "anthropic_messages",
    "targetProtocol": "anthropic_messages",
    "supportedRegions": ["global"],
    "modelMapping": {
      "qwen3.7-max": "claude-sonnet-4",
      "qwen3.7-turbo": "claude-haiku-4"
    },
    "capabilities": ["stream", "tools"],
    "limitations": [
      "�?�?��?�global�?��??",
      "�?��??�?级�??�?�可�?�不�?�?��?��?
    ]
  },
  "transforms": {
    "request": [
      {
        "field": "model",
        "action": "transform",
        "transform": "mapModel"
      }
    ],
    "response": [
      {
        "field": "model",
        "action": "transform",
        "transform": "reverseMapModel"
      }
    ]
  }
}
```

## 转换�?��?�?�示�?
### Alibaba Claude Code Converter

```typescript
// plugins/vendors/alibaba/converter.ts
import { IConverter, ProtocolCode, TransformRule } from '../../core/types';

export class AlibabaClaudeCodeConverter implements IConverter {
  readonly name = 'alibaba-claude-code';
  readonly sourceProtocol: ProtocolCode = 'anthropic_messages';
  readonly targetProtocol: ProtocolCode = 'anthropic_messages';
  
  private modelMapping: Record<string, string> = {
    'qwen3.7-max': 'claude-sonnet-4',
    'qwen3.7-turbo': 'claude-haiku-4'
  };
  
  canConvert(source: ProtocolCode, target: ProtocolCode): boolean {
    return source === this.sourceProtocol && target === this.targetProtocol;
  }
  
  convertRequest(request: any): any {
    const converted = { ...request };
    
    // �?��?模�??名称
    if (converted.model && this.modelMapping[converted.model]) {
      converted.model = this.modelMapping[converted.model];
    }
    
    // 转换系�?�?息格式
    if (converted.system && typeof converted.system === 'string') {
      converted.system = [{ type: 'text', text: converted.system }];
    }
    
    return converted;
  }
  
  convertResponse(response: any): any {
    const converted = { ...response };
    
    // 反�?�?��?模�??名称
    const reverseMapping = Object.fromEntries(
      Object.entries(this.modelMapping).map(([k, v]) => [v, k])
    );
    
    if (converted.model && reverseMapping[converted.model]) {
      converted.model = reverseMapping[converted.model];
    }
    
    return converted;
  }
}
```

### DeepSeek Claude Code Converter (�??�?OpenAI�?�容)

```typescript
// plugins/vendors/deepseek/converter.ts
import { IConverter, ProtocolCode } from '../../core/types';

export class DeepSeekClaudeCodeConverter implements IConverter {
  readonly name = 'deepseek-claude-code';
  readonly sourceProtocol: ProtocolCode = 'openai_compatible';
  readonly targetProtocol: ProtocolCode = 'anthropic_messages';
  
  private modelMapping: Record<string, string> = {
    'deepseek-v4-pro': 'claude-sonnet-4',
    'deepseek-v4-flash': 'claude-haiku-4'
  };
  
  canConvert(source: ProtocolCode, target: ProtocolCode): boolean {
    return source === this.sourceProtocol && target === this.targetProtocol;
  }
  
  convertRequest(request: any): any {
    // OpenAI格式 �??Anthropic格式
    return {
      model: this.modelMapping[request.model] || request.model,
      max_tokens: request.max_tokens || 4096,
      system: this.convertSystemMessage(request.messages),
      messages: this.convertMessages(request.messages),
      tools: this.convertTools(request.tools),
      stream: request.stream || false
    };
  }
  
  convertResponse(response: any): any {
    // Anthropic格式 �??OpenAI格式
    return {
      id: response.id,
      object: 'chat.completion',
      created: Math.floor(Date.now() / 1000),
      model: this.reverseMapModel(response.model),
      choices: [{
        index: 0,
        message: {
          role: 'assistant',
          content: this.extractContent(response.content),
          tool_calls: this.extractToolCalls(response.content)
        },
        finish_reason: response.stop_reason
      }],
      usage: response.usage
    };
  }
  
  private convertSystemMessage(messages: any[]): any[] {
    const systemMsg = messages.find(m => m.role === 'system');
    return systemMsg ? [{ type: 'text', text: systemMsg.content }] : [];
  }
  
  private convertMessages(messages: any[]): any[] {
    return messages
      .filter(m => m.role !== 'system')
      .map(m => ({
        role: m.role,
        content: m.content
      }));
  }
  
  private convertTools(tools: any[]): any[] {
    if (!tools) return [];
    return tools.map(tool => ({
      name: tool.function.name,
      description: tool.function.description,
      input_schema: tool.function.parameters
    }));
  }
  
  private extractContent(content: any[]): string {
    return content
      .filter(c => c.type === 'text')
      .map(c => c.text)
      .join('');
  }
  
  private extractToolCalls(content: any[]): any[] {
    return content
      .filter(c => c.type === 'tool_use')
      .map(c => ({
        id: c.id,
        type: 'function',
        function: {
          name: c.name,
          arguments: JSON.stringify(c.input)
        }
      }));
  }
  
  private reverseMapModel(model: string): string {
    const reverseMapping = Object.fromEntries(
      Object.entries(this.modelMapping).map(([k, v]) => [v, k])
    );
    return reverseMapping[model] || model;
  }
}
```

## �?��?�?��?�?�示�?
### Custom Mapper

```typescript
// plugins/mappers/custom-mapper.ts
import { IMapper, ModelMapping } from '../core/types';

export class CustomMapper implements IMapper {
  readonly name = 'custom-mapper';
  
  map(sourceModel: string, mapping: Record<string, string>): string {
    return mapping[sourceModel] || sourceModel;
  }
  
  mapBatch(models: string[], mapping: Record<string, string>): string[] {
    return models.map(model => this.map(model, mapping));
  }
  
  // �?��?��??�?�符�?��??  mapWithWildcard(sourceModel: string, mapping: Record<string, string>): string {
    // 精确�?��?�
    if (mapping[sourceModel]) {
      return mapping[sourceModel];
    }
    
    // �??�?�符�?��??    for (const [pattern, target] of Object.entries(mapping)) {
      if (pattern.includes('*')) {
        const regex = new RegExp('^' + pattern.replace(/\*/g, '.*') + '$');
        if (regex.test(sourceModel)) {
          return target.replace('$1', sourceModel);
        }
      }
    }
    
    return sourceModel;
  }
}
```

## Vendor�?件�?�置示�?

### Alibaba Plugin

```json
{
  "name": "alibaba-plugin",
  "version": "1.0.0",
  "vendorCode": "alibaba",
  "clientApi": {
    "claude_code": {
      "status": "convert",
      "protocol": "anthropic_messages",
      "endpoint": "/v1/messages",
      "convert": {
        "from": "anthropic_messages",
        "map": {
          "qwen3.7-max": "claude-sonnet-4",
          "qwen3.7-turbo": "claude-haiku-4"
        }
      },
      "caps": ["stream", "tools"],
      "regions": ["global"],
      "note": "Qwen3.7-Max�?�容Anthropic格式"
    },
    "codex": {
      "status": "none",
      "protocol": "openai_compatible",
      "note": "不�?��?�Codex客�?�端API"
    },
    "gemini_cli": {
      "status": "none",
      "protocol": "google_gemini",
      "note": "不�?��?�Gemini CLI客�?�端API"
    }
  }
}
```

### DeepSeek Plugin

```json
{
  "name": "deepseek-plugin",
  "version": "1.0.0",
  "vendorCode": "deepseek",
  "clientApi": {
    "claude_code": {
      "status": "convert",
      "protocol": "openai_compatible",
      "endpoint": "/v1/chat/completions",
      "convert": {
        "from": "openai_compatible",
        "map": {
          "deepseek-v4-pro": "claude-sonnet-4",
          "deepseek-v4-flash": "claude-haiku-4"
        }
      },
      "caps": ["stream", "tools", "code"],
      "regions": ["cn", "global"],
      "note": "�??要代�?�?�?�?协议转换"
    },
    "codex": {
      "status": "none",
      "protocol": "openai_compatible",
      "note": "不�?��?�Codex客�?�端API"
    },
    "gemini_cli": {
      "status": "none",
      "protocol": "google_gemini",
      "note": "不�?��?�Gemini CLI客�?�端API"
    }
  }
}
```

## 工�?��?件�?�置示�?

### Claude Code Tool Plugin

```json
{
  "name": "claude-code-tool",
  "version": "1.0.0",
  "apiCode": "claude_code",
  "description": "Claude Code客�?�端工�?��??�?��??,
  "supportedProtocols": ["anthropic_messages"],
  "requiredCapabilities": ["stream", "tools"],
  "modelRequirements": {
    "minContextLength": 200000,
    "requiredCapabilities": ["stream", "tools"]
  },
  "adapter": {
    "endpoint": "/v1/messages",
    "headers": {
      "anthropic-version": "2023-06-01",
      "content-type": "application/json"
    },
    "requestFormat": "anthropic_messages",
    "responseFormat": "anthropic_messages"
  }
}
```

## 使�?�示�?

### 注�??�?件

```typescript
import { PluginRegistry } from './core/registry';
import { AlibabaClaudeCodeConverter } from './vendors/alibaba/converter';
import { CustomMapper } from './mappers/custom-mapper';

const registry = new PluginRegistry();

// 注�??Alibaba�?件
registry.register({
  name: 'alibaba-claude-code',
  version: '1.0.0',
  vendorCode: 'alibaba',
  apiCode: 'claude_code',
  
  async initialize() {
    console.log('Alibaba Claude Code plugin initialized');
  },
  
  getConverter() {
    return new AlibabaClaudeCodeConverter();
  },
  
  getMapper() {
    return new CustomMapper();
  },
  
  getConfig() {
    return {
      status: 'convert',
      protocol: 'anthropic_messages',
      endpoint: '/v1/messages',
      convert: {
        from: 'anthropic_messages',
        map: {
          'qwen3.7-max': 'claude-sonnet-4',
          'qwen3.7-turbo': 'claude-haiku-4'
        }
      },
      caps: ['stream', 'tools'],
      regions: ['global'],
      note: 'Qwen3.7-Max�?�容Anthropic格式'
    };
  }
});
```

### 使�?�转换�??
```typescript
import { AlibabaClaudeCodeConverter } from './vendors/alibaba/converter';

const converter = new AlibabaClaudeCodeConverter();

// 转换请�?
const openaiRequest = {
  model: 'qwen3.7-max',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Hello!' }
  ],
  max_tokens: 1024,
  stream: true
};

const anthropicRequest = converter.convertRequest(openaiRequest);
console.log(anthropicRequest);
// �?�?�:
// {
//   model: 'claude-sonnet-4',
//   max_tokens: 1024,
//   system: [{ type: 'text', text: 'You are a helpful assistant.' }],
//   messages: [{ role: 'user', content: 'Hello!' }],
//   stream: true
// }
```

## �?证�?�??

### �?件�?证

1. **�?填�?段**�?name, version, vendorCode/apiCode
2. **�??�?�格式**�?遵循语�?�??�??�?��?�??
3. **依�?�?�??*�?�?�?�依�?�?件�?�否�?�??4. **�?�口�?�?�**�?确保�?�?��??�??�?�??�?�口

### 转换�?��?�?
1. **协议�?�容**�?�?�?�源/�?��?协议�?�否�?�容
2. **模�??�?��?**�?�?证�?��?�?�??�??�??�??�??3. **�?��??�?�??*�?确保转换�?��?��?��??�??�?��??
4. **�?�?�?�??**�?要�?提�?�?�??�?�?
### �?�置�?证

1. **�?��?��??�??*�?status�?须�?�native/convert/none
2. **协议�??�??**�?protocol�?须�?�协议代码表�?3. **�?��??�??�??**�?regions�?须�?��?��??代码表�?4. **�?��??�??�??**�?caps�?须�?��?��??代码表�?
## �?��?�?��?�

### �?��?�?转换�?�

```typescript
// plugins/converters/custom/my-converter.ts
import { IConverter } from '../../core/types';

export class MyCustomConverter implements IConverter {
  readonly name = 'my-custom-converter';
  readonly sourceProtocol = 'vendor_native';
  readonly targetProtocol = 'openai_compatible';
  
  // �?�?��?��?�?转换�?��?
}
```

### �?��?�?�?��?�?�

```typescript
// plugins/mappers/custom/my-mapper.ts
import { IMapper } from '../core/types';

export class MyCustomMapper implements IMapper {
  readonly name = 'my-custom-mapper';
  
  // �?�?��?��?�?�?��?�?��?
}
```

## �?�?�?�??

### �?�??�?�?

```typescript
// plugins/vendors/alibaba/tests/converter.test.ts
import { AlibabaClaudeCodeConverter } from '../converter';

describe('AlibabaClaudeCodeConverter', () => {
  let converter: AlibabaClaudeCodeConverter;
  
  beforeEach(() => {
    converter = new AlibabaClaudeCodeConverter();
  });
  
  test('should convert request correctly', () => {
    const request = {
      model: 'qwen3.7-max',
      messages: [{ role: 'user', content: 'Hello!' }]
    };
    
    const result = converter.convertRequest(request);
    
    expect(result.model).toBe('claude-sonnet-4');
    expect(result.messages).toEqual([{ role: 'user', content: 'Hello!' }]);
  });
  
  test('should convert response correctly', () => {
    const response = {
      model: 'claude-sonnet-4',
      content: [{ type: 'text', text: 'Hello!' }]
    };
    
    const result = converter.convertResponse(response);
    
    expect(result.model).toBe('qwen3.7-max');
  });
});
```

## �?�署�??�?�?
### �??�??

```bash
# �??�??�?件
npm run package:plugin -- --name=alibaba-claude-code

# �?证�?件
npm run validate:plugin -- --name=alibaba-claude-code

# �?�?�?件
npm run publish:plugin -- --name=alibaba-claude-code
```

### �??�?�管�?

```bash
# �?��?��??�?�
npm version patch  # 1.0.0 �??1.0.1
npm version minor  # 1.0.0 �??1.1.0
npm version major  # 1.0.0 �??2.0.0
```

## �??档�??�?�

```bash
# �??�?�API�??档
npm run docs:generate

# �??�?��?件�??档
npm run docs:plugin -- --name=alibaba-claude-code
```

## �??�?��??�?��?
### �?��?��??�?�

```typescript
import { metrics } from './core/metrics';

// 记�?转换�?��?�
metrics.recordConversion('alibaba-claude-code', 'request', duration);

// 记�?�??误
metrics.recordError('alibaba-claude-code', 'validation_error');
```

### �?��?记�?

```typescript
import { logger } from './core/logger';

logger.info('Converting request', {
  plugin: 'alibaba-claude-code',
  sourceModel: 'qwen3.7-max',
  targetModel: 'claude-sonnet-4'
});
```

