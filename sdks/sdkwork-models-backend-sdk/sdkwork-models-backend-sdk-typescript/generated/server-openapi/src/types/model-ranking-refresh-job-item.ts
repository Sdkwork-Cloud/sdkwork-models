/** Model ranking refresh job item schema exposed by Claw Router. */
export interface ModelRankingRefreshJobItem {
  /** Duration ms field on model ranking refresh job item. */
  durationMs: number;
  /** Ended at field on model ranking refresh job item. */
  endedAt: string;
  /** Failure count field on model ranking refresh job item. */
  failureCount: number;
  /** Failure reason field on model ranking refresh job item. */
  failureReason?: string;
  /** Generated count field on model ranking refresh job item. */
  generatedCount: number;
  /** Id field on model ranking refresh job item. */
  id: string;
  /** Job name field on model ranking refresh job item. */
  jobName: string;
  /** Next refresh at field on model ranking refresh job item. */
  nextRefreshAt: string;
  /** Organization id field on model ranking refresh job item. */
  organizationId: string;
  /** Rank scope field on model ranking refresh job item. */
  rankScope: string;
  /** Snapshot date field on model ranking refresh job item. */
  snapshotDate: string;
  /** Snapshot period field on model ranking refresh job item. */
  snapshotPeriod: 'hourly' | 'daily' | 'weekly' | 'monthly';
  /** Source count field on model ranking refresh job item. */
  sourceCount: number;
  /** Started at field on model ranking refresh job item. */
  startedAt: string;
  /** Status field on model ranking refresh job item. */
  status: 'succeeded' | 'failed' | 'skipped' | 'empty' | 'running';
  /** Success count field on model ranking refresh job item. */
  successCount: number;
  /** Tenant id field on model ranking refresh job item. */
  tenantId: string;
  /** Window end field on model ranking refresh job item. */
  windowEnd: string;
  /** Window start field on model ranking refresh job item. */
  windowStart: string;
}
