/** Response data returned after manually refreshing model ranking snapshots. */
export interface ModelRankingRefreshTriggerResponse {
  triggered: boolean;
  status: 'succeeded' | 'empty';
  tenantId: string;
  organizationId: string;
  rankScope: string;
  snapshotDate: string;
  snapshotPeriod: 'hourly' | 'daily' | 'weekly' | 'monthly';
  windowStart: string;
  windowEnd: string;
  generatedCount: string;
  sourceCount: string;
  refreshIntervalSeconds: string;
  cacheMaxAgeSeconds: string;
  nextRefreshAt: string;
}
