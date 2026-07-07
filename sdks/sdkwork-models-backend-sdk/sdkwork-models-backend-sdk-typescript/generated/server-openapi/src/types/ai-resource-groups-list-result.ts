import type { NoData } from './no-data';

/** Ai resource groups list result schema exposed by Claw Router. */
export interface AiResourceGroupsListResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
