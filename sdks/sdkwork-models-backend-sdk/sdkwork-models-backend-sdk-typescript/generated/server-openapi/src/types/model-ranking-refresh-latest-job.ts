/** Model ranking refresh latest job schema exposed by Claw Router. */
export interface ModelRankingRefreshLatestJob {
  /** Duration ms field on model ranking refresh latest job. */
  durationMs: string;
  /** Ended at field on model ranking refresh latest job. */
  endedAt: string;
  /** Failure count field on model ranking refresh latest job. */
  failureCount: string;
  /** Failure reason field on model ranking refresh latest job. */
  failureReason: string | null;
  /** Generated count field on model ranking refresh latest job. */
  generatedCount: string;
  /** Stable job execution identifier from ops_job_execution. */
  id: string;
  /** Job name, expected to be model_ranking_refresh. */
  jobName: string;
  /** Next refresh at field on model ranking refresh latest job. */
  nextRefreshAt: string;
  /** Organization id field on model ranking refresh latest job. */
  organizationId: string;
  /** Rank scope field on model ranking refresh latest job. */
  rankScope: string;
  /** Snapshot date field on model ranking refresh latest job. */
  snapshotDate: string;
  /** Snapshot period field on model ranking refresh latest job. */
  snapshotPeriod: string;
  /** Source count field on model ranking refresh latest job. */
  sourceCount: string;
  /** Started at field on model ranking refresh latest job. */
  startedAt: string;
  /** Latest matching ranking refresh job status. */
  status: 'succeeded' | 'failed' | 'empty' | 'skipped' | 'running';
  /** Success count field on model ranking refresh latest job. */
  successCount: string;
  /** Tenant id field on model ranking refresh latest job. */
  tenantId: string;
  /** Window end field on model ranking refresh latest job. */
  windowEnd: string;
  /** Window start field on model ranking refresh latest job. */
  windowStart: string;
}
