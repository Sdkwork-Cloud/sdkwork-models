import type { RankingVendorOption } from './ranking-vendor-option';

/** Ranking vendor options response schema exposed by Claw Router. */
export interface RankingVendorOptionsResponse {
  /** Items field on ranking vendor options response. */
  items: RankingVendorOption[];
}
