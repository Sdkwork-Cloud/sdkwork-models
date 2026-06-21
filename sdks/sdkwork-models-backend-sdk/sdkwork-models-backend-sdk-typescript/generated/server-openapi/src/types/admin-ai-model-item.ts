import type { AdminAiModelRegionPrice } from './admin-ai-model-region-price';

/** Persisted ai model snapshot returned by the backend. */
export interface AdminAiModelItem {
  /** Api format field on admin ai model item. */
  apiFormat: string | null;
  /** Calls field on admin ai model item. */
  calls: string;
  /** Capability intro field on admin ai model item. */
  capabilityIntro: string | null;
  /** Context tokens field on admin ai model item. */
  contextTokens: string | null;
  /** Description field on admin ai model item. */
  description: string | null;
  /** Product display name. Falls back to model when no display name is configured. */
  displayName: string;
  /** Id field on admin ai model item. */
  id: string;
  /** Input modalities field on admin ai model item. */
  inputModalities: string[];
  /** Limitations field on admin ai model item. */
  limitations: string[];
  /** Max output tokens field on admin ai model item. */
  maxOutputTokens: string | null;
  /** Modalities field on admin ai model item. */
  modalities: string[];
  /** Runtime model identifier used for provider calls, routing, and pricing keys. */
  model: string;
  /** Compatibility display alias. Equal to displayName. */
  name: string;
  /** Output modalities field on admin ai model item. */
  outputModalities: string[];
  /** Region prices field on admin ai model item. */
  regionPrices: AdminAiModelRegionPrice[];
  /** Release stage field on admin ai model item. */
  releaseStage: string | null;
  /** Replacement model field on admin ai model item. */
  replacementModel: string | null;
  /** Routing state field on admin ai model item. */
  routingState: string | null;
  /** Shelf state field on admin ai model item. */
  shelfState: string | null;
  /** Status field on admin ai model item. */
  status: 'active' | 'inactive';
  /** Supported languages field on admin ai model item. */
  supportedLanguages: string[];
  /** Supports json schema field on admin ai model item. */
  supportsJsonSchema: boolean;
  /** Supports streaming field on admin ai model item. */
  supportsStreaming: boolean;
  /** Supports tools field on admin ai model item. */
  supportsTools: boolean;
  /** Training data cutoff field on admin ai model item. */
  trainingDataCutoff: string | null;
  /** Type field on admin ai model item. */
  type: 'Chat' | 'Image' | 'Audio' | 'Embedding' | 'Music' | 'SoundEffect' | 'Video';
  /** Use cases field on admin ai model item. */
  useCases: string[];
  /** Vendor code field on admin ai model item. */
  vendorCode: string;
  /** Vendor id field on admin ai model item. */
  vendorId: string;
}
