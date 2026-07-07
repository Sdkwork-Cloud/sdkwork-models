import type { NoData } from './no-data';

/** Models delete result schema exposed by Claw Router. */
export interface ModelsDeleteResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
