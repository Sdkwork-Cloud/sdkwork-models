import type { ModelRankingRefreshJobItem } from './model-ranking-refresh-job-item';

/** Model ranking refresh status schema exposed by Cloud Router. */
export interface ModelRankingRefreshStatus {
  /** Cache max age seconds field on model ranking refresh status. */
  cacheMaxAgeSeconds: number;
  /** Generated at field on model ranking refresh status. */
  generatedAt: string;
  /** Generated count field on model ranking refresh status. */
  generatedCount: number;
  /** Latest job field on model ranking refresh status. */
  latestJob?: ModelRankingRefreshJobItem;
  /** Next refresh at field on model ranking refresh status. */
  nextRefreshAt: string;
  /** Organization id field on model ranking refresh status. */
  organizationId: string;
  /** Rank scope field on model ranking refresh status. */
  rankScope: string;
  /** Refresh interval seconds field on model ranking refresh status. */
  refreshIntervalSeconds: number;
  /** Snapshot date field on model ranking refresh status. */
  snapshotDate: string;
  /** Snapshot period field on model ranking refresh status. */
  snapshotPeriod: 'hourly' | 'daily' | 'weekly' | 'monthly';
  /** Source count field on model ranking refresh status. */
  sourceCount: number;
  /** Source tables field on model ranking refresh status. */
  sourceTables: string[];
  /** Status field on model ranking refresh status. */
  status: 'ready' | 'empty';
  /** Tenant id field on model ranking refresh status. */
  tenantId: string;
  /** Window end field on model ranking refresh status. */
  windowEnd: string;
  /** Window start field on model ranking refresh status. */
  windowStart: string;
}
