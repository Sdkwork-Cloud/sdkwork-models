import type { AdminAiResourceGroupMutationResponse } from './admin-ai-resource-group-mutation-response';

/** Ai resource groups update result schema exposed by Claw Router. */
export interface AiResourceGroupsUpdateResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resource groups update result. */
  data?: AdminAiResourceGroupMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
