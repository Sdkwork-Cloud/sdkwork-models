import type { NoData } from './no-data';

/** Models delete result schema exposed by Claw Router. */
export interface ModelsDeleteResult {
  /** Business response code. */
  code: string;
  /** No business data returned by this operation. */
  data?: NoData;
  /** Human-readable response message. */
  msg?: string;
}
