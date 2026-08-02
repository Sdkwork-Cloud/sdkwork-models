import type { AppModelAccessChannelModel } from './app-model-access-channel-model';

/** Official provider preset used to create a model access channel. */
export interface AppModelAccessChannelPreset {
  providerCode: string;
  providerDisplayName: string;
  protocol: string;
  vendorCode: string;
  vendorName: string;
  channelName: string;
  baseUrl: string;
  models: AppModelAccessChannelModel[];
  defaultModelId?: string | null;
  sortOrder: number;
}
