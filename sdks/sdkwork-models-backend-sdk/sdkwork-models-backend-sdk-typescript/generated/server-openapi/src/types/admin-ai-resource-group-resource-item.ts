/** AI resource projected through a resource-group membership. */
export interface AdminAiResourceGroupResourceItem {
  id: string;
  resourceCode: string;
  resourceType: 'vendor' | 'modality' | 'api_endpoint' | 'model' | 'model_api' | 'bundle' | 'model_access_channel';
  displayName: string;
  vendorCode?: string | null;
  modalityCode?: string | null;
  apiEndpointCode?: string | null;
  method?: string | null;
  path?: string | null;
  catalogKey?: string | null;
  model?: string | null;
  providerNativeModel?: string | null;
  status: 'active' | 'disabled' | 'inactive';
  sortOrder?: string | null;
  memberRole: 'included' | 'optional' | 'fallback';
}
