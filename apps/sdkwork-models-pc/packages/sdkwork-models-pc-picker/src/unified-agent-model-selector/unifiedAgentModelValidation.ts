import type {
  AgentModelConfigurationDraft,
  UnifiedAgentModelOption,
  UnifiedAgentProviderOption,
} from './unifiedAgentModelSelectorTypes';

export interface AgentModelConfigurationValidation {
  apiKeyRequired: boolean;
  baseUrlInvalid: boolean;
  duplicateModel: boolean;
  defaultModelRequired: boolean;
  providerRequired: boolean;
  vendorRequired: boolean;
}

function normalizeList(values: readonly string[]): string[] {
  const seen = new Set<string>();
  const normalized: string[] = [];
  for (const value of values) {
    const item = value.trim();
    const identity = item.toLowerCase();
    if (!item || seen.has(identity)) {
      continue;
    }
    seen.add(identity);
    normalized.push(item);
  }
  return normalized;
}

function isAbsoluteHttpUrl(value: string): boolean {
  try {
    const parsed = new URL(value);
    return parsed.protocol === 'https:' || parsed.protocol === 'http:';
  } catch {
    return false;
  }
}

function slug(value: string): string {
  const normalized = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/gu, '-')
    .replace(/^-+|-+$/gu, '');
  return normalized || 'custom';
}

export function parseSupportedModelIds(value: string): string[] {
  return normalizeList(value.split(/[\n,]+/gu));
}

export function createEmptyAgentModelConfigurationDraft(
  providerOptions: readonly UnifiedAgentProviderOption[],
): AgentModelConfigurationDraft {
  return {
    configurationId: '',
    vendorCode: '',
    baseUrl: '',
    apiKey: '',
    defaultModelId: '',
    supportedModelIds: [],
    supportedProviderIds: providerOptions
      .filter((provider) => !provider.disabled)
      .map((provider) => provider.id),
    supportsMultimodal: true,
  };
}

export function normalizeAgentModelConfigurationDraft(
  draft: AgentModelConfigurationDraft,
): AgentModelConfigurationDraft {
  const defaultModelId = draft.defaultModelId.trim();
  const supportedModelIds = normalizeList([
    defaultModelId,
    ...draft.supportedModelIds,
  ]);
  const vendorCode = draft.vendorCode.trim();
  return {
    ...draft,
    configurationId:
      draft.configurationId.trim()
      || `model.custom.${slug(vendorCode)}.${slug(defaultModelId)}`,
    vendorCode,
    baseUrl: draft.baseUrl.trim().replace(/\/+$/u, ''),
    apiKey: draft.apiKey.trim(),
    defaultModelId,
    supportedModelIds,
    supportedProviderIds: normalizeList(draft.supportedProviderIds),
  };
}

export function validateAgentModelConfigurationDraft(
  draft: AgentModelConfigurationDraft,
  options: readonly UnifiedAgentModelOption[],
  activeProviderId: string,
): AgentModelConfigurationValidation {
  const normalized = normalizeAgentModelConfigurationDraft(draft);
  const defaultIdentity = normalized.defaultModelId.toLowerCase();
  return {
    apiKeyRequired: !normalized.apiKey,
    baseUrlInvalid: !isAbsoluteHttpUrl(normalized.baseUrl),
    duplicateModel: Boolean(
      defaultIdentity
      && options.some((option) => option.modelId.trim().toLowerCase() === defaultIdentity),
    ),
    defaultModelRequired: !normalized.defaultModelId,
    providerRequired:
      normalized.supportedProviderIds.length === 0
      || !normalized.supportedProviderIds.includes(activeProviderId),
    vendorRequired: !normalized.vendorCode,
  };
}

export function isAgentModelConfigurationDraftValid(
  validation: AgentModelConfigurationValidation,
): boolean {
  return !Object.values(validation).some(Boolean);
}
