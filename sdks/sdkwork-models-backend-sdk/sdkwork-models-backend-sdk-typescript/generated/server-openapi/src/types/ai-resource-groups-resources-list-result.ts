import type { NoData } from './no-data';

/** Ai resource groups resources list result schema exposed by Claw Router. */
export interface AiResourceGroupsResourcesListResult {
  /** Business response code. */
  code: string;
  /** No business data returned by this operation. */
  data?: NoData;
  /** Human-readable response message. */
  msg?: string;
}
