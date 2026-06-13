package com.sdkwork.converters

import kotlinx.serialization.Serializable

/**
 * 协议枚举
 */
enum class Protocol(val value: String) {
    OPENAI_RESPONSES("openai_responses"),
    OPENAI_COMPLETIONS("openai_completions"),
    ANTHROPIC_MESSAGES("anthropic_messages"),
    GOOGLE_GEMINI("google_gemini"),
    OPENAI_COMPATIBLE("openai_compatible"),
    VENDOR_NATIVE("vendor_native");

    companion object {
        fun fromValue(value: String): Protocol = entries.first { it.value == value }
    }
}

/**
 * 能力枚举
 */
enum class Capability(val value: String) {
    STREAM("stream"),
    TOOLS("tools"),
    VISION("vision"),
    AUDIO("audio"),
    VIDEO("video"),
    IMAGE("image"),
    CODE("code"),
    REASONING("reasoning")
}

/**
 * 角色枚举
 */
enum class Role(val value: String) {
    SYSTEM("system"),
    USER("user"),
    ASSISTANT("assistant"),
    TOOL("tool")
}

/**
 * 停止原因枚举
 */
enum class StopReason(val value: String) {
    END_TURN("end_turn"),
    STOP_SEQUENCE("stop_sequence"),
    MAX_TOKENS("max_tokens"),
    TOOL_USE("tool_use"),
    STOP("stop"),
    LENGTH("length"),
    CONTENT_FILTER("content_filter")
}

/**
 * 消息
 */
@Serializable
data class Message(
    val role: Role,
    val content: String // 简化版本，实际可能是 String 或 List<ContentPart>
)

/**
 * 转换请求
 */
@Serializable
data class ConversionRequest(
    val protocol: Protocol,
    val model: String,
    val messages: List<Message>,
    val maxTokens: Int? = null,
    val temperature: Double? = null,
    val topP: Double? = null,
    val stream: Boolean = false,
    val tools: List<Tool>? = null,
    val system: String? = null,
    val metadata: Map<String, String>? = null
)

/**
 * 转换响应
 */
@Serializable
data class ConversionResponse(
    val protocol: Protocol,
    val id: String,
    val model: String,
    val content: List<ContentPart>,
    val stopReason: StopReason? = null,
    val usage: Usage,
    val metadata: Map<String, String>? = null
)

/**
 * 内容部分
 */
@Serializable
data class ContentPart(
    val type: String,
    val text: String? = null,
    val imageUrl: ImageUrl? = null,
    val source: ImageSource? = null,
    val id: String? = null,
    val name: String? = null,
    val input: String? = null,
    val toolUseId: String? = null
)

/**
 * 图片URL
 */
@Serializable
data class ImageUrl(
    val url: String,
    val detail: String? = null
)

/**
 * 图片源
 */
@Serializable
data class ImageSource(
    val type: String,
    val mediaType: String,
    val data: String
)

/**
 * 工具定义
 */
@Serializable
data class Tool(
    val type: String = "function",
    val function: Function
)

/**
 * 函数定义
 */
@Serializable
data class Function(
    val name: String,
    val description: String? = null,
    val parameters: String? = null,
    val inputSchema: String? = null
)

/**
 * Token使用统计
 */
@Serializable
data class Usage(
    val promptTokens: Int = 0,
    val completionTokens: Int = 0,
    val totalTokens: Int = 0
)

/**
 * 模型映射配置
 */
@Serializable
data class ModelMapping(
    val mapping: Map<String, String> = emptyMap(),
    val wildcardRules: List<WildcardRule>? = null
) {
    fun resolve(model: String): String = mapping[model] ?: model

    fun reverseResolve(model: String): String {
        val reverse = mapping.entries.associate { (k, v) -> v to k }
        return reverse[model] ?: model
    }
}

/**
 * 通配符规则
 */
@Serializable
data class WildcardRule(
    val pattern: String,
    val target: String
)
