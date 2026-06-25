import type { AppModelCatalogPriceAvailability } from './app-model-catalog-price-availability';
import type { AppModelCatalogReferencePrice } from './app-model-catalog-reference-price';

/** App model catalog item schema exposed by Claw Router. */
export interface AppModelCatalogItem {
  /** Api format field on app model catalog item. */
  apiFormat: string | null;
  /** Capabilities field on app model catalog item. */
  capabilities: string[];
  /** Capability intro field on app model catalog item. */
  capabilityIntro: string | null;
  /** Catalog key field on app model catalog item. */
  catalogKey: string;
  /** Categories field on app model catalog item. */
  categories: ('Recommended' | 'Open Source' | 'Proprietary' | 'Free' | 'New')[];
  /** Context tokens field on app model catalog item. */
  contextTokens: string | null;
  /** Description field on app model catalog item. */
  description: string | null;
  /** Display name field on app model catalog item. */
  displayName: string;
  /** Groups field on app model catalog item. */
  groups: string[];
  /** Input modalities field on app model catalog item. */
  inputModalities: string[];
  /** Limitations field on app model catalog item. */
  limitations: string[];
  /** Max output tokens field on app model catalog item. */
  maxOutputTokens: string | null;
  /** Modalities field on app model catalog item. */
  modalities: string[];
  /** Model field on app model catalog item. */
  model: string;
  /** Complete public official reference prices keyed by regionCode and billing meter. Customer, upstream, provider, and channel prices are never exposed here. */
  officialReferencePrices: AppModelCatalogReferencePrice[];
  /** Output modalities field on app model catalog item. */
  outputModalities: string[];
  /** Price availability field on app model catalog item. */
  priceAvailability: AppModelCatalogPriceAvailability;
  /** Provider codes field on app model catalog item. */
  providerCodes: string[];
  /** Release stage field on app model catalog item. */
  releaseStage: string | null;
  /** Replacement model field on app model catalog item. */
  replacementModel: string | null;
  /** Routing state field on app model catalog item. */
  routingState: string | null;
  /** Shelf state field on app model catalog item. */
  shelfState: string | null;
  /** Supported languages field on app model catalog item. */
  supportedLanguages: string[];
  /** Supports json schema field on app model catalog item. */
  supportsJsonSchema: boolean;
  /** Supports streaming field on app model catalog item. */
  supportsStreaming: boolean;
  /** Supports tools field on app model catalog item. */
  supportsTools: boolean;
  /** Training data cutoff field on app model catalog item. */
  trainingDataCutoff: string | null;
  /** Use cases field on app model catalog item. */
  useCases: string[];
  /** Vendor field on app model catalog item. */
  vendor: string;
  /** Vendor code field on app model catalog item. */
  vendorCode: string;
}
