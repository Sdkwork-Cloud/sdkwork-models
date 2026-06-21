import type { ModelRankingRefreshTriggerResponse } from './model-ranking-refresh-trigger-response';

/** Model rankings refresh result schema exposed by Claw Router. */
export interface ModelRankingsRefreshResult {
  /** Business response code. */
  code: string;
  /** Data field on model rankings refresh result. */
  data?: ModelRankingRefreshTriggerResponse;
  /** Human-readable response message. */
  msg?: string;
}
