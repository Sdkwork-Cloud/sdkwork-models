import type { ModelRankingRefreshStatus } from './model-ranking-refresh-status';

/** Model rankings status retrieve result schema exposed by Cloud Router. */
export interface ModelRankingsStatusRetrieveResult {
  code: 0;
  data: unknown & ModelRankingRefreshStatus;
  /** Server-owned request correlation id. */
  traceId: string;
}
