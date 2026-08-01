import type { AdminAiResourceMemberItem } from './admin-ai-resource-member-item';

/** AI resource returned by the management API. */
export interface AdminAiResourceItem {
  id: string;
  resourceCode: string;
  resourceType: 'vendor' | 'modality' | 'api_endpoint' | 'model_api' | 'bundle';
  displayName: string;
  vendorCode?: string | null;
  modalityCode?: string | null;
  apiEndpointCode?: string | null;
  catalogKey?: string | null;
  model?: string | null;
  providerNativeModel?: string | null;
  capability?: string | null;
  capabilities: string[];
  compositionMode: 'single' | 'any' | 'all';
  status: 'active' | 'disabled' | 'inactive';
  sortOrder?: string | null;
  members: AdminAiResourceMemberItem[];
}
