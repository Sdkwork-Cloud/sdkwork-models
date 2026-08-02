import type { AppModelAccessChannelPresetsPage } from './app-model-access-channel-presets-page';

/** Official provider preset list result. */
export interface ModelAccessChannelPresetsListResult {
  code: 0;
  data: unknown & AppModelAccessChannelPresetsPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
