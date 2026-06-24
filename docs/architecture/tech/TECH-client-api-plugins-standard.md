> Migrated from `docs/client-api-plugins-standard.md` on 2026-06-24.
> Owner: SDKWork maintainers

## 概述

Plugins体系用于实现不同客户端工具之间的协议转换和模型映射。每个vendor可以定义自己的转换器plugin，支持将vendor的原生API转换为各种客户端工具期望的格式。

## 目录结构

```
plugins/
├── README.md                    # 插件体系说明
├── manifest.json                # 插件注册表
├── core/                        # 核心插件框架
│   ├── base-converter.ts       # 转换器基类
│   ├── base-mapper.ts          # 映射器基类
│   ├── types.ts                # 类型定义
│   └── registry.ts             # 插件注册表
├── converters/                  # 协议转换器
│   ├── anthropic-to-openai/    # Anthropic → OpenAI
│   │   ├── index.ts
│   │   ├── request.ts
│   │   ├── response.ts
│   │   └── tests/
│   ├── openai-to-anthropic/    # OpenAI → Anthropic
│   │   ├── index.ts
│   │   ├── request.ts
│   │   ├── response.ts
│   │   └── tests/
│   ├── google-to-openai/       # Google → OpenAI
│   │   ├── index.ts
│   │   ├── request.ts
│   │   ├── response.ts
│   │   └── tests/
│   └── custom/                 # 自定义转换器
│       └── vendor-specific/
├── mappers/                     # 模型映射器
│   ├── direct-mapper.ts        # 直接映射
│   ├── prefix-mapper.ts        # 前缀映射
│   ├── suffix-mapper.ts        # 后缀映射
│   └── custom-mapper.ts        # 自定义映射
├── vendors/                     # Vendor插件
│   ├── openai/
│   │   ├── plugin.json
│   │   ├── converter.ts
│   │   ├── mapper.ts
│   │   └── tests/
│   ├── anthropic/
│   │   ├── plugin.json
│   │   ├── converter.ts
│   │   ├── mapper.ts
│   │   └── tests/
│   ├── alibaba/
│   │   ├── plugin.json
│   │   ├── converter.ts
│   │   ├── mapper.ts
│   │   └── tests/
│   └── deepseek/
│       ├── plugin.json
│       ├── converter.ts
│       ├── mapper.ts
│       └── tests/
└── tools/                       # 客户端工具插件
    ├── claude-code/
    │   ├── plugin.json
    │   ├── adapter.ts
    │   └── tests/
    ├── codex/
    │   ├── plugin.json
    │   ├── adapter.ts
    │   └── tests/
    ├── gemini-cli/
    │   ├── plugin.json
    │   ├── adapter.ts
    │   └── tests/
    └── cursor/
        ├── plugin.json
        ├── adapter.ts
        └── tests/
```

## 核心类型定义

### types.ts

```typescript
// 支持状态
export type SupportStatus = 'native' | 'convert' | 'none';

// 协议代码
export type ProtocolCode = 
  | 'anthropic_messages'
  | 'openai_responses'
  | 'openai_compatible'
  | 'google_gemini'
  | 'vendor_native';

// 能力代码
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

// 区域代码
export type RegionCode = 
  | 'global'
  | 'cn'
  | 'us'
  | 'eu'
  | 'asia';

// 模型映射规则
export interface ModelMapping {
  source: string;
  target: string;
  transform?: 'none' | 'prefix' | 'suffix' | 'custom';
  config?: Record<string, any>;
}

// 转换配置
export interface ConvertConfig {
  from: ProtocolCode;
  map: Record<string, string>;
  rules?: TransformRule[];
}

// 转换规则
export interface TransformRule {
  type: 'request' | 'response';
  field: string;
  action: 'rename' | 'transform' | 'remove' | 'add';
  source?: string;
  target?: string;
  transform?: (value: any) => any;
}

// 客户端API配置
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

// Vendor配置
export interface VendorConfig {
  vendorCode: string;
  displayName: string;
  clientApi: Record<ApiCode, ClientApiConfig>;
}

// 转换器接口
export interface IConverter {
  readonly name: string;
  readonly sourceProtocol: ProtocolCode;
  readonly targetProtocol: ProtocolCode;
  
  canConvert(source: ProtocolCode, target: ProtocolCode): boolean;
  convertRequest(request: any): any;
  convertResponse(response: any): any;
}

// 映射器接口
export interface IMapper {
  readonly name: string;
  
  map(sourceModel: string, mapping: Record<string, string>): string;
  mapBatch(models: string[], mapping: Record<string, string>): string[];
}

// 插件接口
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

// 插件注册表
export interface PluginRegistry {
  register(plugin: IPlugin): void;
  unregister(name: string): void;
  get(name: string): IPlugin | undefined;
  getByVendor(vendorCode: string): IPlugin[];
  getByApi(apiCode: ApiCode): IPlugin[];
  list(): IPlugin[];
}
```

## 插件配置格式

### plugin.json

```json
{
  "name": "alibaba-claude-code",
  "version": "1.0.0",
  "description": "Alibaba Cloud Claude Code转换器",
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
      "仅支持global区域",
      "部分高级功能可能不完全兼容"
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

## 转换器实现示例

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
    
    // 映射模型名称
    if (converted.model && this.modelMapping[converted.model]) {
      converted.model = this.modelMapping[converted.model];
    }
    
    // 转换系统消息格式
    if (converted.system && typeof converted.system === 'string') {
      converted.system = [{ type: 'text', text: converted.system }];
    }
    
    return converted;
  }
  
  convertResponse(response: any): any {
    const converted = { ...response };
    
    // 反向映射模型名称
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

### DeepSeek Claude Code Converter (通过OpenAI兼容)

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
    // OpenAI格式 → Anthropic格式
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
    // Anthropic格式 → OpenAI格式
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

## 映射器实现示例

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
  
  // 支持通配符匹配
  mapWithWildcard(sourceModel: string, mapping: Record<string, string>): string {
    // 精确匹配
    if (mapping[sourceModel]) {
      return mapping[sourceModel];
    }
    
    // 通配符匹配
    for (const [pattern, target] of Object.entries(mapping)) {
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

## Vendor插件配置示例

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
      "note": "Qwen3.7-Max兼容Anthropic格式"
    },
    "codex": {
      "status": "none",
      "protocol": "openai_compatible",
      "note": "不支持Codex客户端API"
    },
    "gemini_cli": {
      "status": "none",
      "protocol": "google_gemini",
      "note": "不支持Gemini CLI客户端API"
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
      "note": "需要代理层进行协议转换"
    },
    "codex": {
      "status": "none",
      "protocol": "openai_compatible",
      "note": "不支持Codex客户端API"
    },
    "gemini_cli": {
      "status": "none",
      "protocol": "google_gemini",
      "note": "不支持Gemini CLI客户端API"
    }
  }
}
```

## 工具插件配置示例

### Claude Code Tool Plugin

```json
{
  "name": "claude-code-tool",
  "version": "1.0.0",
  "apiCode": "claude_code",
  "description": "Claude Code客户端工具适配器",
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

## 使用示例

### 注册插件

```typescript
import { PluginRegistry } from './core/registry';
import { AlibabaClaudeCodeConverter } from './vendors/alibaba/converter';
import { CustomMapper } from './mappers/custom-mapper';

const registry = new PluginRegistry();

// 注册Alibaba插件
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
      note: 'Qwen3.7-Max兼容Anthropic格式'
    };
  }
});
```

### 使用转换器

```typescript
import { AlibabaClaudeCodeConverter } from './vendors/alibaba/converter';

const converter = new AlibabaClaudeCodeConverter();

// 转换请求
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
// 输出:
// {
//   model: 'claude-sonnet-4',
//   max_tokens: 1024,
//   system: [{ type: 'text', text: 'You are a helpful assistant.' }],
//   messages: [{ role: 'user', content: 'Hello!' }],
//   stream: true
// }
```

## 验证规则

### 插件验证

1. **必填字段**：name, version, vendorCode/apiCode
2. **版本格式**：遵循语义化版本规范
3. **依赖检查**：检查依赖插件是否存在
4. **接口实现**：确保实现所有必需接口

### 转换器验证

1. **协议兼容**：检查源/目标协议是否兼容
2. **模型映射**：验证映射规则的有效性
3. **能力检查**：确保转换器支持所需能力
4. **测试覆盖**：要求提供单元测试

### 配置验证

1. **状态有效**：status必须是native/convert/none
2. **协议有效**：protocol必须在协议代码表中
3. **区域有效**：regions必须在区域代码表中
4. **能力有效**：caps必须在能力代码表中

## 扩展机制

### 自定义转换器

```typescript
// plugins/converters/custom/my-converter.ts
import { IConverter } from '../../core/types';

export class MyCustomConverter implements IConverter {
  readonly name = 'my-custom-converter';
  readonly sourceProtocol = 'vendor_native';
  readonly targetProtocol = 'openai_compatible';
  
  // 实现自定义转换逻辑
}
```

### 自定义映射器

```typescript
// plugins/mappers/custom/my-mapper.ts
import { IMapper } from '../core/types';

export class MyCustomMapper implements IMapper {
  readonly name = 'my-custom-mapper';
  
  // 实现自定义映射逻辑
}
```

## 测试规范

### 单元测试

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

## 部署和发布

### 打包

```bash
# 打包插件
npm run package:plugin -- --name=alibaba-claude-code

# 验证插件
npm run validate:plugin -- --name=alibaba-claude-code

# 发布插件
npm run publish:plugin -- --name=alibaba-claude-code
```

### 版本管理

```bash
# 更新版本
npm version patch  # 1.0.0 → 1.0.1
npm version minor  # 1.0.0 → 1.1.0
npm version major  # 1.0.0 → 2.0.0
```

## 文档生成

```bash
# 生成API文档
npm run docs:generate

# 生成插件文档
npm run docs:plugin -- --name=alibaba-claude-code
```

## 监控和日志

### 性能监控

```typescript
import { metrics } from './core/metrics';

// 记录转换时间
metrics.recordConversion('alibaba-claude-code', 'request', duration);

// 记录错误
metrics.recordError('alibaba-claude-code', 'validation_error');
```

### 日志记录

```typescript
import { logger } from './core/logger';

logger.info('Converting request', {
  plugin: 'alibaba-claude-code',
  sourceModel: 'qwen3.7-max',
  targetModel: 'claude-sonnet-4'
});
```

