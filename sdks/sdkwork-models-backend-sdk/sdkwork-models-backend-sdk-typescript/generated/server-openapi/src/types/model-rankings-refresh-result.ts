import type { NoData } from './no-data';

/** Model rankings refresh result schema exposed by Claw Router. */
export interface ModelRankingsRefreshResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
