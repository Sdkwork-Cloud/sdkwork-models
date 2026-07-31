/** Model ranking history entry schema exposed by Claw Router. */
export interface ModelRankingHistoryEntry {
  /** Catalog key field on model ranking history entry. */
  catalogKey: string;
  /** Color field on model ranking history entry. */
  color: string;
  /** Model field on model ranking history entry. */
  model: string;
  /** Rank field on model ranking history entry. */
  rank: number;
  /** Volume field on model ranking history entry. */
  volume: number;
}
