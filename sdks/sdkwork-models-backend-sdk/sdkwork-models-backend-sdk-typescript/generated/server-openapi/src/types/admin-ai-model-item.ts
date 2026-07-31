import type { AdminAiModelRegionPrice } from './admin-ai-model-region-price';

/** Admin ai model item schema exposed by Claw Router. */
export interface AdminAiModelItem {
  /** Api format field on admin ai model item. */
  apiFormat: string | null;
  /** Calls field on admin ai model item. */
  calls: string;
  /** Capability intro field on admin ai model item. */
  capabilityIntro: string | null;
  /** Context tokens field on admin ai model item. */
  contextTokens: number | null;
  /** Description field on admin ai model item. */
  description: string | null;
  /** Display name field on admin ai model item. */
  displayName: string;
  /** Id field on admin ai model item. */
  id: string;
  /** Input modalities field on admin ai model item. */
  inputModalities: string[];
  /** Limitations field on admin ai model item. */
  limitations: string[];
  /** Max output tokens field on admin ai model item. */
  maxOutputTokens: number | null;
  /** Modalities field on admin ai model item. */
  modalities: string[];
  /** Model field on admin ai model item. */
  model: string;
  /** Name field on admin ai model item. */
  name: string;
  /** Output modalities field on admin ai model item. */
  outputModalities: string[];
  /** Region prices field on admin ai model item. */
  regionPrices: AdminAiModelRegionPrice[];
  /** Release stage field on admin ai model item. */
  releaseStage: number | null;
  /** Replacement model field on admin ai model item. */
  replacementModel: string | null;
  /** Routing state field on admin ai model item. */
  routingState: number | null;
  /** Shelf state field on admin ai model item. */
  shelfState: number | null;
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
