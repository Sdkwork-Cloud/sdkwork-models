import type { NoData } from './no-data';

/** Model mappings list result schema exposed by Claw Router. */
export interface ModelMappingsListResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
