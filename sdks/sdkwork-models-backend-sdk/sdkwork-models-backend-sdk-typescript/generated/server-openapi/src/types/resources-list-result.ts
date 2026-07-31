import type { NoData } from './no-data';

/** Resources list result schema exposed by Claw Router. */
export interface ResourcesListResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
