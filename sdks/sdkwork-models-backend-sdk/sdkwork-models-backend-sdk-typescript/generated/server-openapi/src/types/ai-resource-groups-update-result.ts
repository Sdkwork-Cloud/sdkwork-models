import type { NoData } from './no-data';

/** Ai resource groups update result schema exposed by Claw Router. */
export interface AiResourceGroupsUpdateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
