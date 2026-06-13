package com.sdkwork.converters;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * 转换器接口
 */
public interface Converter {
    /**
     * 转换器名称
     */
    String name();

    /**
     * 源协议
     */
    Protocol sourceProtocol();

    /**
     * 目标协议
     */
    Protocol targetProtocol();

    /**
     * 支持的能力
     */
    List<Capability> capabilities();

    /**
     * 是否支持该转换
     */
    default boolean canConvert(Protocol source, Protocol target) {
        return sourceProtocol().equals(source) && targetProtocol().equals(target);
    }

    /**
     * 转换请求
     */
    CompletableFuture<ConversionRequest> convertRequest(ConversionRequest request);

    /**
     * 转换响应
     */
    CompletableFuture<ConversionResponse> convertResponse(ConversionResponse response);
}
