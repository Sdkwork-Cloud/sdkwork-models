import type { NoData } from './no-data';

/** Model mappings delete result schema exposed by Claw Router. */
export interface ModelMappingsDeleteResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
