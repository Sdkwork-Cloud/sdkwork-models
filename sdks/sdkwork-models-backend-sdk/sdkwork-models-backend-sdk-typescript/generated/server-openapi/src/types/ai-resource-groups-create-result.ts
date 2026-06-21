import type { AdminAiResourceGroupMutationResponse } from './admin-ai-resource-group-mutation-response';

/** Ai resource groups create result schema exposed by Claw Router. */
export interface AiResourceGroupsCreateResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resource groups create result. */
  data?: AdminAiResourceGroupMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
