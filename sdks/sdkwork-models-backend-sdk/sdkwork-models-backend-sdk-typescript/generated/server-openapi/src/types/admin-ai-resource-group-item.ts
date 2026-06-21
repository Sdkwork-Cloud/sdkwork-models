/** Admin ai resource group item schema exposed by Claw Router. */
export interface AdminAiResourceGroupItem {
  /** Capabilities field on admin ai resource group item. */
  capabilities?: string[];
  /** Capability field on admin ai resource group item. */
  capability?: string | null;
  /** Description field on admin ai resource group item. */
  description?: string | null;
  /** Dynamic field on admin ai resource group item. */
  dynamic: boolean;
  /** Group code field on admin ai resource group item. */
  groupCode: string;
  /** Group name field on admin ai resource group item. */
  groupName: string;
  /** Group type field on admin ai resource group item. */
  groupType: 'api_group';
  /** Id field on admin ai resource group item. */
  id: string;
  /** Resource count field on admin ai resource group item. */
  resourceCount: string;
  /** Selection mode field on admin ai resource group item. */
  selectionMode: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  /** Sort order field on admin ai resource group item. */
  sortOrder?: string | null;
  /** Status field on admin ai resource group item. */
  status: 'active' | 'disabled' | 'inactive';
  /** Vendor codes field on admin ai resource group item. */
  vendorCodes?: string[];
}
