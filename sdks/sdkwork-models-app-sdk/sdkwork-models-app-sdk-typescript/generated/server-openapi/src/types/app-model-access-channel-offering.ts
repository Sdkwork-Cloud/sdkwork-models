import type { AppModelAccessChannelModel } from './app-model-access-channel-model';

/** Models from one vendor exposed by an official endpoint or relay station. */
export interface AppModelAccessChannelOffering {
  vendorCode: string;
  vendorName: string;
  models: AppModelAccessChannelModel[];
}
