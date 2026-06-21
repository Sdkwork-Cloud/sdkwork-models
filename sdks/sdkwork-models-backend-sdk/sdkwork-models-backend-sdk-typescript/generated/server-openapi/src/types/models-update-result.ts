import type { AdminAiModelMutationResponse } from './admin-ai-model-mutation-response';

/** Models update result schema exposed by Claw Router. */
export interface ModelsUpdateResult {
  /** Business response code. */
  code: string;
  /** Data field on models update result. */
  data?: AdminAiModelMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
