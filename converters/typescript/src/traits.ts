import type {
  Protocol,
  Capability,
  ConversionRequest,
  ConversionResponse,
  ModelMapping,
} from './types';

/** 转换器接口 */
export interface Converter {
  /** 转换器名称 */
  name(): string;

  /** 源协议 */
  sourceProtocol(): Protocol;

  /** 目标协议 */
  targetProtocol(): Protocol;

  /** 支持的能力 */
  capabilities(): Capability[];

  /** 是否支持该转换 */
  canConvert(source: Protocol, target: Protocol): boolean;

  /** 转换请求 */
  convertRequest(request: ConversionRequest): Promise<ConversionRequest>;

  /** 转换响应 */
  convertResponse(response: ConversionResponse): Promise<ConversionResponse>;
}

/** 映射器接口 */
export interface Mapper {
  /** 映射器名称 */
  name(): string;

  /** 映射单个模型 */
  map(sourceModel: string, mapping: ModelMapping): string;

  /** 反向映射 */
  reverseMap(targetModel: string, mapping: ModelMapping): string;

  /** 批量映射 */
  mapBatch(models: string[], mapping: ModelMapping): string[];
}
