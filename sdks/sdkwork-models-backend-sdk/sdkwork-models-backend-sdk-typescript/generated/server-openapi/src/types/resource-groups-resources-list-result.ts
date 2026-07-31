import type { NoData } from './no-data';

/** Resource groups resources list result schema exposed by Claw Router. */
export interface ResourceGroupsResourcesListResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
