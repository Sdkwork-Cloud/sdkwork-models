/** Model ranking refresh trigger response schema exposed by Claw Router. */
export interface ModelRankingRefreshTriggerResponse {
  /** Cache max age seconds field on model ranking refresh trigger response. */
  cacheMaxAgeSeconds: string;
  /** Generated count field on model ranking refresh trigger response. */
  generatedCount: string;
  /** Next refresh at field on model ranking refresh trigger response. */
  nextRefreshAt: string;
  /** Organization id field on model ranking refresh trigger response. */
  organizationId: string;
  /** Rank scope field on model ranking refresh trigger response. */
  rankScope: string;
  /** Refresh interval seconds field on model ranking refresh trigger response. */
  refreshIntervalSeconds: string;
  /** Snapshot date field on model ranking refresh trigger response. */
  snapshotDate: string;
  /** Snapshot period field on model ranking refresh trigger response. */
  snapshotPeriod: 'hourly' | 'daily' | 'weekly' | 'monthly';
  /** Source count field on model ranking refresh trigger response. */
  sourceCount: string;
  /** Result of the manual ranking worker run. */
  status: 'succeeded' | 'empty';
  /** Tenant id field on model ranking refresh trigger response. */
  tenantId: string;
  /** Whether a manual refresh worker run was started. */
  triggered: boolean;
  /** Window end field on model ranking refresh trigger response. */
  windowEnd: string;
  /** Window start field on model ranking refresh trigger response. */
  windowStart: string;
}
