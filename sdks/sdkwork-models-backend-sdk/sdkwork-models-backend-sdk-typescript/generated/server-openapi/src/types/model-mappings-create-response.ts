import type { ModelMappingsCreateResult } from './model-mappings-create-result';

export interface ModelMappingsCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
