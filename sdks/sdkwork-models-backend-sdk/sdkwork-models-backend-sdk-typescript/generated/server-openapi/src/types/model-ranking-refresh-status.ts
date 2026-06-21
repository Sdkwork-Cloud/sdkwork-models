import type { JsonNull } from './json-null';
import type { ModelRankingRefreshLatestJob } from './model-ranking-refresh-latest-job';

/** Model ranking refresh status schema exposed by Claw Router. */
export interface ModelRankingRefreshStatus {
  /** Maximum cache age clients and services should use for this status snapshot. */
  cacheMaxAgeSeconds: string;
  /** Time when the ranking snapshot was generated. */
  generatedAt: string;
  /** Number of ranking rows generated in the selected snapshot. */
  generatedCount: string;
  /** Latest job field on model ranking refresh status. */
  latestJob: ModelRankingRefreshLatestJob | JsonNull;
  /** Planned next refresh time. */
  nextRefreshAt: string;
  /** Organization scope used by the selected ranking snapshot. */
  organizationId: string;
  /** Ranking scope, for example commercial-default. */
  rankScope: string;
  /** Planned refresh interval used by the ranking task. */
  refreshIntervalSeconds: string;
  /** Snapshot business date for the latest visible ranking. */
  snapshotDate: string;
  /** Snapshot period granularity, for example daily. */
  snapshotPeriod: string;
  /** Number of source usage rows represented by the selected snapshot. */
  sourceCount: string;
  /** Source tables field on model ranking refresh status. */
  sourceTables: string[];
  /** Published ranking read-model status for the latest visible snapshot. */
  status: 'ready' | 'empty' | 'unavailable';
  /** Tenant scope used by the selected ranking snapshot. */
  tenantId: string;
  /** Exclusive source aggregation window end. */
  windowEnd: string;
  /** Inclusive source aggregation window start. */
  windowStart: string;
}
