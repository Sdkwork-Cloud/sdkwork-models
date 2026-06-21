import type { ModelRankingRefreshJobItem } from './model-ranking-refresh-job-item';

/** Model ranking refresh job history page schema exposed by Claw Router. */
export interface ModelRankingRefreshJobHistoryPage {
  /** Items field on model ranking refresh job history page. */
  items: ModelRankingRefreshJobItem[];
}
