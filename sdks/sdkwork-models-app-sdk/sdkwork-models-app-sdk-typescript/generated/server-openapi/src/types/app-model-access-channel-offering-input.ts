import type { AppModelAccessChannelModelInput } from './app-model-access-channel-model-input';

/** One vendor and its supported model ids for an official endpoint or relay station. */
export interface AppModelAccessChannelOfferingInput {
  vendorCode: string;
  vendorName: string;
  modelIds?: string[];
  models?: AppModelAccessChannelModelInput[];
}
