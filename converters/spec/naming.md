# Converter Naming Specification

## 命名格式

### 转换器名称

```
<SOURCE_PROTOCOL>_TO_<TARGET_PROTOCOL>
```

### 目录/文件命名

| 语言 | 目录命名 | 文件命名 |
|------|----------|----------|
| Rust | snake_case | snake_case.rs |
| TypeScript | kebab-case | kebab-case.ts |
| Python | snake_case | snake_case.py |
| Java | PascalCase | PascalCase.java |
| Go | snake_case | snake_case.go |
| Swift | PascalCase | PascalCase.swift |

### 示例

| 转换器名称 | Rust目录 | TypeScript目录 | Python目录 |
|------------|----------|----------------|------------|
| OPENAI_RESPONSES_TO_ANTHROPIC_MESSAGES | openai_responses_to_anthropic_messages/ | openai-responses-to-anthropic-messages/ | openai_responses_to_anthropic_messages/ |
| OPENAI_COMPLETIONS_TO_ANTHROPIC_MESSAGES | openai_completions_to_anthropic_messages/ | openai-completions-to-anthropic-messages/ | openai_completions_to_anthropic_messages/ |

## 协议代码

| 代码 | 说明 |
|------|------|
| OPENAI_RESPONSES | OpenAI Responses API |
| OPENAI_COMPLETIONS | OpenAI Chat Completions API |
| ANTHROPIC_MESSAGES | Anthropic Messages API |
| GOOGLE_GEMINI | Google Gemini API |
| OPENAI_COMPATIBLE | OpenAI兼容格式 |

## 能力代码

| 代码 | 说明 |
|------|------|
| stream | 流式输出 |
| tools | 工具调用 |
| vision | 图像理解 |
| audio | 音频处理 |
| video | 视频处理 |
| image | 图像生成 |
| code | 代码生成 |
| reasoning | 推理能力 |
