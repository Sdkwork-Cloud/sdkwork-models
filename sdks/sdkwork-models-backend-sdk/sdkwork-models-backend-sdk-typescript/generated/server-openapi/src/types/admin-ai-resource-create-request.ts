import type { AdminAiResourceMemberInput } from './admin-ai-resource-member-input';

/** Request body for creating an AI resource. */
export interface AdminAiResourceCreateRequest {
  resourceCode: string;
  resourceType: 'vendor' | 'modality' | 'api_endpoint' | 'model_api' | 'bundle';
  displayName: string;
  vendorCode?: string | null;
  modalityCode?: string | null;
  apiEndpointCode?: string | null;
  catalogKey?: string | null;
  model?: string | null;
  providerNativeModel?: string | null;
  compositionMode?: 'single' | 'any' | 'all' | null;
  status?: 'active' | 'disabled' | 'inactive' | null;
  sortOrder?: string | null;
  members?: AdminAiResourceMemberInput[];
}
