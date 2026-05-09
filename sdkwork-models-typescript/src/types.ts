export type ModelCapability =
  | "chat"
  | "embedding"
  | "image"
  | "audio"
  | "music"
  | "video"
  | "rerank"
  | "tool";

export type ModelModality =
  | "text"
  | "image"
  | "audio"
  | "music"
  | "video"
  | "embedding"
  | "rerank"
  | "tool";

export interface SourceEvidence {
  sourceUrl: string;
  observedAt: string;
  sourceHash?: string;
}

export interface BillingMeter {
  meterCode: string;
  displayName: string;
  modality: string;
  defaultUnitSize: string;
}

export interface ModelVendor {
  vendorCode: string;
  regionCode: string;
  displayName: string;
  legalName?: string;
  vendorType: string;
  marketScope: string;
  billingCurrency: string;
  billingJurisdiction: string;
  operatingRegions: string[];
  capabilities: ModelCapability[];
  openSource: boolean;
}

export interface ModelInfo {
  catalogKey: string;
  modelId: string;
  displayName: string;
  vendorCode: string;
  regionCode: string;
  familyCode: string;
  primaryCapability: ModelCapability;
  capabilities: ModelCapability[];
  inputModalities: ModelModality[];
  outputModalities: ModelModality[];
  apiFormat: string;
  lifecycle: string;
  releaseStage: string;
  shelfState: string;
  routingState: string;
  source: SourceEvidence;
}

export interface ModelPrice {
  priceId: string;
  priceSide: string;
  pricingScope: string;
  meterCode: string;
  unitSize: string;
  unitPrice: string;
  minimumQuantity: string;
  currency: string;
  effectiveFrom: string;
  source: SourceEvidence;
}

export interface ModelPricing {
  catalogKey: string;
  vendorCode: string;
  regionCode: string;
  modelId: string;
  currency: string;
  prices: ModelPrice[];
}

export interface VendorCatalog {
  vendorCode: string;
  regionCode: string;
  vendor: ModelVendor;
  models: ModelInfo[];
  pricing: ModelPricing[];
}

export interface VendorRegionRef {
  vendorCode: string;
  regionCode: string;
}

export interface ModelCatalog {
  catalogVersion: string;
  schemaVersion: string;
  meters: BillingMeter[];
  vendors: VendorCatalog[];
}

export interface ModelVendorIdentity {
  vendorCode: string;
  displayName: string;
  legalName?: string;
  vendorType: string;
  capabilities: ModelCapability[];
  openSource: boolean;
}
