import type { RankingVendorOptionsResponse } from './ranking-vendor-options-response';

/** Model vendors list result schema exposed by Claw Router. */
export interface ModelVendorsListResult {
  /** Business response code. */
  code: string;
  /** Data field on model vendors list result. */
  data?: RankingVendorOptionsResponse;
  /** Human-readable response message. */
  msg?: string;
}
