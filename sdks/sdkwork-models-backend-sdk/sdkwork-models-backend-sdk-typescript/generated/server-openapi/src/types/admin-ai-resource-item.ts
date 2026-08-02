import type { AdminAiResourceMemberItem } from './admin-ai-resource-member-item';

/** AI resource returned by the management API. */
export interface AdminAiResourceItem {
  id: string;
  resourceCode: string;
  resourceType: 'vendor' | 'modality' | 'api_endpoint' | 'model' | 'model_api' | 'bundle' | 'model_access_channel';
  displayName: string;
  vendorCode?: string | null;
  modalityCode?: string | null;
  apiEndpointCode?: string | null;
  catalogKey?: string | null;
  model?: string | null;
  providerNativeModel?: string | null;
  accessChannelKind?: 'official' | 'relay' | null;
  baseUrl?: string | null;
  defaultVendorCode?: string | null;
  defaultModelId?: string | null;
  supportedAgentProviderIds?: string[];
  description?: string | null;
  capability?: string | null;
  capabilities: string[];
  compositionMode: 'single' | 'any' | 'all';
  status: 'active' | 'disabled' | 'inactive';
  sortOrder?: string | null;
  members: AdminAiResourceMemberItem[];
}
