import type { NoData } from './no-data';

/** Model mappings replace result schema exposed by Claw Router. */
export interface ModelMappingsReplaceResult {
  code: 0;
  data: unknown & NoData;
  /** Server-owned request correlation id. */
  traceId: string;
}
