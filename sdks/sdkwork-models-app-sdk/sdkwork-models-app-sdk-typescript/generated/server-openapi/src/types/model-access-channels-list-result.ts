import type { AppModelAccessChannelsPage } from './app-model-access-channels-page';

/** Model access channel list result. */
export interface ModelAccessChannelsListResult {
  code: 0;
  data: unknown & AppModelAccessChannelsPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
