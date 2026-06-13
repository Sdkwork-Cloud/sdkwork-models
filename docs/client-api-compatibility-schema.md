# Client API Compatibility Schema Design

## 当前问题

1. 缺少明确的转换/映射配置
2. 不支持时没有提供替代方案
3. 缺少模型级别的映射配置
4. 结构不够通用，难以扩展

## 新的通用数据结构设计

### 1. Vendor级别配置

```json
{
  "clientApiCompatibility": {
    "claude_code": {
      "supportStatus": "supported|partial|unsupported",
      "nativeSupport": {
        "enabled": true|false,
        "protocol": "anthropic_messages",
        "endpoint": "/v1/messages",
        "apiVersion": "2023-06-01"
      },
      "conversion": {
        "enabled": true|false,
        "strategy": "direct|proxy|transform",
        "protocol": "openai_compatible",
        "endpoint": "/v1/chat/completions",
        "transformRules": {
          "requestTransform": "anthropic_to_openai",
          "responseTransform": "openai_to_anthropic",
          "systemPromptFormat": "anthropic",
          "toolCallFormat": "anthropic"
        }
      },
      "modelMapping": {
        "enabled": true|false,
        "mappingType": "direct|prefix|suffix|custom",
        "rules": [
          {
            "sourceModel": "vendor-model-id",
            "targetModel": "client-expected-model-id",
            "transform": "none|prefix|suffix|custom"
          }
        ]
      },
      "capabilities": {
        "streaming": true|false,
        "tools": true|false,
        "vision": true|false,
        "audio": true|false
      },
      "limitations": [
        "不支持system消息",
        "不支持tool_choice参数"
      ],
      "source": {
        "observedAt": "2026-06-13T00:00:00Z",
        "sourceUrl": "https://..."
      }
    }
  }
}
```

### 2. 示例配置

#### OpenAI Codex (原生支持)
```json
{
  "codex": {
    "supportStatus": "supported",
    "nativeSupport": {
      "enabled": true,
      "protocol": "openai_responses",
      "endpoint": "/v1/responses",
      "apiVersion": "2024-10-01"
    },
    "conversion": {
      "enabled": false
    },
    "modelMapping": {
      "enabled": false
    },
    "capabilities": {
      "streaming": true,
      "tools": true,
      "vision": true,
      "audio": false
    },
    "limitations": []
  }
}
```

#### Alibaba Claude Code (转换支持)
```json
{
  "claude_code": {
    "supportStatus": "partial",
    "nativeSupport": {
      "enabled": false
    },
    "conversion": {
      "enabled": true,
      "strategy": "transform",
      "protocol": "anthropic_messages",
      "endpoint": "/v1/messages",
      "transformRules": {
        "requestTransform": "passthrough",
        "responseTransform": "passthrough",
        "systemPromptFormat": "anthropic",
        "toolCallFormat": "anthropic"
      }
    },
    "modelMapping": {
      "enabled": true,
      "mappingType": "direct",
      "rules": [
        {
          "sourceModel": "qwen3.7-max",
          "targetModel": "claude-sonnet-4-20250514",
          "transform": "none"
        }
      ]
    },
    "capabilities": {
      "streaming": true,
      "tools": true,
      "vision": false,
      "audio": false
    },
    "limitations": [
      "仅支持global区域",
      "部分高级功能可能不完全兼容"
    ]
  }
}
```

#### DeepSeek Claude Code (OpenAI兼容转换)
```json
{
  "claude_code": {
    "supportStatus": "unsupported",
    "nativeSupport": {
      "enabled": false
    },
    "conversion": {
      "enabled": true,
      "strategy": "proxy",
      "protocol": "openai_compatible",
      "endpoint": "/v1/chat/completions",
      "transformRules": {
        "requestTransform": "anthropic_to_openai",
        "responseTransform": "openai_to_anthropic",
        "systemPromptFormat": "openai",
        "toolCallFormat": "openai"
      }
    },
    "modelMapping": {
      "enabled": true,
      "mappingType": "custom",
      "rules": [
        {
          "sourceModel": "deepseek-v4-pro",
          "targetModel": "claude-sonnet-4-20250514",
          "transform": "custom",
          "transformConfig": {
            "maxTokens": 8192,
            "temperature": 0.7,
            "systemPrompt": "You are Claude, an AI assistant made by Anthropic."
          }
        },
        {
          "sourceModel": "deepseek-v4-flash",
          "targetModel": "claude-haiku-4-20250414",
          "transform": "custom",
          "transformConfig": {
            "maxTokens": 4096,
            "temperature": 0.5
          }
        }
      ]
    },
    "capabilities": {
      "streaming": true,
      "tools": true,
      "vision": false,
      "audio": false
    },
    "limitations": [
      "需要额外的代理层进行协议转换",
      "部分Claude特有功能可能不支持"
    ]
  }
}
```

### 3. 模型映射类型

| 映射类型 | 说明 | 示例 |
|----------|------|------|
| direct | 直接映射，模型名不变 | qwen3.7-max → qwen3.7-max |
| prefix | 添加前缀 | v4-pro → deepseek-v4-pro |
| suffix | 添加后缀 | claude → claude-sonnet |
| custom | 自定义映射规则 | deepseek-v4-pro → claude-sonnet-4-20250514 |

### 4. 转换策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| direct | 直接使用原生API | 原生支持的vendor |
| proxy | 通过代理层转换 | 协议不兼容但功能相似 |
| transform | 完全转换请求/响应 | 协议差异较大 |

### 5. 协议转换规则

| 转换规则 | 说明 |
|----------|------|
| passthrough | 直接透传，不转换 |
| anthropic_to_openai | Anthropic格式 → OpenAI格式 |
| openai_to_anthropic | OpenAI格式 → Anthropic格式 |
| google_to_openai | Google格式 → OpenAI格式 |
| openai_to_google | OpenAI格式 → Google格式 |

## 实施计划

1. 更新schema定义文件
2. 更新vendors.json中的clientApiCompatibility结构
3. 添加模型映射配置
4. 验证新的数据结构
5. 更新文档
