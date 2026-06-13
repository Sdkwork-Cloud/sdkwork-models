package com.sdkwork.converters

/**
 * 转换器接口
 */
interface Converter {
    /**
     * 转换器名称
     */
    val name: String

    /**
     * 源协议
     */
    val sourceProtocol: Protocol

    /**
     * 目标协议
     */
    val targetProtocol: Protocol

    /**
     * 支持的能力
     */
    val capabilities: List<Capability>

    /**
     * 是否支持该转换
     */
    fun canConvert(source: Protocol, target: Protocol): Boolean {
        return sourceProtocol == source && targetProtocol == target
    }

    /**
     * 转换请求
     */
    suspend fun convertRequest(request: ConversionRequest): ConversionRequest

    /**
     * 转换响应
     */
    suspend fun convertResponse(response: ConversionResponse): ConversionResponse
}

/**
 * 映射器接口
 */
interface Mapper {
    /**
     * 映射器名称
     */
    val name: String

    /**
     * 映射单个模型
     */
    fun map(sourceModel: String, mapping: ModelMapping): String

    /**
     * 反向映射
     */
    fun reverseMap(targetModel: String, mapping: ModelMapping): String

    /**
     * 批量映射
     */
    fun mapBatch(models: List<String>, mapping: ModelMapping): List<String> {
        return models.map { map(it, mapping) }
    }
}

/**
 * 标准模型映射器
 */
class ModelMapper : Mapper {
    override val name: String = "model_mapper"

    override fun map(sourceModel: String, mapping: ModelMapping): String {
        return mapping.resolve(sourceModel)
    }

    override fun reverseMap(targetModel: String, mapping: ModelMapping): String {
        return mapping.reverseResolve(targetModel)
    }
}

/**
 * 前缀映射器
 */
class PrefixMapper(private val prefix: String) : Mapper {
    override val name: String = "prefix_mapper"

    override fun map(sourceModel: String, mapping: ModelMapping): String {
        return if (sourceModel.startsWith(prefix)) sourceModel else "$prefix$sourceModel"
    }

    override fun reverseMap(targetModel: String, mapping: ModelMapping): String {
        return if (targetModel.startsWith(prefix)) targetModel.removePrefix(prefix) else targetModel
    }
}
