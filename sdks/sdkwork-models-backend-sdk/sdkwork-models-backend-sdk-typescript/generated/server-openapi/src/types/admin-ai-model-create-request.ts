import type { AdminAiModelRegionPriceRequest } from './admin-ai-model-region-price-request';

/** Request body for creating an AI model catalog entry. */
export interface AdminAiModelCreateRequest {
  vendorId: string;
  model: string;
  displayName?: string | null;
  type: 'chat' | 'image' | 'embedding' | 'audio' | 'video' | 'rerank' | 'moderation';
  regionPrices: AdminAiModelRegionPriceRequest[];
  contextTokens: string;
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
  /** Product usage scopes where the model can be applied. */
  usageScopes?: ('coding' | 'chat' | 'agent')[];
  /** Whether the model is shown in code IDE surfaces. */
  codingVisible?: boolean;
  releaseStage?: number;
  shelfState?: number;
  routingState?: number;
  replacementModel?: string | null;
}
