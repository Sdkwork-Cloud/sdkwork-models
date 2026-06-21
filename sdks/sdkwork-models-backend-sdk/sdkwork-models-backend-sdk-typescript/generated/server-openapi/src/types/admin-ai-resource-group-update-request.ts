import type { AdminAiResourceGroupMemberInput } from './admin-ai-resource-group-member-input';

/** Admin ai resource group update request schema exposed by Claw Router. */
export interface AdminAiResourceGroupUpdateRequest {
  /** Description field on admin ai resource group update request. */
  description?: string | null;
  /** Group code field on admin ai resource group update request. */
  groupCode?: string;
  /** Group name field on admin ai resource group update request. */
  groupName?: string;
  /** Group type field on admin ai resource group update request. */
  groupType?: 'api_group';
  /** Members field on admin ai resource group update request. */
  members?: AdminAiResourceGroupMemberInput[];
  /** Selection mode field on admin ai resource group update request. */
  selectionMode?: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  /** Sort order field on admin ai resource group update request. */
  sortOrder?: string | null;
  /** Status field on admin ai resource group update request. */
  status?: 'active' | 'disabled' | 'inactive';
}
