import type { NoData } from './no-data';

/** Ai resource groups delete result schema exposed by Claw Router. */
export interface AiResourceGroupsDeleteResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
