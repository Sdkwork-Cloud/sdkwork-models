import type { AppModelAccessChannelItemData } from './app-model-access-channel-item-data';

/** Model access channel upsert result. */
export interface ModelAccessChannelUpsertResult {
  code: 0;
  data: unknown & AppModelAccessChannelItemData;
  /** Server-owned request correlation id. */
  traceId: string;
}
