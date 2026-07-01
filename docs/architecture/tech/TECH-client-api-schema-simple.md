> Owner: SDKWork maintainers

## �?�??�?�??�?�据�?�??
```json
{
  "clientApi": {
    "claude_code": {
      "status": "native|convert|none",
      "protocol": "anthropic_messages",
      "endpoint": "/v1/messages",
      "convert": {
        "from": "openai_compatible",
        "map": { "deepseek-v4-pro": "claude-sonnet-4" }
      },
      "caps": ["stream", "tools", "vision"],
      "note": "�?�?��?�global�?��??"
    }
  }
}
```

## �?段说�??

| �?段 | 类�?? | 说�?? |
|------|------|------|
| status | string | native=�??�??�?��?�, convert=�??转换, none=不�?��??|
| protocol | string | 使�?��??协议代�?|
| endpoint | string | API端�?� |
| convert.from | string | 转换来源协议 |
| convert.map | object | 模�??�?��? {源模�?? �?��?模�??} |
| caps | array | �?��?��??�?��??|
| note | string | �?注说�?? |

## �?��??代码

| 代码 | 说�?? |
|------|------|
| stream | 流式�?�?� |
| tools | 工�?��?�?� |
| vision | �?��?��?解 |
| audio | �?��?�?�? |

## 示�?�?�置

### OpenAI (�??�??�?��?�Codex)
```json
{
  "clientApi": {
    "codex": {
      "status": "native",
      "protocol": "openai_responses",
      "endpoint": "/v1/responses",
      "caps": ["stream", "tools", "vision"]
    }
  }
}
```

### Alibaba (转换�?��?�Claude Code)
```json
{
  "clientApi": {
    "claude_code": {
      "status": "convert",
      "protocol": "anthropic_messages",
      "endpoint": "/v1/messages",
      "convert": {
        "from": "anthropic_messages",
        "map": { "qwen3.7-max": "claude-sonnet-4" }
      },
      "caps": ["stream", "tools"],
      "note": "�?global�?��??�?Qwen3.7-Max�?�容Anthropic格式"
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
      "protocol": "openai_compatible",
      "endpoint": "/v1/chat/completions",
      "convert": {
        "from": "openai_compatible",
        "map": {
          "deepseek-v4-pro": "claude-sonnet-4",
          "deepseek-v4-flash": "claude-haiku-4"
        }
      },
      "caps": ["stream", "tools"],
      "note": "�??要代�?�?�?�?协议转换"
    }
  }
}
```

### 不�?��?��??�??�?�
```json
{
  "clientApi": {
    "claude_code": {
      "status": "none",
      "note": "不�?��?�Claude Code客�?�端API"
    }
  }
}
```

