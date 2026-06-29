import type { ModelRankingsStatusRetrieveResult } from './model-rankings-status-retrieve-result';

export interface ModelRankingsStatusRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
