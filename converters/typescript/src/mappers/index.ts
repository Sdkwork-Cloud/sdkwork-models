import type { Mapper } from '../traits';
import type { ModelMapping } from '../types';

/** 标准模型映射器 */
export class ModelMapper implements Mapper {
  name(): string {
    return 'model_mapper';
  }

  map(sourceModel: string, mapping: ModelMapping): string {
    return mapping.mapping[sourceModel] ?? sourceModel;
  }

  reverseMap(targetModel: string, mapping: ModelMapping): string {
    const reverse: Record<string, string> = {};
    for (const [key, value] of Object.entries(mapping.mapping)) {
      reverse[value] = key;
    }
    return reverse[targetModel] ?? targetModel;
  }

  mapBatch(models: string[], mapping: ModelMapping): string[] {
    return models.map((m) => this.map(m, mapping));
  }
}

/** 前缀映射器 */
export class PrefixMapper implements Mapper {
  constructor(private prefix: string) {}

  name(): string {
    return 'prefix_mapper';
  }

  map(sourceModel: string, _mapping: ModelMapping): string {
    if (sourceModel.startsWith(this.prefix)) {
      return sourceModel;
    }
    return `${this.prefix}${sourceModel}`;
  }

  reverseMap(targetModel: string, _mapping: ModelMapping): string {
    if (targetModel.startsWith(this.prefix)) {
      return targetModel.slice(this.prefix.length);
    }
    return targetModel;
  }

  mapBatch(models: string[], mapping: ModelMapping): string[] {
    return models.map((m) => this.map(m, mapping));
  }
}
