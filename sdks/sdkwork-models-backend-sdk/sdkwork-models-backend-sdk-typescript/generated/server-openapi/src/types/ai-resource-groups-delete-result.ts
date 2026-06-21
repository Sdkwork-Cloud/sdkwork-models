import type { AdminAiResourceGroupDeleteResponse } from './admin-ai-resource-group-delete-response';

/** Ai resource groups delete result schema exposed by Claw Router. */
export interface AiResourceGroupsDeleteResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resource groups delete result. */
  data?: AdminAiResourceGroupDeleteResponse;
  /** Human-readable response message. */
  msg?: string;
}
