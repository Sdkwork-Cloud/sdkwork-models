import type { ModelsRefreshResult } from './models-refresh-result';

export interface ModelsRefreshResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
