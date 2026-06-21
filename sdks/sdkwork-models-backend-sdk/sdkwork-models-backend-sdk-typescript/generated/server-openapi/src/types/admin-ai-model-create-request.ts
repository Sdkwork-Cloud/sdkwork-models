import type { AdminAiModelRegionPrice } from './admin-ai-model-region-price';

/** Admin ai model create request schema exposed by Claw Router. */
export interface AdminAiModelCreateRequest {
  /** Api format field on admin ai model create request. */
  apiFormat?: string | null;
  /** Capability intro field on admin ai model create request. */
  capabilityIntro?: string | null;
  /** Positive token window, accepting plain integers or K/M suffixes. */
  contextTokens: string;
  /** Description field on admin ai model create request. */
  description?: string | null;
  /** Display name field on admin ai model create request. */
  displayName?: string | null;
  /** Input modalities field on admin ai model create request. */
  inputModalities?: string[];
  /** Limitations field on admin ai model create request. */
  limitations?: string[];
  /** Max output tokens field on admin ai model create request. */
  maxOutputTokens?: string | null;
  /** Modalities field on admin ai model create request. */
  modalities?: string[];
  /** Runtime model identifier used for provider calls, routing, and pricing keys. */
  model: string;
  /** Output modalities field on admin ai model create request. */
  outputModalities?: string[];
  /** Official reference prices by region. */
  regionPrices: AdminAiModelRegionPrice[];
  /** Release stage field on admin ai model create request. */
  releaseStage?: string | null;
  /** Replacement model field on admin ai model create request. */
  replacementModel?: string | null;
  /** Routing state field on admin ai model create request. */
  routingState?: string | null;
  /** Shelf state field on admin ai model create request. */
  shelfState?: string | null;
  /** Supported languages field on admin ai model create request. */
  supportedLanguages?: string[];
  /** Supports json schema field on admin ai model create request. */
  supportsJsonSchema?: boolean;
  /** Supports streaming field on admin ai model create request. */
  supportsStreaming?: boolean;
  /** Supports tools field on admin ai model create request. */
  supportsTools?: boolean;
  /** Training data cutoff field on admin ai model create request. */
  trainingDataCutoff?: string | null;
  /** Primary model modality shown in the admin console. */
  type: 'Chat' | 'Image' | 'Audio' | 'Embedding' | 'Music' | 'SoundEffect' | 'Video';
  /** Use cases field on admin ai model create request. */
  useCases?: string[];
  /** Vendor row id or vendor code selected in the admin console. */
  vendorId: string;
}
