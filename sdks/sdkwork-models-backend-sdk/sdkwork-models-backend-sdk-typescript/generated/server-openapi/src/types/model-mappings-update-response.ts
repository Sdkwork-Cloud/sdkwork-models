import type { ModelMappingsUpdateResult } from './model-mappings-update-result';

export interface ModelMappingsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
