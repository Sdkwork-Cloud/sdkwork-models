import type { NoData } from './no-data';

/** Models create result schema exposed by Claw Router. */
export interface ModelsCreateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
