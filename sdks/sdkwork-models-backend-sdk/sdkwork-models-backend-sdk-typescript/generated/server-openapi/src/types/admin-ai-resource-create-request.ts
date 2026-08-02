import type { AdminAiResourceMemberInput } from './admin-ai-resource-member-input';

/** Request body for creating an AI resource. */
export interface AdminAiResourceCreateRequest {
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
  compositionMode?: 'single' | 'any' | 'all' | null;
  status?: 'active' | 'disabled' | 'inactive' | null;
  sortOrder?: string | null;
  members?: AdminAiResourceMemberInput[];
}
