import type { AdminModelMappingResolveResponse } from './admin-model-mapping-resolve-response';

/** Model mappings resolve create result schema exposed by Claw Router. */
export interface ModelMappingsResolveCreateResult {
  /** Business response code. */
  code: string;
  /** Data field on model mappings resolve create result. */
  data?: AdminModelMappingResolveResponse;
  /** Human-readable response message. */
  msg?: string;
}
