import type { AdminAiResourceGroupResourcesResponse } from './admin-ai-resource-group-resources-response';

/** Ai resource groups resources list result schema exposed by Claw Router. */
export interface AiResourceGroupsResourcesListResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resource groups resources list result. */
  data?: AdminAiResourceGroupResourcesResponse;
  /** Human-readable response message. */
  msg?: string;
}
