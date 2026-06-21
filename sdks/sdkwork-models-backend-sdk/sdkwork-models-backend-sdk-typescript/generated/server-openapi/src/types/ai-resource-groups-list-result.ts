import type { AdminAiResourceGroupsResponse } from './admin-ai-resource-groups-response';

/** Ai resource groups list result schema exposed by Claw Router. */
export interface AiResourceGroupsListResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resource groups list result. */
  data?: AdminAiResourceGroupsResponse;
  /** Human-readable response message. */
  msg?: string;
}
