import type { NoData } from './no-data';

/** Ai resource groups create result schema exposed by Claw Router. */
export interface AiResourceGroupsCreateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
