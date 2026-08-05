import type { ReactNode } from 'react';

export type AgentModelCatalogSource = 'database' | 'fallback' | 'custom';

export type ModelAccessChannelKind = 'official' | 'relay' | 'custom';

export type ModelCatalogSortOrder = number | string | null;

export interface AgentProviderOption {
  id: string;
  label: string;
  disabled?: boolean;
}

export interface ModelVendor {
  code: string;
  name: string;
  iconKey?: string;
  searchTerms?: readonly string[];
  sortOrder?: ModelCatalogSortOrder;
}

/** Generated from a direct provider entry owned by sdkwork-models. */
export interface OfficialModelVendorPreset {
  providerCode: string;
  providerDisplayName: string;
  protocol: string;
  vendorCode: string;
  vendorName: string;
  channelName: string;
  baseUrl: string;
  /** Current public model rows returned by the authoritative Models catalog. */
  models?: readonly OfficialModelVendorPresetModel[];
  /** The catalog's preferred default model for this official vendor. */
  defaultModelId?: string | null;
  sortOrder: number;
}

export interface OfficialModelVendorPresetModel {
  catalogKey?: string;
  model: string;
  displayName: string;
  sortOrder?: ModelCatalogSortOrder;
}

export interface AgentModelCatalogOption {
  id: string;
  modelId: string;
  label: string;
  vendorCode: string;
  vendorName: string;
  catalogKey?: string;
  catalogVersion?: string;
  contextTokens?: number;
  description?: string;
  iconKey?: string;
  kind?: 'built-in' | 'custom';
  metadataLabel?: string;
  inputModalities?: readonly string[];
  maxOutputTokens?: number;
  modalities?: readonly string[];
  outputModalities?: readonly string[];
  releaseStage?: string;
  source?: AgentModelCatalogSource;
  sourceObservedAt?: string;
  searchTerms?: readonly string[];
  sortOrder?: ModelCatalogSortOrder;
  rankScore?: number;
  supportedAgentProviderIds?: readonly string[];
  supportsTools?: boolean;
  toolCallRounds?: number;
  disabled?: boolean;
}

export interface ModelOfferingModel {
  catalogKey?: string;
  model: string;
  displayName: string;
  modelOptionId?: string;
  sortOrder?: ModelCatalogSortOrder;
  /** Context window in tokens; preserved from catalog / import metadata. */
  contextTokens?: number;
  maxOutputTokens?: number;
  toolCallRounds?: number;
}

export interface ModelOffering {
  vendorCode: string;
  vendorName: string;
  models: readonly ModelOfferingModel[];
}

/** Public catalog projection. Credentials must never be added to this type. */
export interface ModelAccessChannel {
  id: string;
  code?: string;
  name: string;
  kind: ModelAccessChannelKind;
  offerings: readonly ModelOffering[];
  defaultVendorCode?: string;
  defaultModelId?: string;
  apiKeyConfigured?: boolean;
  baseUrl?: string;
  description?: string | null;
  disabled?: boolean;
  isCustom?: boolean;
  searchTerms?: readonly string[];
  sortOrder?: ModelCatalogSortOrder;
  supportedAgentProviderIds: readonly string[];
  vendorCount?: number;
  modelCount?: number;
  /**
   * Where the channel projection came from. Fallback channels are derived from
   * generated presets and stay usable for model selection resolution, but the
   * picker lists only user-added `database`/`custom` channels.
   */
  source?: 'database' | 'custom' | 'fallback';
}

export interface ModelOfferingConfigurationModelDraft {
  modelId: string;
  displayName: string;
  /** Context window in tokens; preserved from catalog / import metadata. */
  contextTokens?: number;
  maxOutputTokens?: number;
  toolCallRounds?: number;
}

export interface ModelOfferingConfigurationDraft {
  vendorCode: string;
  vendorName: string;
  /** Ordered UI and configuration authority. */
  models: ModelOfferingConfigurationModelDraft[];
  /** Compatibility mirror for SDK consumers that still persist model IDs only. */
  modelIds: string[];
}

/**
 * Write-only configuration command. The API key is submitted through the host
 * callback and is intentionally absent from ModelAccessChannel.
 */
export interface ModelAccessChannelConfigurationDraft {
  channelId: string;
  kind: ModelAccessChannelKind;
  name: string;
  description: string;
  baseUrl: string;
  apiKey: string;
  apiKeyConfigured: boolean;
  offerings: ModelOfferingConfigurationDraft[];
  defaultVendorCode: string;
  defaultModelId: string;
  supportedAgentProviderIds: string[];
}

export interface AgentModelAccessSelection {
  channel: ModelAccessChannel;
  model: AgentModelCatalogOption;
  offering: ModelOffering;
  offeredModel: ModelOfferingModel;
}

export type AgentModelAccessSelectionOutcome =
  | void
  | {
      status: 'configuration-required';
      channelId?: string;
    };

export interface ModelAccessApiKeyRequestContext {
  channelId?: string;
  kind: ModelAccessChannelKind;
  vendorCode?: string;
}

export interface AgentModelAccessSelectorMessages {
  accessChannelsTab: string;
  addAccessChannel: string;
  addKnownModel: string;
  addModel: string;
  addVendor: string;
  apiKeyConfiguredHint: string;
  apiKeyLabel: string;
  apiKeyPlaceholder: string;
  apiKeyRequired: string;
  atLeastOneOfferingRequired: string;
  back: string;
  baseUrlInvalid: string;
  baseUrlLabel: string;
  baseUrlPlaceholder: string;
  builtInModels: string;
  cancel: string;
  channelKindLabel: string;
  channelNameLabel: string;
  channelNamePlaceholder: string;
  channelNameRequired: string;
  clearSearch: string;
  close: string;
  createAccessChannelTitle: string;
  createFailed: string;
  creating: string;
  customModels: string;
  customTag: string;
  customChannelDescription: string;
  customChannelLabel: string;
  customChannels: string;
  defaultModelLabel: string;
  defaultModelPlaceholder: string;
  defaultModelRequired: string;
  defaultVendorLabel: string;
  deleteChannel: string;
  deleteChannelConfirm: string;
  duplicateVendor: string;
  editAccessChannel: string;
  editAccessChannelTitle: string;
  getApiKey: string;
  modelAccessSelectorLabel: string;
  modelCount: (count: number) => string;
  modelDisplayNameLabel: string;
  modelIdLabel: string;
  modelsForVendorLabel: string;
  modelsForVendorPlaceholder: string;
  modelsTab: string;
  moreModels: string;
  moveModelDown: string;
  moveModelUp: string;
  noAccessChannels: string;
  noKnownModels: string;
  noModels: string;
  noSearchResults: string;
  officialChannelDescription: string;
  officialChannelLabel: string;
  officialChannels: string;
  officialConfigurationHint: string;
  officialVendorLabel: string;
  officialVendorPlaceholder: string;
  offeringsHint: string;
  offeringsLabel: string;
  previewTag: string;
  providerRequired: string;
  providerSection: string;
  relayChannelDescription: string;
  relayChannelLabel: string;
  relayChannels: string;
  removeModel: string;
  removeVendor: string;
  saveChanges: string;
  saving: string;
  searchPlaceholder: string;
  selectFailed: string;
  supportedProvidersHint: string;
  vendorCodeLabel: string;
  vendorCodePlaceholder: string;
  vendorNameLabel: string;
  vendorNamePlaceholder: string;
  vendorRequired: string;
}

export interface AgentModelAccessSelectorProps {
  activeProviderId: string;
  providerOptions: readonly AgentProviderOption[];
  /** Optional vendor directory used by the configuration form. */
  vendorOptions?: readonly ModelVendor[];
  /** Database-backed models. A non-empty value is authoritative. */
  models: readonly AgentModelCatalogOption[];
  /** Optional explicit fallback. The generated mainstream catalog is the default. */
  fallbackModels?: readonly AgentModelCatalogOption[];
  /** Database-backed channels. Empty means generated official fallback channels. */
  accessChannels: readonly ModelAccessChannel[];
  /**
   * Official vendor presets from the injected Models App SDK catalog. A non-empty
   * value is authoritative; the generated sdkwork-models presets remain the
   * deterministic fallback when the runtime catalog is unavailable or empty.
   */
  officialVendorPresets?: readonly OfficialModelVendorPreset[];
  selectedModelOptionId: string;
  selectedAccessChannelId?: string;
  fallbackLabel: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectModelAccess: (
    selection: AgentModelAccessSelection,
  ) => AgentModelAccessSelectionOutcome | Promise<AgentModelAccessSelectionOutcome>;
  messages: AgentModelAccessSelectorMessages;
  onCreateAccessChannel?: (
    draft: ModelAccessChannelConfigurationDraft,
  ) => void | Promise<void>;
  onUpdateAccessChannel?: (
    draft: ModelAccessChannelConfigurationDraft,
  ) => void | Promise<void>;
  /** Removes a client-local channel; only wired for user-owned channels. */
  onDeleteAccessChannel?: (
    channel: ModelAccessChannel,
  ) => void | Promise<void>;
  onGetApiKey?: (context: ModelAccessApiKeyRequestContext) => void;
  /** Called as the user types so the host can query the authoritative catalog. */
  onSearchQueryChange?: (query: string) => void;
  /** Indicates that an authoritative search request is in flight. */
  isSearchLoading?: boolean;
  renderModelIcon?: (model: AgentModelCatalogOption) => ReactNode;
  renderChannelIcon?: (channel: ModelAccessChannel) => ReactNode;
  className?: string;
  disabled?: boolean;
}
