import type { ModelRankingRefreshJobItem } from './model-ranking-refresh-job-item';
import type { PageInfo } from './page-info';

/** Model ranking refresh job history page schema exposed by Cloud Router. */
export interface ModelRankingRefreshJobHistoryPage {
  /** Items field on model ranking refresh job history page. */
  items: ModelRankingRefreshJobItem[];
  /** Page info field on model ranking refresh job history page. */
  pageInfo: PageInfo;
}
