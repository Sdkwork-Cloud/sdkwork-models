import type { NoData } from './no-data';

/** Models update result schema exposed by Claw Router. */
export interface ModelsUpdateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
