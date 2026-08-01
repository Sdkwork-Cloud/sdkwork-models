import type { ReactNode } from 'react';

export type UnifiedAgentModelKind = 'built-in' | 'custom';

export interface UnifiedAgentProviderOption {
  id: string;
  label: string;
  disabled?: boolean;
}

export interface AgentModelConfigurationDraft {
  configurationId: string;
  vendorCode: string;
  baseUrl: string;
  apiKey: string;
  defaultModelId: string;
  supportedModelIds: string[];
  supportedProviderIds: string[];
  inputContextTokens?: number;
  outputContextTokens?: number;
  toolCallRounds?: number;
  supportsMultimodal: boolean;
}

export interface UnifiedAgentModelOption {
  id: string;
  configurationId?: string;
  modelId: string;
  label: string;
  description?: string;
  iconKey?: string;
  kind: UnifiedAgentModelKind;
  metadataLabel?: string;
  vendorCode?: string;
  baseUrl?: string;
  supportedModelIds?: readonly string[];
  supportedProviderIds?: readonly string[];
  inputContextTokens?: number;
  outputContextTokens?: number;
  toolCallRounds?: number;
  supportsMultimodal?: boolean;
  apiKeyConfigured?: boolean;
  disabled?: boolean;
}

export interface UnifiedAgentModelSelectorMessages {
  addModel: string;
  addModelTitle: string;
  advancedSettings: string;
  apiKeyLabel: string;
  apiKeyPlaceholder: string;
  apiKeyRequired: string;
  baseUrlInvalid: string;
  baseUrlLabel: string;
  baseUrlPlaceholder: string;
  builtInModels: string;
  cancel: string;
  close: string;
  createFailed: string;
  creating: string;
  customModels: string;
  customTag: string;
  defaultModelLabel: string;
  defaultModelPlaceholder: string;
  defaultModelRequired: string;
  getApiKey: string;
  inputContextLabel: string;
  modelAlreadyExists: string;
  modelSelectorLabel: string;
  multimodalLabel: string;
  noModels: string;
  notSupported: string;
  outputContextLabel: string;
  providerRequired: string;
  providerSection: string;
  selectFailed: string;
  submit: string;
  supportedModelsLabel: string;
  supportedModelsPlaceholder: string;
  supportedProvidersHint: string;
  supported: string;
  toolCallRoundsLabel: string;
  useSystemDefaultPlaceholder: string;
  vendorLabel: string;
  vendorPlaceholder: string;
  vendorRequired: string;
}

export interface UnifiedAgentModelSelectorProps {
  activeProviderId: string;
  providerOptions: readonly UnifiedAgentProviderOption[];
  options: readonly UnifiedAgentModelOption[];
  selectedModelOptionId: string;
  fallbackLabel: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectModelOption: (
    option: UnifiedAgentModelOption,
  ) => void | Promise<void>;
  messages: UnifiedAgentModelSelectorMessages;
  onCreateModelConfiguration?: (
    draft: AgentModelConfigurationDraft,
  ) => void | Promise<void>;
  onGetApiKey?: (vendorCode: string) => void;
  renderModelIcon?: (option: UnifiedAgentModelOption) => ReactNode;
  className?: string;
  disabled?: boolean;
}
