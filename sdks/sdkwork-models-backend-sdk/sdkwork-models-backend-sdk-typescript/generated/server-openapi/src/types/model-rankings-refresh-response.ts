import type { ModelRankingRefreshTriggerResponse } from './model-ranking-refresh-trigger-response';

export interface ModelRankingsRefreshResponse {
  code: 0;
  data: unknown & ModelRankingRefreshTriggerResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
