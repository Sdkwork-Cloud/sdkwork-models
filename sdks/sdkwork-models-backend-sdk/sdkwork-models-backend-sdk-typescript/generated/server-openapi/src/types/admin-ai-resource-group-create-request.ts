import type { AdminAiResourceGroupMemberInput } from './admin-ai-resource-group-member-input';

/** Admin ai resource group create request schema exposed by Claw Router. */
export interface AdminAiResourceGroupCreateRequest {
  /** Description field on admin ai resource group create request. */
  description?: string | null;
  /** Group code field on admin ai resource group create request. */
  groupCode: string;
  /** Group name field on admin ai resource group create request. */
  groupName: string;
  /** Group type field on admin ai resource group create request. */
  groupType?: 'api_group';
  /** Members field on admin ai resource group create request. */
  members?: AdminAiResourceGroupMemberInput[];
  /** Selection mode field on admin ai resource group create request. */
  selectionMode?: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  /** Sort order field on admin ai resource group create request. */
  sortOrder?: string | null;
  /** Status field on admin ai resource group create request. */
  status?: 'active' | 'disabled' | 'inactive';
}
