import type { AdminModelMappingDeleteResponse } from './admin-model-mapping-delete-response';

/** Model mappings delete result schema exposed by Claw Router. */
export interface ModelMappingsDeleteResult {
  /** Business response code. */
  code: string;
  /** Data field on model mappings delete result. */
  data?: AdminModelMappingDeleteResponse;
  /** Human-readable response message. */
  msg?: string;
}
