import type { ModelRankingsPage } from './model-rankings-page';

/** Model rankings list result schema exposed by Claw Router. */
export interface ModelRankingsListResult {
  code: 0;
  data: unknown & ModelRankingsPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
