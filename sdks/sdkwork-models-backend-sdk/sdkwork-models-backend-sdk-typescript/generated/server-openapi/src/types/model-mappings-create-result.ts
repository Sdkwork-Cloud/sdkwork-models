import type { NoData } from './no-data';

/** Model mappings create result schema exposed by Claw Router. */
export interface ModelMappingsCreateResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
