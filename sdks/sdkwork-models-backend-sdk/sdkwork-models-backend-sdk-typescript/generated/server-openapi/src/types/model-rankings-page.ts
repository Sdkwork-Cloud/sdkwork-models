import type { ModelRankingHistoryPoint } from './model-ranking-history-point';
import type { ModelRankingItem } from './model-ranking-item';
import type { ModelRankingsSource } from './model-rankings-source';
import type { PageInfo } from './page-info';

/** Model rankings page schema exposed by Claw Router. */
export interface ModelRankingsPage {
  /** History field on model rankings page. */
  history: ModelRankingHistoryPoint[];
  /** Items field on model rankings page. */
  items: ModelRankingItem[];
  /** Page info field on model rankings page. */
  pageInfo: PageInfo;
  /** Source field on model rankings page. */
  source: ModelRankingsSource;
}
