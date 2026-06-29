import type { ModelRankingsRefreshResult } from './model-rankings-refresh-result';

export interface ModelRankingsRefreshResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
