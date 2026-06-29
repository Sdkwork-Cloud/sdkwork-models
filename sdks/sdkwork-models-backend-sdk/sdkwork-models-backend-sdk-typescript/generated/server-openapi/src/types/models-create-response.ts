import type { ModelsCreateResult } from './models-create-result';

export interface ModelsCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
