/** Request body for manually refreshing model ranking snapshots. */
export interface ModelRankingRefreshTriggerRequest {
  rankScope?: string | null;
  snapshotPeriod?: 'hourly' | 'daily' | 'weekly' | 'monthly' | null;
  limit?: string | null;
  lookbackDays?: string | null;
  refreshIntervalSeconds?: string | null;
  cacheMaxAgeSeconds?: string | null;
}
