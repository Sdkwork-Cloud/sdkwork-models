import type { NoData } from './no-data';

/** Model vendors list result schema exposed by Claw Router. */
export interface ModelVendorsListResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
