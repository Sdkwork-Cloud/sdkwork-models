> Owner: SDKWork maintainers

## 设计�??�??

1. **�?�?��??* - �?�??�??�??可�?��??客�?�端工�?��??协议
2. **可�?��?�??* - �?��?��?�来�?��?工�?��?�协议�?��?��??3. **�?�?�?�容** - �?��??�?��?�??�?�容�?��??�?��?�据
4. **�?洁�??* - �?段�?�名�?洁�?�?�??�?�?�
5. **可�?证�??* - �?��?��?�置�?证�??�?�?��?��?�??
## 核�?�?�据�?�??

```json
{
  "clientApi": {
    "<api_code>": {
      "status": "native|convert|none",
      "version": "1.0",
      "protocol": "<protocol_code>",
      "endpoint": "<api_endpoint>",
      "convert": {
        "from": "<source_protocol>",
        "map": { "<source_model>": "<target_model>" }
      },
      "caps": ["<capability_code>"],
      "regions": ["<region_code>"],
      "note": "<description>",
      "meta": {}
    }
  }
}
```

## �?段�?�?

### �?填�?段

| �?段 | 类�?? | 说�?? | 示�? |
|------|------|------|------|
| status | string | �?��?��?��??| native, convert, none |
| protocol | string | 协议代码 | anthropic_messages, openai_compatible |

### 可�??�?�?
| �?段 | 类�?? | 说�?? | �?认�??|
|------|------|------|--------|
| version | string | �?�置�??�?� | "1.0" |
| endpoint | string | API端�?� | - |
| convert | object | 转换�?�置 | null |
| caps | array | �?��??�??表 | [] |
| regions | array | �?��?��?��?? | ["global"] |
| note | string | �?注说�?? | - |
| meta | object | �?��?�??�?��?| {} |

### convert 对象

| �?段 | 类�?? | 说�?? |
|------|------|------|
| from | string | 转换来源协议 |
| map | object | 模�??�?��? {�? �?��?} |

## �?��?��?�?
| �?��??| 说�?? | 使�?��?��?� |
|------|------|----------|
| native | �??�??�?��?� | vendor�?��?��?��?�该API |
| convert | �??要转�?| �??�?协议转换�?��?� |
| none | 不�?��??| �?��?使�?�该API |

## 协议代码�?
| 代码 | 说�?? | �??�?� |
|------|------|------|
| anthropic_messages | Anthropic Messages API | 2023-06-01 |
| openai_responses | OpenAI Responses API | 2024-10-01 |
| openai_compatible | OpenAI�?�容格式 | - |
| google_gemini | Google Gemini API | v1 |
| vendor_native | �??�??�??�??API | - |

## �?��??代码�?
| 代码 | 说�?? | 可�?��?|
|------|------|--------|
| stream | 流式�?�?� | - |
| tools | 工�?��?�?� | - |
| vision | �?��?��?解 | - |
| audio | �?��?�?�? | - |
| video | �?�?�?�? | �??|
| image | �?��?��??�?� | �??|
| music | �?�乐�??�?� | �??|
| code | 代码�??�?� | �??|
| reasoning | �?��?�?��?? | �??|

## �?��??代码�?
| 代码 | 说�?? |
|------|------|
| global | �?��?�?��?? |
| cn | 中�?�大�?? |
| us | �?�?� |
| eu | 欧�?? |
| asia | �?太 |

## API代码�?
| 代码 | 说�?? | 可�?��?|
|------|------|--------|
| claude_code | Anthropic Claude Code | - |
| codex | OpenAI Codex | - |
| gemini_cli | Google Gemini CLI | - |
| cursor | Cursor IDE | �??|
| copilot | GitHub Copilot | �??|
| cline | Cline | �??|
| aider | Aider | �??|

## �?��?�?��?�

### 1. meta �?��?

```json
{
  "meta": {
    "custom_field": "value",
    "feature_flags": ["flag1", "flag2"],
    "rate_limit": {
      "requests_per_minute": 1000
    }
  }
}
```

### 2. �?��?��?��?
```json
{
  "dynamic": {
    "enabled": true,
    "refresh_interval": 3600,
    "config_url": "https://api.example.com/config"
  }
}
```

### 3. �??�?��?�容

```json
{
  "version": "2.0",
  "compatibility": {
    "min_version": "1.0",
    "deprecated_fields": ["old_field"],
    "migration_notes": "..."
  }
}
```

## �?�?�示�?

### OpenAI (�??�??�?��?�Codex)

```json
{
  "clientApi": {
    "codex": {
      "status": "native",
      "version": "1.0",
      "protocol": "openai_responses",
      "endpoint": "/v1/responses",
      "caps": ["stream", "tools", "vision", "code"],
      "regions": ["global"],
      "note": "OpenAI�??�??Codex API",
      "meta": {
        "api_version": "2024-10-01",
        "max_tokens": 128000
      }
    }
  }
}
```

### Alibaba Cloud (转换�?��?�Claude Code)

```json
{
  "clientApi": {
    "claude_code": {
      "status": "convert",
      "version": "1.0",
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
      "note": "Qwen3.7-Max�?�容Anthropic格式�?�?global�?��??",
      "meta": {
        "compatibility_level": "high",
        "limitations": ["不�?��?�vision"]
      }
    }
  }
}
```

### DeepSeek (�??�?代�?�?��?�Claude Code)

```json
{
  "clientApi": {
    "claude_code": {
      "status": "convert",
      "version": "1.0",
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
      "note": "�??要代�?�?�?�?协议转换",
      "meta": {
        "proxy_required": true,
        "conversion_overhead": "low"
      }
    }
  }
}
```

### �?�来�?��?示�? (Cursor)

```json
{
  "clientApi": {
    "cursor": {
      "status": "convert",
      "version": "1.0",
      "protocol": "openai_compatible",
      "endpoint": "/v1/chat/completions",
      "convert": {
        "from": "openai_compatible",
        "map": {
          "gpt-4": "gpt-4-turbo",
          "claude-3-opus": "claude-3-opus-20240229"
        }
      },
      "caps": ["stream", "tools", "code", "vision"],
      "regions": ["global"],
      "note": "�??�?OpenAI�?�容�?�口�?��?�Cursor",
      "meta": {
        "ide_integration": true,
        "context_window": 128000
      }
    }
  }
}
```

## �?证�?�??

### �?填�?证

- status �?须�??native|convert|none
- protocol �?须�?�协议代码表�?- �?status=convert �?��?convert.from �?填

### �?�?��?��?�?
- �?status=native �?��?convert �?为 null �??enabled=false
- �?status=none �?��?protocol �??endpoint 可�??- caps 中�??�?��??代码�?须�?��?��??代码表�?
### �?��?�?证

- meta 中�??�?��?�?�?段不影�?�核�?�??�?�
- version �?须遵循语�?�??�??�?��?�??- regions 中�??�?��??代码�?须�?��?��??代码表�?
## 迁移�??�?

### v1 �??v2

1. 添�?� version �?段
2. �?supportStatus �?�为 status
3. �?capabilities �?�为 caps
4. �?limitations �?�为 note
5. �?�??convert �?�??

### �?�?�?�容

- v2 �?�??可以读�? v1 �?�据
- v1 工�?��??要�??�?��?��?��?�读�?v2 �?�据
- 建议�?�步迁移�?保�?��?�??�?��?��?�

