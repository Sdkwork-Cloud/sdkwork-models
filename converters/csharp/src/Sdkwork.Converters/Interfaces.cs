namespace Sdkwork.Converters;

/// <summary>
/// 转换器接口
/// </summary>
public interface IConverter
{
    /// <summary>
    /// 转换器名称
    /// </summary>
    string Name { get; }

    /// <summary>
    /// 源协议
    /// </summary>
    Protocol SourceProtocol { get; }

    /// <summary>
    /// 目标协议
    /// </summary>
    Protocol TargetProtocol { get; }

    /// <summary>
    /// 支持的能力
    /// </summary>
    IReadOnlyList<Capability> Capabilities { get; }

    /// <summary>
    /// 是否支持该转换
    /// </summary>
    bool CanConvert(Protocol source, Protocol target)
    {
        return SourceProtocol == source && TargetProtocol == target;
    }

    /// <summary>
    /// 转换请求
    /// </summary>
    Task<ConversionRequest> ConvertRequestAsync(ConversionRequest request, CancellationToken cancellationToken = default);

    /// <summary>
    /// 转换响应
    /// </summary>
    Task<ConversionResponse> ConvertResponseAsync(ConversionResponse response, CancellationToken cancellationToken = default);
}

/// <summary>
/// 映射器接口
/// </summary>
public interface IMapper
{
    /// <summary>
    /// 映射器名称
    /// </summary>
    string Name { get; }

    /// <summary>
    /// 映射单个模型
    /// </summary>
    string Map(string sourceModel, ModelMapping mapping);

    /// <summary>
    /// 反向映射
    /// </summary>
    string ReverseMap(string targetModel, ModelMapping mapping);

    /// <summary>
    /// 批量映射
    /// </summary>
    IReadOnlyList<string> MapBatch(IReadOnlyList<string> models, ModelMapping mapping)
    {
        return models.Select(m => Map(m, mapping)).ToList();
    }
}
