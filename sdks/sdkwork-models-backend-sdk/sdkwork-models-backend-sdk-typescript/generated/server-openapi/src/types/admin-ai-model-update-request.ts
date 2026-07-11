import type { AdminAiModelRegionPriceRequest } from './admin-ai-model-region-price-request';

/** Request body for patching an AI model catalog entry. */
export interface AdminAiModelUpdateRequest {
  vendorId?: string | null;
  model?: string | null;
  displayName?: string | null;
  type?: 'chat' | 'image' | 'embedding' | 'audio' | 'video' | 'rerank' | 'moderation' | null;
  regionPrices?: AdminAiModelRegionPriceRequest[];
  status?: 'active' | 'disabled' | 'inactive' | null;
  contextTokens?: string | null;
  description?: string | null;
  /** Supported model modalities. */
  modalities?: string[];
  /** Supported input modalities. */
  inputModalities?: string[];
  /** Supported output modalities. */
  outputModalities?: string[];
  apiFormat?: string | null;
  capabilityIntro?: string | null;
  /** Model limitations. */
  limitations?: string[];
  /** Supported natural languages. */
  supportedLanguages?: string[];
  /** Recommended use cases. */
  useCases?: string[];
  trainingDataCutoff?: string | null;
  maxOutputTokens?: string | null;
  supportsStreaming?: boolean;
  supportsTools?: boolean;
  supportsJsonSchema?: boolean;
  releaseStage?: number;
  shelfState?: number;
  routingState?: number;
  replacementModel?: string | null;
}
