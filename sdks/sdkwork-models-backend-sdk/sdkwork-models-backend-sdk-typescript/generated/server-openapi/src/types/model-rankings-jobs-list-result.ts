import type { ModelRankingRefreshJobHistoryPage } from './model-ranking-refresh-job-history-page';

/** Model rankings jobs list result schema exposed by Claw Router. */
export interface ModelRankingsJobsListResult {
  /** Business response code. */
  code: string;
  /** Data field on model rankings jobs list result. */
  data?: ModelRankingRefreshJobHistoryPage;
  /** Human-readable response message. */
  msg?: string;
}
