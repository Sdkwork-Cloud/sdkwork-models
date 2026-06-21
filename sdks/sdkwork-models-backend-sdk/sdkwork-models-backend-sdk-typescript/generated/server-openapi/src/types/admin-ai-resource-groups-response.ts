import type { AdminAiResourceGroupItem } from './admin-ai-resource-group-item';

/** Admin ai resource groups response schema exposed by Claw Router. */
export interface AdminAiResourceGroupsResponse {
  /** Items field on admin ai resource groups response. */
  items: AdminAiResourceGroupItem[];
}
