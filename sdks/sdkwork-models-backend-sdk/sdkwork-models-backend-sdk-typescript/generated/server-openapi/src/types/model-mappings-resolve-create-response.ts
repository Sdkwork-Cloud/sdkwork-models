import type { ModelMappingsResolveCreateResult } from './model-mappings-resolve-create-result';

export interface ModelMappingsResolveCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
