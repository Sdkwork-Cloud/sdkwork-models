import type { AdminAiResourceMutationResponse } from './admin-ai-resource-mutation-response';

/** Ai resources create result schema exposed by Claw Router. */
export interface AiResourcesCreateResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resources create result. */
  data?: AdminAiResourceMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
