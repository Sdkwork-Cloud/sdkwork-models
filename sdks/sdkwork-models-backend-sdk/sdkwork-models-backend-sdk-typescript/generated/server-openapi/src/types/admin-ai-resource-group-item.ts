/** AI resource group returned by the management API. */
export interface AdminAiResourceGroupItem {
  id: string;
  groupCode: string;
  groupName: string;
  groupType: 'api_group';
  selectionMode: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description?: string | null;
  vendorCodes: string[];
  capability?: string | null;
  capabilities: string[];
  sortOrder?: string | null;
  status: 'active' | 'disabled' | 'inactive';
  resourceCount: string;
  dynamic: boolean;
}
