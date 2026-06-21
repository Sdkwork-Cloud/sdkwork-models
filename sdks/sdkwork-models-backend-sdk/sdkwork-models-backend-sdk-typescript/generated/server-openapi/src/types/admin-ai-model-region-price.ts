/** Regional official reference pricing returned by admin AI model payloads. */
export interface AdminAiModelRegionPrice {
  /** Optional official reference cache-read unit price in native official reference currency. */
  cacheReadPrice?: string;
  /** Optional official reference cache-write unit price in native official reference currency. */
  cacheWritePrice?: string;
  /** Official reference price currency code for this region. */
  currency: string;
  /** Official reference input unit price in native official reference currency. */
  priceIn: string;
  /** Official reference output unit price in native official reference currency. */
  priceOut: string;
  /** Model catalog pricing region code. */
  regionCode: string;
}
