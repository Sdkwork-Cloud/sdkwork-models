# Protocol Specification

## 协议定义

### OPENAI_RESPONSES

- **全称**: OpenAI Responses API
- **版本**: 2024-10-01
- **端点**: `/v1/responses`
- **文档**: https://platform.openai.com/docs/api-reference/responses

### OPENAI_COMPLETIONS

- **全称**: OpenAI Chat Completions API
- **版本**: 2024-10-01
- **端点**: `/v1/chat/completions`
- **文档**: https://platform.openai.com/docs/api-reference/chat

### ANTHROPIC_MESSAGES

- **全称**: Anthropic Messages API
- **版本**: 2023-06-01
- **端点**: `/v1/messages`
- **文档**: https://docs.anthropic.com/en/api/messages

### GOOGLE_GEMINI

- **全称**: Google Gemini API
- **版本**: v1
- **端点**: `/v1/models/{model}:generateContent`
- **文档**: https://ai.google.dev/gemini-api/docs

### OPENAI_COMPATIBLE

- **全称**: OpenAI兼容格式
- **版本**: -
- **端点**: `/v1/chat/completions`
- **说明**: 通用OpenAI兼容协议，用于第三方vendor

## 消息格式对比

### 角色

| 角色 | OpenAI | Anthropic | 说明 |
|------|--------|-----------|------|
| 系统 | system | system | 系统提示 |
| 用户 | user | user | 用户输入 |
| 助手 | assistant | assistant | AI响应 |
| 工具 | tool | tool | 工具调用结果 |

### 内容类型

| 类型 | OpenAI | Anthropic | 说明 |
|------|--------|-----------|------|
| 文本 | text | text | 纯文本 |
| 图片URL | image_url | image_url | 网络图片 |
| Base64图片 | image_url (data:) | image (base64) | 内嵌图片 |
| 工具调用 | tool_calls | tool_use | AI调用工具 |
| 工具结果 | tool | tool_result | 工具返回结果 |

### 停止原因

| OpenAI | Anthropic | 说明 |
|--------|-----------|------|
| stop | end_turn | 正常结束 |
| length | max_tokens | 达到最大token |
| tool_calls | tool_use | 需要调用工具 |
| content_filter | - | 内容过滤 |

## 转换规则

### 请求转换

1. **模型映射**: 根据mapping配置转换模型名称
2. **系统消息**: 从messages中提取，转换为system字段
3. **消息格式**: 保持角色和内容不变
4. **工具格式**: 转换function定义格式
5. **参数映射**: max_tokens, temperature, top_p等

### 响应转换

1. **模型反向映射**: 根据mapping配置反向转换模型名称
2. **内容转换**: 保持内容类型和文本不变
3. **停止原因映射**: 转换停止原因格式
4. **使用统计**: 保持token计数不变
