/** Model ranking item schema exposed by Cloud Router. */
export interface ModelRankingItem {
  /** Base volume field on model ranking item. */
  baseVolume: number;
  /** Color field on model ranking item. */
  color: string;
  /** Context size field on model ranking item. */
  contextSize?: string;
  /** Cost field on model ranking item. */
  cost: number;
  /** Cost indicator field on model ranking item. */
  costIndicator: number;
  /** Currency field on model ranking item. */
  currency: string;
  /** Id field on model ranking item. */
  id: string;
  /** Is new field on model ranking item. */
  isNew: boolean;
  /** Latency field on model ranking item. */
  latency: number;
  /** License field on model ranking item. */
  license?: string;
  /** Modality field on model ranking item. */
  modality: string;
  /** Name field on model ranking item. */
  name: string;
  /** Prev rank field on model ranking item. */
  prevRank: number;
  /** Pricing field on model ranking item. */
  pricing?: string;
  /** Rank field on model ranking item. */
  rank: number;
  /** Requests field on model ranking item. */
  requests: number;
  /** Strengths field on model ranking item. */
  strengths: string[];
  /** Tokens field on model ranking item. */
  tokens: number;
  /** Trend score field on model ranking item. */
  trendScore?: number;
  /** Vendor field on model ranking item. */
  vendor: string;
  /** Vendor code field on model ranking item. */
  vendorCode: string;
  /** Win rate field on model ranking item. */
  winRate?: number;
}
