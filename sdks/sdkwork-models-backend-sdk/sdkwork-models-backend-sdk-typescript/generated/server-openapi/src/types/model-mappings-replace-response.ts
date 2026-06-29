import type { ModelMappingsReplaceResult } from './model-mappings-replace-result';

export interface ModelMappingsReplaceResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
