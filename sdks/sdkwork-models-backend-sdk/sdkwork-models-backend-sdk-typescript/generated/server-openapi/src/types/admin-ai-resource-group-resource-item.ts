/** Admin ai resource group resource item schema exposed by Claw Router. */
export interface AdminAiResourceGroupResourceItem {
  /** Api endpoint code field on admin ai resource group resource item. */
  apiEndpointCode?: string | null;
  /** Catalog key field on admin ai resource group resource item. */
  catalogKey?: string | null;
  /** Display name field on admin ai resource group resource item. */
  displayName: string;
  /** Id field on admin ai resource group resource item. */
  id: string;
  /** Member role field on admin ai resource group resource item. */
  memberRole: 'included' | 'optional' | 'fallback';
  /** Modality code field on admin ai resource group resource item. */
  modalityCode?: string | null;
  /** Model field on admin ai resource group resource item. */
  model?: string | null;
  /** Provider native model field on admin ai resource group resource item. */
  providerNativeModel?: string | null;
  /** Resource code field on admin ai resource group resource item. */
  resourceCode: string;
  /** Resource type field on admin ai resource group resource item. */
  resourceType: 'api_endpoint';
  /** Sort order field on admin ai resource group resource item. */
  sortOrder?: string | null;
  /** Status field on admin ai resource group resource item. */
  status: 'active' | 'disabled' | 'inactive';
  /** Vendor code field on admin ai resource group resource item. */
  vendorCode?: string | null;
}
