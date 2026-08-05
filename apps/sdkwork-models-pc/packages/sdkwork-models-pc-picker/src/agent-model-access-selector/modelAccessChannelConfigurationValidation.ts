import type {
  AgentModelCatalogOption,
  AgentProviderOption,
  ModelAccessChannel,
  ModelAccessChannelConfigurationDraft,
  ModelAccessChannelKind,
  ModelOfferingConfigurationDraft,
  ModelOfferingConfigurationModelDraft,
} from './agentModelAccessSelectorTypes';

export interface ModelAccessChannelConfigurationValidation {
  apiKeyRequired: boolean;
  baseUrlInvalid: boolean;
  channelNameRequired: boolean;
  defaultModelRequired: boolean;
  duplicateVendor: boolean;
  offeringModelsRequired: boolean;
  officialVendorUnsupported: boolean;
  officialVendorCountInvalid: boolean;
  offeringsRequired: boolean;
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

// Monotonic counter keeps the non-ASCII slug fallback unique even when two
// channels are created within the same millisecond.
let customSlugSequence = 0;

function slug(value: string): string {
  const normalized = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/gu, '-')
    .replace(/^-+|-+$/gu, '');
  // Names without ASCII slug characters (for example CJK-only names) must not
  // collapse to the same generic "custom" identity: every saved channel needs
  // a distinct code or later saves silently overwrite earlier ones.
  return normalized || `custom-${Date.now().toString(36)}-${(customSlugSequence++).toString(36)}`;
}

function isAbsoluteHttpUrl(value: string): boolean {
  try {
    const parsed = new URL(value);
    return parsed.protocol === 'https:' || parsed.protocol === 'http:';
  } catch {
    return false;
  }
}

export function parseModelOfferingIds(value: string): string[] {
  return normalizeList(value.split(/[\n,]+/gu));
}

function normalizeConfigurationModels(
  models: readonly ModelOfferingConfigurationModelDraft[],
  compatibilityModelIds: readonly string[],
): ModelOfferingConfigurationModelDraft[] {
  const source: readonly ModelOfferingConfigurationModelDraft[] = models.length > 0
    ? models
    : compatibilityModelIds.map((modelId) => (
        { modelId, displayName: modelId } satisfies ModelOfferingConfigurationModelDraft
      ));
  const seen = new Set<string>();
  const normalized: ModelOfferingConfigurationModelDraft[] = [];
  for (const model of source) {
    const modelId = model.modelId.trim();
    const identity = modelId.toLowerCase();
    if (!modelId || seen.has(identity)) {
      continue;
    }
    seen.add(identity);
    normalized.push({
      modelId,
      displayName: model.displayName.trim() || modelId,
      ...(model.contextTokens == null ? {} : { contextTokens: model.contextTokens }),
      ...(model.maxOutputTokens == null ? {} : { maxOutputTokens: model.maxOutputTokens }),
      ...(model.toolCallRounds == null ? {} : { toolCallRounds: model.toolCallRounds }),
    });
  }
  return normalized;
}

export function createModelOfferingConfigurationDraft(
  vendorCode = '',
  vendorName = '',
  models: readonly Pick<
    AgentModelCatalogOption,
    'label' | 'modelId' | 'contextTokens' | 'maxOutputTokens' | 'toolCallRounds'
  >[] = [],
): ModelOfferingConfigurationDraft {
  const configurationModels = models.map((model) => ({
    modelId: model.modelId,
    displayName: model.label,
    ...(model.contextTokens == null ? {} : { contextTokens: model.contextTokens }),
    ...(model.maxOutputTokens == null ? {} : { maxOutputTokens: model.maxOutputTokens }),
    ...(model.toolCallRounds == null ? {} : { toolCallRounds: model.toolCallRounds }),
  }));
  return {
    vendorCode,
    vendorName,
    models: configurationModels,
    modelIds: configurationModels.map((model) => model.modelId),
  };
}

export function replaceModelOfferingConfigurationModels(
  offering: ModelOfferingConfigurationDraft,
  models: readonly ModelOfferingConfigurationModelDraft[],
): ModelOfferingConfigurationDraft {
  const normalizedModels = normalizeConfigurationModels(models, []);
  return {
    ...offering,
    models: normalizedModels,
    modelIds: normalizedModels.map((model) => model.modelId),
  };
}

export function createEmptyModelAccessChannelConfigurationDraft(
  providerOptions: readonly AgentProviderOption[],
  kind: ModelAccessChannelKind = 'official',
): ModelAccessChannelConfigurationDraft {
  return {
    channelId: '',
    kind,
    name: '',
    description: '',
    baseUrl: '',
    apiKey: '',
    apiKeyConfigured: false,
    offerings: [createModelOfferingConfigurationDraft()],
    defaultVendorCode: '',
    defaultModelId: '',
    supportedAgentProviderIds: providerOptions
      .filter((provider) => !provider.disabled)
      .map((provider) => provider.id),
  };
}

export function createModelAccessChannelConfigurationDraft(
  channel: ModelAccessChannel,
): ModelAccessChannelConfigurationDraft {
  const configuredDefaultOffering = channel.defaultVendorCode
    ? channel.offerings.find((offering) => (
        offering.vendorCode.trim().toLowerCase()
          === channel.defaultVendorCode?.trim().toLowerCase()
      ))
    : undefined;
  const defaultOffering = configuredDefaultOffering ?? channel.offerings[0];
  const configuredDefaultModel = channel.defaultModelId
    ? defaultOffering?.models.find((model) => (
        model.model.trim().toLowerCase() === channel.defaultModelId?.trim().toLowerCase()
      ))
    : undefined;
  const defaultModel = configuredDefaultModel ?? defaultOffering?.models[0];
  return {
    // The channel code (resource code) is the upsert path identity; the item
    // id is a database row id that must never be sent as the channel code.
    channelId: channel.code ?? channel.id,
    kind: channel.kind,
    name: channel.name,
    description: channel.description ?? '',
    baseUrl: channel.baseUrl ?? '',
    apiKey: '',
    // Credentials are managed outside this API (Config SPI), and the API
    // never returns whether a key exists, so an existing channel is treated
    // as configured and its key field is optional on edit.
    apiKeyConfigured: channel.apiKeyConfigured !== false,
    offerings: channel.offerings.map((offering) => {
      const models = offering.models.map((model) => ({
        modelId: model.model,
        displayName: model.displayName || model.model,
        ...(model.contextTokens == null ? {} : { contextTokens: model.contextTokens }),
        ...(model.maxOutputTokens == null ? {} : { maxOutputTokens: model.maxOutputTokens }),
        ...(model.toolCallRounds == null ? {} : { toolCallRounds: model.toolCallRounds }),
      }));
      return {
        vendorCode: offering.vendorCode,
        vendorName: offering.vendorName,
        models,
        modelIds: models.map((model) => model.modelId),
      };
    }),
    defaultVendorCode: defaultOffering?.vendorCode ?? '',
    defaultModelId: defaultModel?.model ?? '',
    supportedAgentProviderIds: [...channel.supportedAgentProviderIds],
  };
}

function normalizeOffering(
  offering: ModelOfferingConfigurationDraft,
): ModelOfferingConfigurationDraft {
  const vendorCode = offering.vendorCode.trim();
  const models = normalizeConfigurationModels(offering.models ?? [], offering.modelIds ?? []);
  return {
    vendorCode,
    vendorName: offering.vendorName.trim() || vendorCode,
    models,
    modelIds: models.map((model) => model.modelId),
  };
}

export function normalizeModelAccessChannelConfigurationDraft(
  draft: ModelAccessChannelConfigurationDraft,
): ModelAccessChannelConfigurationDraft {
  const name = draft.name.trim();
  return {
    ...draft,
    channelId: draft.channelId.trim() || `model-access.${draft.kind}.${slug(name)}`,
    name,
    description: draft.description.trim(),
    baseUrl: draft.baseUrl.trim().replace(/\/+$/u, ''),
    apiKey: draft.apiKey.trim(),
    offerings: draft.offerings.map(normalizeOffering),
    defaultVendorCode: draft.defaultVendorCode.trim(),
    defaultModelId: draft.defaultModelId.trim(),
    supportedAgentProviderIds: normalizeList(draft.supportedAgentProviderIds),
  };
}

/**
 * Validates a channel configuration draft.
 *
 * `activeProviderId` names the provider the configuration is being prepared
 * for (the chat composer passes the active engine id, so the active engine
 * must stay checked). An empty value means "any checked subset is fine" —
 * the settings panel has no active engine and must allow saving a
 * configuration for a single secondary provider.
 */
export function validateModelAccessChannelConfigurationDraft(
  draft: ModelAccessChannelConfigurationDraft,
  activeProviderId: string,
  officialVendorCodes: readonly string[] = [],
): ModelAccessChannelConfigurationValidation {
  const normalized = normalizeModelAccessChannelConfigurationDraft(draft);
  const populatedOfferings = normalized.offerings.filter((offering) => (
    offering.vendorCode && offering.modelIds.length > 0
  ));
  const vendorCodes = normalized.offerings
    .map((offering) => offering.vendorCode.toLowerCase())
    .filter(Boolean);
  const defaultVendor = normalized.defaultVendorCode.toLowerCase();
  const defaultModel = normalized.defaultModelId.toLowerCase();
  const normalizedOfficialVendorCodes = new Set(
    officialVendorCodes.map((vendorCode) => vendorCode.trim().toLowerCase()),
  );
  const defaultExists = populatedOfferings.some((offering) => (
    offering.vendorCode.toLowerCase() === defaultVendor
    && offering.modelIds.some((modelId) => modelId.toLowerCase() === defaultModel)
  ));
  return {
    apiKeyRequired: !normalized.apiKeyConfigured && !normalized.apiKey,
    baseUrlInvalid: !isAbsoluteHttpUrl(normalized.baseUrl),
    channelNameRequired: !normalized.name,
    defaultModelRequired: !defaultVendor || !defaultModel || !defaultExists,
    duplicateVendor: new Set(vendorCodes).size !== vendorCodes.length,
    offeringModelsRequired: normalized.offerings.some((offering) => (
      Boolean(offering.vendorCode) && offering.modelIds.length === 0
    )),
    officialVendorUnsupported:
      normalized.kind === 'official'
      && normalizedOfficialVendorCodes.size > 0
      && !normalized.offerings.some((offering) => (
        normalizedOfficialVendorCodes.has(offering.vendorCode.toLowerCase())
      )),
    officialVendorCountInvalid:
      normalized.kind === 'official' && normalized.offerings.length !== 1,
    offeringsRequired: populatedOfferings.length === 0,
    providerRequired:
      normalized.supportedAgentProviderIds.length === 0
      || (activeProviderId
        ? !normalized.supportedAgentProviderIds.includes(activeProviderId)
        : false),
    vendorRequired: normalized.offerings.some((offering) => !offering.vendorCode),
  };
}

export function isModelAccessChannelConfigurationDraftValid(
  validation: ModelAccessChannelConfigurationValidation,
): boolean {
  return !Object.values(validation).some(Boolean);
}
