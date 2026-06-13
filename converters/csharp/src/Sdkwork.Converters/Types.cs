namespace Sdkwork.Converters;

/// <summary>
/// 协议枚举
/// </summary>
public enum Protocol
{
    OpenAiResponses,
    OpenAiCompletions,
    AnthropicMessages,
    GoogleGemini,
    OpenAiCompatible,
    VendorNative
}

/// <summary>
/// 能力枚举
/// </summary>
public enum Capability
{
    Stream,
    Tools,
    Vision,
    Audio,
    Video,
    Image,
    Code,
    Reasoning
}

/// <summary>
/// 角色枚举
/// </summary>
public enum Role
{
    System,
    User,
    Assistant,
    Tool
}

/// <summary>
/// 停止原因枚举
/// </summary>
public enum StopReason
{
    EndTurn,
    StopSequence,
    MaxTokens,
    ToolUse,
    Stop,
    Length,
    ContentFilter
}

/// <summary>
/// 消息
/// </summary>
public record Message
{
    public Role Role { get; init; }
    public object Content { get; init; } = null!;
}

/// <summary>
/// 转换请求
/// </summary>
public record ConversionRequest
{
    public Protocol Protocol { get; init; }
    public string Model { get; init; } = string.Empty;
    public List<Message> Messages { get; init; } = new();
    public int? MaxTokens { get; init; }
    public double? Temperature { get; init; }
    public double? TopP { get; init; }
    public bool Stream { get; init; }
    public List<Tool>? Tools { get; init; }
    public string? System { get; init; }
    public Dictionary<string, object>? Metadata { get; init; }
}

/// <summary>
/// 转换响应
/// </summary>
public record ConversionResponse
{
    public Protocol Protocol { get; init; }
    public string Id { get; init; } = string.Empty;
    public string Model { get; init; } = string.Empty;
    public List<ContentPart> Content { get; init; } = new();
    public StopReason? StopReason { get; init; }
    public Usage Usage { get; init; } = new();
    public Dictionary<string, object>? Metadata { get; init; }
}

/// <summary>
/// 内容部分
/// </summary>
public record ContentPart
{
    public string Type { get; init; } = string.Empty;
    public string? Text { get; init; }
    public ImageUrl? ImageUrl { get; init; }
    public ImageSource? Source { get; init; }
    public string? Id { get; init; }
    public string? Name { get; init; }
    public object? Input { get; init; }
    public string? ToolUseId { get; init; }
}

/// <summary>
/// 图片URL
/// </summary>
public record ImageUrl
{
    public string Url { get; init; } = string.Empty;
    public string? Detail { get; init; }
}

/// <summary>
/// 图片源
/// </summary>
public record ImageSource
{
    public string Type { get; init; } = string.Empty;
    public string MediaType { get; init; } = string.Empty;
    public string Data { get; init; } = string.Empty;
}

/// <summary>
/// 工具定义
/// </summary>
public record Tool
{
    public string Type { get; init; } = "function";
    public Function Function { get; init; } = new();
}

/// <summary>
/// 函数定义
/// </summary>
public record Function
{
    public string Name { get; init; } = string.Empty;
    public string? Description { get; init; }
    public object? Parameters { get; init; }
    public object? InputSchema { get; init; }
}

/// <summary>
/// Token使用统计
/// </summary>
public record Usage
{
    public int PromptTokens { get; init; }
    public int CompletionTokens { get; init; }
    public int TotalTokens { get; init; }
}

/// <summary>
/// 模型映射配置
/// </summary>
public record ModelMapping
{
    public Dictionary<string, string> Mapping { get; init; } = new();
    public List<WildcardRule>? WildcardRules { get; init; }

    public string Resolve(string model)
    {
        return Mapping.TryGetValue(model, out var target) ? target : model;
    }

    public string ReverseResolve(string model)
    {
        var reverse = Mapping.ToDictionary(kvp => kvp.Value, kvp => kvp.Key);
        return reverse.TryGetValue(model, out var target) ? target : model;
    }
}

/// <summary>
/// 通配符规则
/// </summary>
public record WildcardRule
{
    public string Pattern { get; init; } = string.Empty;
    public string Target { get; init; } = string.Empty;
}
