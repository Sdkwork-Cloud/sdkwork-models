import type { AdminModelMappingsResponse } from './admin-model-mappings-response';

/** Model mappings list result schema exposed by Claw Router. */
export interface ModelMappingsListResult {
  /** Business response code. */
  code: string;
  /** Data field on model mappings list result. */
  data?: AdminModelMappingsResponse;
  /** Human-readable response message. */
  msg?: string;
}
