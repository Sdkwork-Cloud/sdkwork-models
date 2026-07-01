> Owner: SDKWork maintainers

## �?�?��?��?

1. 缺�?�??确�??转�?�?��?�?�置
2. 不�?��?��?�没�??提�?�?�代�?��?
3. 缺�?模�??级�?��??�?��?�?��?4. �?�??不�?�??�?��?�?�以�?��?
## �?��??�??�?��?�据�?�??设计

### 1. Vendor级�?��?�置

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
        "不�?��?�system�?息",
        "不�?��?�tool_choice�?�?�"
      ],
      "source": {
        "observedAt": "2026-06-13T00:00:00Z",
        "sourceUrl": "https://..."
      }
    }
  }
}
```

### 2. 示�?�?�置

#### OpenAI Codex (�??�??�?��?�)
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

#### Alibaba Claude Code (转换�?��?�)
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
      "�?�?��?�global�?��??",
      "�?��??�?级�??�?�可�?�不�?�?��?��?
    ]
  }
}
```

#### DeepSeek Claude Code (OpenAI�?�容转换)
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
      "�??要额�?�??代�?�?�?�?协议转�?,
      "�?��??Claude�?��??�??�?�可�?�不�?��??
    ]
  }
}
```

### 3. 模�??�?��?类�??

| �?��?类�?? | 说�?? | 示�? |
|----------|------|------|
| direct | �?��?��?��?�?模�??名不�? | qwen3.7-max �??qwen3.7-max |
| prefix | 添�?��?��? | v4-pro �??deepseek-v4-pro |
| suffix | 添�?��?�? | claude �??claude-sonnet |
| custom | �?��?�?�?��?�?�??| deepseek-v4-pro �??claude-sonnet-4-20250514 |

### 4. 转换�?�?�

| �?�?� | 说�?? | �??�?��?��?� |
|------|------|----------|
| direct | �?��?�使�?��??�??API | �??�??�?��?��??vendor |
| proxy | �??�?代�?�?转�?| 协议不�?�容�?�??�?��?�似 |
| transform | �?�?�转换请�?/�?��? | 协议差�?�?大 |

### 5. 协议转换�?�??

| 转换�?�?? | 说�?? |
|----------|------|
| passthrough | �?��?��?�传�?不转换 |
| anthropic_to_openai | Anthropic格式 �??OpenAI格式 |
| openai_to_anthropic | OpenAI格式 �??Anthropic格式 |
| google_to_openai | Google格式 �??OpenAI格式 |
| openai_to_google | OpenAI格式 �??Google格式 |

## �?�?�计�??

1. �?��?�schema�?�?�??件
2. �?��?�vendors.json中�??clientApiCompatibility�?�??
3. 添�?�模�??�?��?�?�置
4. �?证�?��??�?�据�?�??
5. �?��?��??档

