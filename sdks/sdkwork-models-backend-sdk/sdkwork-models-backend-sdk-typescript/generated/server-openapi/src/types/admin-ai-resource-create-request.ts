import type { AdminAiResourceMemberInput } from './admin-ai-resource-member-input';

/** Admin ai resource create request schema exposed by Claw Router. */
export interface AdminAiResourceCreateRequest {
  /** Api endpoint code field on admin ai resource create request. */
  apiEndpointCode?: string | null;
  /** Catalog key field on admin ai resource create request. */
  catalogKey?: string | null;
  /** Composition mode field on admin ai resource create request. */
  compositionMode?: 'single' | 'any' | 'all';
  /** Display name field on admin ai resource create request. */
  displayName: string;
  /** Members field on admin ai resource create request. */
  members?: AdminAiResourceMemberInput[];
  /** Modality code field on admin ai resource create request. */
  modalityCode?: string | null;
  /** Model field on admin ai resource create request. */
  model?: string | null;
  /** Provider native model field on admin ai resource create request. */
  providerNativeModel?: string | null;
  /** Stable normalized AI resource code. */
  resourceCode: string;
  /** Resource type field on admin ai resource create request. */
  resourceType: 'vendor' | 'modality' | 'api_endpoint' | 'model_api' | 'bundle';
  /** Sort order field on admin ai resource create request. */
  sortOrder?: string | null;
  /** Status field on admin ai resource create request. */
  status?: 'active' | 'disabled' | 'inactive';
  /** Vendor code field on admin ai resource create request. */
  vendorCode?: string | null;
}
