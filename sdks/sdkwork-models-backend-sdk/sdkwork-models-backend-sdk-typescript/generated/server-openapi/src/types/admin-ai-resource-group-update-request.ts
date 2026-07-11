import type { AdminAiResourceGroupMemberInput } from './admin-ai-resource-group-member-input';

/** Request body for updating an AI resource group. */
export interface AdminAiResourceGroupUpdateRequest {
  groupCode?: string | null;
  groupName?: string | null;
  groupType?: 'api_group' | null;
  selectionMode?: 'manual' | 'all' | 'any' | 'dynamic_all_api' | null;
  description?: string | null;
  sortOrder?: string | null;
  status?: 'active' | 'disabled' | 'inactive' | null;
  members?: AdminAiResourceGroupMemberInput[];
}
