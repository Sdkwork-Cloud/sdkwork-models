import type { AdminAiResourceMemberItem } from './admin-ai-resource-member-item';

/** Admin ai resource item schema exposed by Claw Router. */
export interface AdminAiResourceItem {
  /** Api endpoint code field on admin ai resource item. */
  apiEndpointCode?: string;
  /** Capabilities field on admin ai resource item. */
  capabilities?: string[];
  /** Capability field on admin ai resource item. */
  capability?: string | null;
  /** Catalog key field on admin ai resource item. */
  catalogKey?: string;
  /** Composition mode field on admin ai resource item. */
  compositionMode: 'single' | 'any' | 'all';
  /** Display name field on admin ai resource item. */
  displayName: string;
  /** Id field on admin ai resource item. */
  id: string;
  /** Members field on admin ai resource item. */
  members: AdminAiResourceMemberItem[];
  /** Modality code field on admin ai resource item. */
  modalityCode?: string;
  /** Model field on admin ai resource item. */
  model?: string;
  /** Provider native model field on admin ai resource item. */
  providerNativeModel?: string;
  /** Resource code field on admin ai resource item. */
  resourceCode: string;
  /** Resource type field on admin ai resource item. */
  resourceType: 'vendor' | 'modality' | 'api_endpoint' | 'model_api' | 'bundle';
  /** Sort order field on admin ai resource item. */
  sortOrder?: string;
  /** Status field on admin ai resource item. */
  status: 'active' | 'disabled' | 'inactive';
  /** Vendor code field on admin ai resource item. */
  vendorCode?: string;
}
