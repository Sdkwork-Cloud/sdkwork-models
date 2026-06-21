/** Model rankings source schema exposed by Claw Router. */
export interface ModelRankingsSource {
  /** Cache max age seconds field on model rankings source. */
  cacheMaxAgeSeconds: string;
  /** Generated at field on model rankings source. */
  generatedAt: string;
  /** Next refresh at field on model rankings source. */
  nextRefreshAt: string;
  /** Observed at field on model rankings source. */
  observedAt: string;
  /** Rank scope field on model rankings source. */
  rankScope: string;
  /** Refresh interval seconds field on model rankings source. */
  refreshIntervalSeconds: string;
  /** Snapshot date field on model rankings source. */
  snapshotDate: string;
  /** Snapshot period field on model rankings source. */
  snapshotPeriod: string;
  /** Source description field on model rankings source. */
  sourceDescription: string;
  /** Source label field on model rankings source. */
  sourceLabel: string;
  /** Source tables field on model rankings source. */
  sourceTables: string[];
  /** Window end field on model rankings source. */
  windowEnd: string;
  /** Window start field on model rankings source. */
  windowStart: string;
}
