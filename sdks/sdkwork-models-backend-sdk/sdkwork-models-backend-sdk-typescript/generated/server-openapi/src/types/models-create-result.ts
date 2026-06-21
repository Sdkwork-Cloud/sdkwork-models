import type { AdminAiModelMutationResponse } from './admin-ai-model-mutation-response';

/** Models create result schema exposed by Claw Router. */
export interface ModelsCreateResult {
  /** Business response code. */
  code: string;
  /** Data field on models create result. */
  data?: AdminAiModelMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
