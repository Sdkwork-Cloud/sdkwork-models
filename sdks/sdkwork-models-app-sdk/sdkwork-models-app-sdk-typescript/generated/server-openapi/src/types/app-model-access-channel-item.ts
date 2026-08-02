import type { AppModelAccessChannelOffering } from './app-model-access-channel-offering';

/** Official model endpoint or relay station returned by the app catalog API. */
export interface AppModelAccessChannelItem {
  id: string;
  code: string;
  name: string;
  kind: 'official' | 'relay' | 'custom';
  /** Public HTTP(S) API base URL. Credentials are never returned by this API. */
  baseUrl: string;
  description?: string | null;
  defaultVendorCode: string;
  defaultModelId: string;
  /** Agent provider ids that can consume this channel. */
  supportedAgentProviderIds: string[];
  offerings: AppModelAccessChannelOffering[];
  vendorCount: number;
  modelCount: number;
  sortOrder?: string | null;
}
