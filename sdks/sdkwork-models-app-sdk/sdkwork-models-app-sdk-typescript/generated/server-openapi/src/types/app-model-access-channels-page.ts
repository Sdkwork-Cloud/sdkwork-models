import type { AppModelAccessChannelItem } from './app-model-access-channel-item';
import type { PageInfo } from './page-info';

/** Paginated official endpoints and relay stations. */
export interface AppModelAccessChannelsPage {
  items: AppModelAccessChannelItem[];
  /** Offset pagination metadata for model access channels. */
  pageInfo: PageInfo;
}
