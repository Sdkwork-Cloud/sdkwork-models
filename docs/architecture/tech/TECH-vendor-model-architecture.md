> Owner: SDKWork maintainers

Generated: 2026-06-13

## Vendor Summary

| # | Vendor Code | Display Name | Open Source | Protocols | Regions | Client APIs |
|---|-------------|--------------|-------------|-----------|---------|-------------|
| 1 | openai | OpenAI | No | openai_compatible, openai_responses | global | CC:unsupported / CX:supported / GC:unsupported |
| 2 | anthropic | Anthropic | No | anthropic_messages | global | CC:supported / CX:unsupported / GC:unsupported |
| 3 | google | Google | No | google_gemini, vendor_native | global | CC:unsupported / CX:unsupported / GC:supported |
| 4 | xai | xAI | No | openai_compatible | global | CC:unsupported / CX:unsupported / GC:unsupported |
| 5 | alibaba | Alibaba Cloud | No | anthropic_messages, openai_compatible | cn, global | CC:partial / CX:unsupported / GC:unsupported |
| 6 | deepseek | DeepSeek | No | anthropic_messages, openai_compatible | cn, global | CC:unsupported / CX:unsupported / GC:unsupported |
| 7 | moonshot | Moonshot Kimi | No | openai_compatible | cn, global | CC:unsupported / CX:unsupported / GC:unsupported |
| 8 | zhipu | Zhipu AI | No | anthropic_messages, openai_compatible | cn | CC:unsupported / CX:unsupported / GC:unsupported |
| 9 | baidu | Baidu AI Cloud | No | vendor_native | cn | CC:unsupported / CX:unsupported / GC:unsupported |
| 10 | tencent | Tencent Cloud | No | openai_compatible | cn | CC:unsupported / CX:unsupported / GC:unsupported |
| 11 | bytedance | ByteDance | No | openai_compatible, vendor_native | cn, global | CC:unsupported / CX:unsupported / GC:unsupported |
| 12 | minimax | MiniMax | No | openai_compatible | cn, global | CC:unsupported / CX:unsupported / GC:unsupported |
| 13 | kuaishou | Kuaishou | No | vendor_native | cn, global | CC:unsupported / CX:unsupported / GC:unsupported |
| 14 | stability_ai | Stability AI | No | vendor_native | global | CC:unsupported / CX:unsupported / GC:unsupported |
| 15 | black_forest_labs | Black Forest Labs | No | vendor_native | global | CC:unsupported / CX:unsupported / GC:unsupported |
| 16 | suno | Suno | No | vendor_native | global | CC:unsupported / CX:unsupported / GC:unsupported |
| 17 | elevenlabs | ElevenLabs | No | vendor_native | global | CC:unsupported / CX:unsupported / GC:unsupported |
| 18 | xiaomi | Xiaomi MiMo | Yes | openai_compatible | cn, global | CC:unsupported / CX:unsupported / GC:unsupported |

## Model Architecture by Region

### Alibaba Cloud (alibaba)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| qwen3.6-max-preview | 262K | chat | CNY 9.00 / 54.00 |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| qwen3.7-max | 1000K | chat | USD 2.50 / 7.50 |

### Anthropic (anthropic)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| claude-haiku-4-5 | 200K | chat | USD 1.00 / 5.00 |
| claude-opus-4-7 | 1000K | chat | USD 5.00 / 25.00 |
| claude-opus-4-8 | 1000K | chat | USD 5.00 / 25.00 |
| claude-sonnet-4-6 | 1000K | chat | USD 3.00 / 15.00 |

### Baidu AI Cloud (baidu)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| ernie-5.0-thinking-preview | 128K | chat | CNY 4.00 / 16.00 |

### Black Forest Labs (black_forest_labs)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| flux-2-pro | N/A | image | N/A |

### ByteDance (bytedance)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| doubao-seed-2-0-pro-260215 | 256K | chat | CNY 3.20 / 16.00 |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| doubao-seedance-2-0-260128 | N/A | video | N/A |

### DeepSeek (deepseek)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| deepseek-v4-flash | 1048K | chat | CNY 1.00 / 2.00 |
| deepseek-v4-pro | 1048K | chat | CNY 3.00 / 6.00 |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| deepseek-v4-flash | 1048K | chat | USD 0.14 / 0.28 |
| deepseek-v4-pro | 1048K | chat | USD 0.435 / 0.87 |

### ElevenLabs (elevenlabs)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| eleven_text_to_sound_v2 | N/A | audio | N/A |

### Google (google)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| gemini-3-flash-preview | 1048K | chat | USD 0.50 / 3.00 |
| gemini-3.1-flash-image-preview | 32K | image, chat | N/A |
| gemini-3.1-flash-lite-preview | 1048K | chat | USD 0.10 / 0.40 |
| gemini-3.1-flash-lite | 1048K | chat | USD 0.10 / 0.40 |
| gemini-3.1-flash-live-preview | 131K | chat, audio | USD 0.50 / 2.00 |
| gemini-3.1-flash-tts-preview | 32K | audio | N/A |
| gemini-3.1-pro-preview | 1048K | chat | USD 2.00 / 12.00 |
| gemini-3.5-flash | 1048K | chat | USD 1.50 / 9.00 |
| imagen-4.0-generate-001 | N/A | image | N/A |
| veo-3.1-fast-generate-preview | N/A | video | N/A |
| veo-3.1-generate-preview | N/A | video | N/A |
| veo-3.1-lite-generate-preview | N/A | video | N/A |

### Kuaishou (kuaishou)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| kling-v3-0-preview | N/A | video | N/A |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| kling-v3-0-preview | N/A | video | N/A |

### MiniMax (minimax)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| M2-her | 65K | chat | CNY 2.10 / 8.40 |
| MiniMax-M2.5-highspeed | 204K | chat | CNY 4.20 / 16.80 |
| MiniMax-M2.5 | 204K | chat | CNY 2.10 / 8.40 |
| MiniMax-M2.7-highspeed | 204K | chat | CNY 4.20 / 16.80 |
| MiniMax-M2.7 | 204K | chat | CNY 2.10 / 8.40 |
| MiniMax-M3 | 1000K | chat | CNY 2.10 / 8.40 |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| M2-her | 65K | chat | USD 0.30 / 1.20 |
| MiniMax-M2.5-highspeed | 204K | chat | USD 0.60 / 2.40 |
| MiniMax-M2.5 | 204K | chat | USD 0.30 / 1.20 |
| MiniMax-M2.7-highspeed | 204K | chat | USD 0.60 / 2.40 |
| MiniMax-M2.7 | 204K | chat | USD 0.30 / 1.20 |
| MiniMax-M3 | 1000K | chat | USD 0.30 / 1.20 |

### Moonshot Kimi (moonshot)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| kimi-k2.5 | 262K | chat | CNY 4.00 / 21.00 |
| kimi-k2.6 | 262K | chat | CNY 6.50 / 27.00 |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| kimi-k2.5 | 262K | chat | USD 0.60 / 3.00 |
| kimi-k2.6 | 262K | chat | USD 0.95 / 4.00 |

### OpenAI (openai)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| gpt-4o-transcribe | N/A | audio | N/A |
| gpt-5.2 | 400K | chat | USD 1.75 / 14.00 |
| gpt-5.4-mini | 400K | chat | USD 0.75 / 4.50 |
| gpt-5.4-nano | 400K | chat | USD 0.20 / 1.25 |
| gpt-5.4-pro | 1050K | chat | USD 30.00 / 180.00 |
| gpt-5.4 | 1050K | chat | USD 2.50 / 15.00 |
| gpt-5.5-pro | 1050K | chat | USD 30.00 / 180.00 |
| gpt-5.5 | 1050K | chat | USD 5.00 / 30.00 |
| gpt-image-1.5 | N/A | image | N/A |
| gpt-image-2 | N/A | image | N/A |
| gpt-realtime-1.5 | N/A | chat, audio | USD 4.00 / 16.00 |
| gpt-realtime-2 | 128K | chat, audio | USD 4.00 / 24.00 |
| text-embedding-3-small | N/A | embedding | N/A |

### Stability AI (stability_ai)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| stable-image-ultra | N/A | image | N/A |

### Suno (suno)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| suno-v5 | N/A | music | N/A |

### Tencent Cloud (tencent)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| hunyuan-turbos-latest | 32K | chat | CNY 0.80 / 2.00 |

### xAI (xai)

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| grok-4.3 | 1000K | chat | USD 1.25 / 2.50 |

### Xiaomi MiMo (xiaomi)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| mimo-v2-flash | 262K | chat | CNY 0.70 / 2.10 |
| mimo-v2.5-pro | 1048K | chat | CNY 14.00 / 42.00 |
| mimo-v2.5 | 1048K | chat, image, audio, video | CNY 5.60 / 16.80 |

**Region: GLOBAL** (USD)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| mimo-v2-flash | 262K | chat | USD 0.10 / 0.30 |
| mimo-v2.5-pro | 1048K | chat | USD 2.00 / 6.00 |
| mimo-v2.5 | 1048K | chat, image, audio, video | USD 0.80 / 2.40 |

### Zhipu AI (zhipu)

**Region: CN** (CNY)

| Model ID | Context | Modalities | Pricing (Input/Output) |
|----------|---------|------------|------------------------|
| glm-5.1 | 200K | chat | CNY 6.00 / 24.00 |

## Statistics Summary

### Vendor Count by Region

| Region | Vendors | Models | Pricing Files |
|--------|---------|--------|---------------|
| CN | 10 | 19 | 18 |
| GLOBAL | 15 | 50 | 50 |

### Client API Support

| API | Supported | Partial | Unsupported |
|-----|-----------|---------|-------------|
| claude_code | 1 | 1 | 16 |
| codex | 1 | 0 | 17 |
| gemini_cli | 1 | 0 | 17 |

### Protocol Support

| Protocol | Vendors |
|----------|---------|
| openai_compatible | 10 |
| vendor_native | 8 |
| anthropic_messages | 4 |
| openai_responses | 1 |
| google_gemini | 1 |

### Capability Support

| Capability | Vendors |
|------------|---------|
| chat | 13 |
| image | 10 |
| video | 8 |
| audio | 5 |
| music | 3 |
| tool | 2 |
| embedding | 1 |

