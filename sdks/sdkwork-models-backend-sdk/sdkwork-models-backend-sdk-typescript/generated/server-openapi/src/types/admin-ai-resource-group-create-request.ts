import type { AdminAiResourceGroupMemberInput } from './admin-ai-resource-group-member-input';

/** Request body for creating an AI resource group. */
export interface AdminAiResourceGroupCreateRequest {
  groupCode: string;
  groupName: string;
  groupType?: 'api_group' | null;
  selectionMode?: 'manual' | 'all' | 'any' | 'dynamic_all_api' | null;
  description?: string | null;
  sortOrder?: string | null;
  status?: 'active' | 'disabled' | 'inactive' | null;
  members?: AdminAiResourceGroupMemberInput[];
}
