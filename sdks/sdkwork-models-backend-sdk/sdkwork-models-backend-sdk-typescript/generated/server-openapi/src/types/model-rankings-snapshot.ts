import type { ModelRankingHistoryPoint } from './model-ranking-history-point';
import type { ModelRankingItem } from './model-ranking-item';
import type { ModelRankingsSource } from './model-rankings-source';

/** Model rankings snapshot schema exposed by Claw Router. */
export interface ModelRankingsSnapshot {
  /** History field on model rankings snapshot. */
  history: ModelRankingHistoryPoint[];
  /** Items field on model rankings snapshot. */
  items: ModelRankingItem[];
  /** Source field on model rankings snapshot. */
  source: ModelRankingsSource;
}
