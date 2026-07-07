import type { NoData } from './no-data';

/** Model vendors create result schema exposed by Claw Router. */
export interface ModelVendorsCreateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
