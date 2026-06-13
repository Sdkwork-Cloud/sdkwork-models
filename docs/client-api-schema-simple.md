# Client API Compatibility Schema (简化版)

## 简化后的数据结构

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
      "note": "仅支持global区域"
    }
  }
}
```

## 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| status | string | native=原生支持, convert=需转换, none=不支持 |
| protocol | string | 使用的协议代码 |
| endpoint | string | API端点 |
| convert.from | string | 转换来源协议 |
| convert.map | object | 模型映射 {源模型: 目标模型} |
| caps | array | 支持的能力 |
| note | string | 备注说明 |

## 能力代码

| 代码 | 说明 |
|------|------|
| stream | 流式输出 |
| tools | 工具调用 |
| vision | 图像理解 |
| audio | 音频处理 |

## 示例配置

### OpenAI (原生支持Codex)
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

### Alibaba (转换支持Claude Code)
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
      "note": "仅global区域，Qwen3.7-Max兼容Anthropic格式"
    }
  }
}
```

### DeepSeek (通过代理支持Claude Code)
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
      "note": "需要代理层进行协议转换"
    }
  }
}
```

### 不支持的情况
```json
{
  "clientApi": {
    "claude_code": {
      "status": "none",
      "note": "不支持Claude Code客户端API"
    }
  }
}
```
