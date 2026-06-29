import type { ModelsUpdateResult } from './models-update-result';

export interface ModelsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
