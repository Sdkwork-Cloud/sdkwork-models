import type { AdminModelMappingMutationResponse } from './admin-model-mapping-mutation-response';

/** Model mappings create result schema exposed by Claw Router. */
export interface ModelMappingsCreateResult {
  /** Business response code. */
  code: string;
  /** Data field on model mappings create result. */
  data?: AdminModelMappingMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
