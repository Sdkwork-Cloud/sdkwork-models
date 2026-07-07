import type { NoData } from './no-data';

/** Ai resources create result schema exposed by Claw Router. */
export interface AiResourcesCreateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
