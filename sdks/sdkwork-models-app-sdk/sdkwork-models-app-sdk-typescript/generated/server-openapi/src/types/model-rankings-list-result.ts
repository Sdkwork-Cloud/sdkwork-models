import type { NoData } from './no-data';

/** Model rankings list result schema exposed by Claw Router. */
export interface ModelRankingsListResult {
  /** Business response code. */
  code: string;
  /** No business data returned by this operation. */
  data?: NoData;
  /** Human-readable response message. */
  msg?: string;
}
