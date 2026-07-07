import type { NoData } from './no-data';

/** Model mappings resolve create result schema exposed by Claw Router. */
export interface ModelMappingsResolveCreateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
