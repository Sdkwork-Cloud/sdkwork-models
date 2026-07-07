import type { NoData } from './no-data';

/** Ai resources update result schema exposed by Claw Router. */
export interface AiResourcesUpdateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
