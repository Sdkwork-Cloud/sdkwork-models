import type { ModelRankingRefreshJobHistoryPage } from './model-ranking-refresh-job-history-page';

/** Model rankings jobs list result schema exposed by Cloud Router. */
export interface ModelRankingsJobsListResult {
  code: 0;
  data: unknown & ModelRankingRefreshJobHistoryPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
