import type { AdminModelMappingMutationResponse } from './admin-model-mapping-mutation-response';

/** Model mappings update result schema exposed by Claw Router. */
export interface ModelMappingsUpdateResult {
  /** Business response code. */
  code: string;
  /** Data field on model mappings update result. */
  data?: AdminModelMappingMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
