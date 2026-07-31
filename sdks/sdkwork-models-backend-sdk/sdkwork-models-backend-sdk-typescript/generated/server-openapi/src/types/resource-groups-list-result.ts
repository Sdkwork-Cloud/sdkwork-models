import type { NoData } from './no-data';

/** Resource groups list result schema exposed by Claw Router. */
export interface ResourceGroupsListResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
