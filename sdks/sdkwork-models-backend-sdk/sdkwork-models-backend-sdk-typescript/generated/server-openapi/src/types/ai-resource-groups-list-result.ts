import type { NoData } from './no-data';

/** Ai resource groups list result schema exposed by Claw Router. */
export interface AiResourceGroupsListResult {
  /** Business response code. */
  code: string;
  /** No business data returned by this operation. */
  data?: NoData;
  /** Human-readable response message. */
  msg?: string;
}
