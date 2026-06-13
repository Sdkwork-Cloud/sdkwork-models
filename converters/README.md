# SDKWork Client API Converters

多语言协议转换器库，支持在不同AI客户端API之间进行转换。

## 目录结构

```
converters/
├── README.md                    # 本文件
├── spec/                        # 跨语言共享规范
│   ├── protocols.md            # 协议定义
│   ├── naming.md               # 命名规范
│   ├── types.md                # 类型规范
│   └── test-cases/             # 共享测试用例（JSON）
│
├── rust/                        # Rust实现
├── typescript/                  # TypeScript/Node.js实现
├── python/                      # Python实现
├── java/                        # Java实现
├── go/                          # Go实现
├── swift/                       # Swift实现
├── csharp/                      # C#/.NET实现
├── kotlin/                      # Kotlin实现
├── ruby/                        # Ruby实现
└── php/                         # PHP实现
```

## 支持的转换

| 转换器名称 | 源协议 | 目标协议 | 说明 |
|------------|--------|----------|------|
| OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES | OpenAI Responses | Anthropic Messages | Codex → Claude Code |
| OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES | OpenAI Completions | Anthropic Messages | DeepSeek → Claude Code |
| ANTHROPIC_MESSAGES_TO_OPENAI_RESPONSES | Anthropic Messages | OpenAI Responses | Claude Code → Codex |
| ANTHROPIC_MESSAGES_TO_OPENAI_COMPLETIONS | Anthropic Messages | OpenAI Completions | Claude Code → DeepSeek |

## 语言实现状态

| 语言 | 模块名 | 状态 | 测试 |
|------|--------|------|------|
| Rust | converters-rust | ✅ 完成 | 15/15 |
| TypeScript | converters-typescript | ✅ 完成 | 12/12 |
| Python | converters-python | ✅ 完成 | 12/12 |
| Java | converters-java | 🚧 开发中 | - |
| Go | converters-go | 🚧 开发中 | - |
| Swift | converters-swift | 📋 计划中 | - |
| C# | converters-csharp | 📋 计划中 | - |
| Kotlin | converters-kotlin | 📋 计划中 | - |
| Ruby | converters-ruby | 📋 计划中 | - |
| PHP | converters-php | 📋 计划中 | - |

## 快速开始

### Rust

```rust
use converters_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ConverterError> {
    let mut registry = ConverterRegistry::new();
    registry.register_defaults();
    
    let converter = registry.get("OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES").unwrap();
    // ...
}
```

### TypeScript/Node.js

```typescript
import { ConverterRegistry } from '@sdkwork/converters';

const registry = new ConverterRegistry();
registry.registerDefaults();

const converter = registry.get('OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES');
```

### Python

```python
from sdkwork_converters import ConverterRegistry

registry = ConverterRegistry()
registry.register_defaults()

converter = registry.get("OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES")
```

## 开发指南

### 添加新的转换器

1. 在 `spec/test-cases/` 中添加测试用例JSON
2. 在每种语言实现中添加转换器
3. 运行测试确保所有语言行为一致

### 添加新的语言支持

1. 创建新的语言目录 `converters-<language>/`
2. 实现核心接口：Converter, Mapper, Registry
3. 使用 `spec/test-cases/` 中的测试用例验证

## 规范文档

- [协议定义](spec/protocols.md)
- [命名规范](spec/naming.md)
- [类型规范](spec/types.md)
