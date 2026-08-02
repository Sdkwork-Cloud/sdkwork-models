import type { AppModelAccessChannelOfferingInput } from './app-model-access-channel-offering-input';

/** Public model access channel metadata. Credentials are intentionally forbidden. */
export interface AppModelAccessChannelUpsertRequest {
  name: string;
  kind: 'official' | 'relay' | 'custom';
  /** Public HTTP(S) API base URL. API keys are forbidden. */
  baseUrl: string;
  description?: string | null;
  offerings: AppModelAccessChannelOfferingInput[];
  defaultVendorCode: string;
  defaultModelId: string;
  supportedAgentProviderIds: string[];
}
