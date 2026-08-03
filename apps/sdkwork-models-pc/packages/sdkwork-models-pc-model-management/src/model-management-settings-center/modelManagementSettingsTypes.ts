import type {
  AgentModelAccessSelectorMessages,
  AgentModelCatalogOption,
  AgentProviderOption,
  ModelAccessChannel,
  ModelAccessChannelConfigurationDraft,
  ModelAccessChannelKind,
  OfficialModelVendorPreset,
} from '@sdkwork/models-pc-picker';

/**
 * A client-local per-engine model selection: which channel/model each Agent
 * engine is currently bound to.
 */
export interface ModelManagementEngineSelection {
  engineId: string;
  channelCode: string;
  modelId: string;
}

export interface ModelManagementSettingsMessages {
  title: string;
  description: string;
  officialSupplierLabel: string;
  officialSupplierDescription: string;
  /** Short badge shown next to the default official relay station entry. */
  defaultSupplierTag: string;
  relayStationsLabel: string;
  customConfigsLabel: string;
  addRelayStation: string;
  addCustomConfig: string;
  /** Opens the shared dialog on the official kind tab. */
  addOfficialSupplier: string;
  emptyRelayStations: string;
  emptyCustomConfigs: string;
  emptyOfficialSuppliers: string;
  officialVendorsLabel: string;
  officialVendorProtocol: string;
  officialVendorDefaultModel: string;
  noSelection: string;
  edit: string;
  delete: string;
  deleteConfirm: string;
  cancel: string;
  save: string;
  saving: string;
  deleting: string;
  saveFailed: string;
  deleteFailed: string;
  channelNameLabel: string;
  baseUrlLabel: string;
  apiKeyLabel: string;
  apiKeyConfiguredHint: string;
  defaultVendorLabel: string;
  defaultModelLabel: string;
  offeringsLabel: string;
  vendorsLabel: string;
  keyConfigured: string;
  keyNotConfigured: string;
  engineBindingsLabel: string;
  engineBindingsEmpty: string;
  kindLabel: string;
  modelCount: (count: number) => string;
}

export type ModelManagementChannelKind = Extract<
  ModelAccessChannelKind,
  'relay' | 'custom'
>;

export interface ModelManagementSettingsCenterProps {
  /** Official vendor presets of the BirdCoder platform (read-only display). */
  officialPresets: readonly OfficialModelVendorPreset[];
  /** Client-local channels (official/relay/custom) owned by the user. */
  channels: readonly ModelAccessChannel[];
  /** Agent engine options; used for bindings labels and provider defaults. */
  providerOptions: readonly AgentProviderOption[];
  /** Catalog models used by the inline form's known-model suggestions. */
  models: readonly AgentModelCatalogOption[];
  /** Per-engine bindings for the selected channel. */
  engineSelections: readonly ModelManagementEngineSelection[];
  /** Page chrome localized messages. */
  messages: ModelManagementSettingsMessages;
  /** Form localized messages (the picker's selector messages). */
  formMessages: AgentModelAccessSelectorMessages;
  /** Persists a channel (and its key) to the client-local store; resolves
   *  with the saved channel code so the center can select it. */
  onSaveChannel: (draft: ModelAccessChannelConfigurationDraft) => Promise<string | void>;
  /** Deletes a channel from the client-local store. */
  onDeleteChannel: (channel: ModelAccessChannel) => Promise<void>;
}

/** The BirdCoder official platform supplier's fixed identity. */
export const BIRDOODER_OFFICIAL_SUPPLIER_ID = 'birdcoder';
