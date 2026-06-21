import type { AdminAiResourceMemberInput } from './admin-ai-resource-member-input';

/** Admin ai resource update request schema exposed by Claw Router. */
export interface AdminAiResourceUpdateRequest {
  /** Api endpoint code field on admin ai resource update request. */
  apiEndpointCode?: string | null;
  /** Catalog key field on admin ai resource update request. */
  catalogKey?: string | null;
  /** Composition mode field on admin ai resource update request. */
  compositionMode?: 'single' | 'any' | 'all';
  /** Display name field on admin ai resource update request. */
  displayName?: string;
  /** Members field on admin ai resource update request. */
  members?: AdminAiResourceMemberInput[];
  /** Modality code field on admin ai resource update request. */
  modalityCode?: string | null;
  /** Model field on admin ai resource update request. */
  model?: string | null;
  /** Provider native model field on admin ai resource update request. */
  providerNativeModel?: string | null;
  /** Stable normalized AI resource code. */
  resourceCode?: string;
  /** Resource type field on admin ai resource update request. */
  resourceType?: 'vendor' | 'modality' | 'api_endpoint' | 'model_api' | 'bundle';
  /** Sort order field on admin ai resource update request. */
  sortOrder?: string | null;
  /** Status field on admin ai resource update request. */
  status?: 'active' | 'disabled' | 'inactive';
  /** Vendor code field on admin ai resource update request. */
  vendorCode?: string | null;
}
