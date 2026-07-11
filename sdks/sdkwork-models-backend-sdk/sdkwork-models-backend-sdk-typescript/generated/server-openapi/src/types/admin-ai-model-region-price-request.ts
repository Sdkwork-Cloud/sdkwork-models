/** Regional model pricing input. Decimal wire fields are encoded as strings. */
export interface AdminAiModelRegionPriceRequest {
  regionCode: string;
  currency: string;
  priceIn?: string | null;
  priceOut?: string | null;
  cacheReadPrice?: string | null;
  cacheWritePrice?: string | null;
}
