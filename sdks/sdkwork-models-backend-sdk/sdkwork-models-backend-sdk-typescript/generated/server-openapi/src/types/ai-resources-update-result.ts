import type { AdminAiResourceMutationResponse } from './admin-ai-resource-mutation-response';

/** Ai resources update result schema exposed by Claw Router. */
export interface AiResourcesUpdateResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resources update result. */
  data?: AdminAiResourceMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
