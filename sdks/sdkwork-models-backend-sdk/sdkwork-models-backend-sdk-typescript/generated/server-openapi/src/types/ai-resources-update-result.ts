import type { NoData } from './no-data';

/** Ai resources update result schema exposed by Claw Router. */
export interface AiResourcesUpdateResult {
  /** Business response code. */
  code: string;
  /** No business data returned by this operation. */
  data?: NoData;
  /** Human-readable response message. */
  msg?: string;
}
