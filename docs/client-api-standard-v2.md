# Client API Compatibility Standard v2

## 设计原则

1. **完整性** - 覆盖所有可能的客户端工具和协议
2. **可扩展性** - 支持未来新增工具、协议、能力
3. **向后兼容** - 新版本结构兼容旧版本数据
4. **简洁性** - 字段命名简洁，结构清晰
5. **可验证性** - 支持配置验证和一致性检查

## 核心数据结构

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

## 字段定义

### 必填字段

| 字段 | 类型 | 说明 | 示例 |
|------|------|------|------|
| status | string | 支持状态 | native, convert, none |
| protocol | string | 协议代码 | anthropic_messages, openai_compatible |

### 可选字段

| 字段 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| version | string | 配置版本 | "1.0" |
| endpoint | string | API端点 | - |
| convert | object | 转换配置 | null |
| caps | array | 能力列表 | [] |
| regions | array | 支持区域 | ["global"] |
| note | string | 备注说明 | - |
| meta | object | 扩展元数据 | {} |

### convert 对象

| 字段 | 类型 | 说明 |
|------|------|------|
| from | string | 转换来源协议 |
| map | object | 模型映射 {源: 目标} |

## 状态定义

| 状态 | 说明 | 使用场景 |
|------|------|----------|
| native | 原生支持 | vendor直接暴露该API |
| convert | 需要转换 | 通过协议转换支持 |
| none | 不支持 | 无法使用该API |

## 协议代码表

| 代码 | 说明 | 版本 |
|------|------|------|
| anthropic_messages | Anthropic Messages API | 2023-06-01 |
| openai_responses | OpenAI Responses API | 2024-10-01 |
| openai_compatible | OpenAI兼容格式 | - |
| google_gemini | Google Gemini API | v1 |
| vendor_native | 厂商原生API | - |

## 能力代码表

| 代码 | 说明 | 可扩展 |
|------|------|--------|
| stream | 流式输出 | - |
| tools | 工具调用 | - |
| vision | 图像理解 | - |
| audio | 音频处理 | - |
| video | 视频处理 | ✓ |
| image | 图像生成 | ✓ |
| music | 音乐生成 | ✓ |
| code | 代码生成 | ✓ |
| reasoning | 推理能力 | ✓ |

## 区域代码表

| 代码 | 说明 |
|------|------|
| global | 全球区域 |
| cn | 中国大陆 |
| us | 美国 |
| eu | 欧盟 |
| asia | 亚太 |

## API代码表

| 代码 | 说明 | 可扩展 |
|------|------|--------|
| claude_code | Anthropic Claude Code | - |
| codex | OpenAI Codex | - |
| gemini_cli | Google Gemini CLI | - |
| cursor | Cursor IDE | ✓ |
| copilot | GitHub Copilot | ✓ |
| cline | Cline | ✓ |
| aider | Aider | ✓ |

## 扩展机制

### 1. meta 扩展

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

### 2. 动态配置

```json
{
  "dynamic": {
    "enabled": true,
    "refresh_interval": 3600,
    "config_url": "https://api.example.com/config"
  }
}
```

### 3. 版本兼容

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

## 完整示例

### OpenAI (原生支持Codex)

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
      "note": "OpenAI原生Codex API",
      "meta": {
        "api_version": "2024-10-01",
        "max_tokens": 128000
      }
    }
  }
}
```

### Alibaba Cloud (转换支持Claude Code)

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
      "note": "Qwen3.7-Max兼容Anthropic格式，仅global区域",
      "meta": {
        "compatibility_level": "high",
        "limitations": ["不支持vision"]
      }
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
      "note": "需要代理层进行协议转换",
      "meta": {
        "proxy_required": true,
        "conversion_overhead": "low"
      }
    }
  }
}
```

### 未来扩展示例 (Cursor)

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
      "note": "通过OpenAI兼容接口支持Cursor",
      "meta": {
        "ide_integration": true,
        "context_window": 128000
      }
    }
  }
}
```

## 验证规则

### 必填验证

- status 必须是 native|convert|none
- protocol 必须在协议代码表中
- 当 status=convert 时，convert.from 必填

### 一致性验证

- 当 status=native 时，convert 应为 null 或 enabled=false
- 当 status=none 时，protocol 和 endpoint 可选
- caps 中的能力代码必须在能力代码表中

### 扩展验证

- meta 中的自定义字段不影响核心功能
- version 必须遵循语义化版本规范
- regions 中的区域代码必须在区域代码表中

## 迁移指南

### v1 → v2

1. 添加 version 字段
2. 将 supportStatus 改为 status
3. 将 capabilities 改为 caps
4. 将 limitations 改为 note
5. 简化 convert 结构

### 向后兼容

- v2 结构可以读取 v1 数据
- v1 工具需要适配器才能读取 v2 数据
- 建议逐步迁移，保持双版本支持
