import type { ModelRankingRefreshStatus } from './model-ranking-refresh-status';

/** Model rankings status retrieve result schema exposed by Claw Router. */
export interface ModelRankingsStatusRetrieveResult {
  /** Business response code. */
  code: string;
  /** Data field on model rankings status retrieve result. */
  data?: ModelRankingRefreshStatus;
  /** Human-readable response message. */
  msg?: string;
}
