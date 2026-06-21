/** Model ranking item schema exposed by Claw Router. */
export interface ModelRankingItem {
  /** Base volume field on model ranking item. */
  baseVolume: string;
  /** Color field on model ranking item. */
  color: string;
  /** Context size field on model ranking item. */
  contextSize?: string | null;
  /** Cost field on model ranking item. */
  cost: number;
  /** Cost indicator field on model ranking item. */
  costIndicator: string;
  /** Currency field on model ranking item. */
  currency: string;
  /** Stable model catalog identity; must match ranking history catalogKey and must not include snapshot date prefixes. */
  id: string;
  /** Is new field on model ranking item. */
  isNew: boolean;
  /** Latency field on model ranking item. */
  latency: string;
  /** License field on model ranking item. */
  license?: 'Open Source' | 'Proprietary' | null;
  /** Modality field on model ranking item. */
  modality: 'LLM' | 'Image' | 'Audio' | 'Video' | 'Music' | 'Embedding' | 'Rerank' | 'Unknown';
  /** Name field on model ranking item. */
  name: string;
  /** Prev rank field on model ranking item. */
  prevRank: string;
  /** Pricing field on model ranking item. */
  pricing?: string | null;
  /** Rank field on model ranking item. */
  rank: string;
  /** Requests field on model ranking item. */
  requests: string;
  /** Strengths field on model ranking item. */
  strengths: string[];
  /** Tokens field on model ranking item. */
  tokens: string;
  /** Trend score field on model ranking item. */
  trendScore?: number | null;
  /** Vendor field on model ranking item. */
  vendor: string;
  /** Vendor code field on model ranking item. */
  vendorCode: string;
  /** Win rate field on model ranking item. */
  winRate?: number | null;
}
