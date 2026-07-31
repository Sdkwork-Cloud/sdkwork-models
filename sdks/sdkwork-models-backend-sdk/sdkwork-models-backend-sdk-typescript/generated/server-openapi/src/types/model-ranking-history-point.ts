import type { ModelRankingHistoryEntry } from './model-ranking-history-entry';

/** Model ranking history point schema exposed by Claw Router. */
export interface ModelRankingHistoryPoint {
  /** Date field on model ranking history point. */
  date: string;
  /** Entries field on model ranking history point. */
  entries: ModelRankingHistoryEntry[];
  /** Index field on model ranking history point. */
  index: number;
}
