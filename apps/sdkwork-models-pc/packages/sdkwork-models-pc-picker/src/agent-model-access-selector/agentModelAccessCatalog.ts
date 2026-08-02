import type {
  AgentModelAccessSelection,
  AgentModelCatalogOption,
  ModelAccessChannel,
  ModelCatalogSortOrder,
  ModelOffering,
  ModelOfferingModel,
  OfficialModelVendorPreset,
} from './agentModelAccessSelectorTypes';
import { SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS } from './officialModelVendorPresets.generated.ts';

function compareText(left: string, right: string): number {
  const normalizedLeft = left.trim().toLowerCase();
  const normalizedRight = right.trim().toLowerCase();
  return normalizedLeft < normalizedRight ? -1 : normalizedLeft > normalizedRight ? 1 : 0;
}

function numericOrder(value?: ModelCatalogSortOrder): number {
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : Number.MAX_SAFE_INTEGER;
  }
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : Number.MAX_SAFE_INTEGER;
  }
  return Number.MAX_SAFE_INTEGER;
}

function compareOptionalOrder(
  left?: ModelCatalogSortOrder,
  right?: ModelCatalogSortOrder,
): number {
  return numericOrder(left) - numericOrder(right);
}

function releaseStageOrder(stage?: string): number {
  return stage === 'active' ? 0 : stage === 'preview' ? 1 : 2;
}

function observedAtScore(value?: string): number {
  if (!value) {
    return 0;
  }
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function normalizeSearchTerms(query: string): string[] {
  return query
    .trim()
    .toLowerCase()
    .split(/\s+/u)
    .filter(Boolean);
}

function matchesTerms(
  haystackParts: readonly (string | null | undefined)[],
  query: string,
): boolean {
  const terms = normalizeSearchTerms(query);
  if (terms.length === 0) {
    return true;
  }
  const haystack = haystackParts.filter(Boolean).join(' ').toLowerCase();
  return terms.every((term) => haystack.includes(term));
}

function sameOfferingModel(
  offering: ModelOffering,
  offeredModel: ModelOfferingModel,
  model: AgentModelCatalogOption,
): boolean {
  if (offeredModel.modelOptionId && offeredModel.modelOptionId === model.id) {
    return true;
  }
  if (offeredModel.catalogKey && offeredModel.catalogKey === model.catalogKey) {
    return true;
  }
  return offering.vendorCode.trim().toLowerCase() === model.vendorCode.trim().toLowerCase()
    && offeredModel.model.trim().toLowerCase() === model.modelId.trim().toLowerCase();
}

function findChannelOffering(
  channel: ModelAccessChannel,
  model: AgentModelCatalogOption,
): { offering: ModelOffering; offeredModel: ModelOfferingModel } | undefined {
  for (const offering of channel.offerings) {
    const offeredModel = offering.models.find((item) => sameOfferingModel(
      offering,
      item,
      model,
    ));
    if (offeredModel) {
      return { offering, offeredModel };
    }
  }
  return undefined;
}

export function compareAgentModelCatalogOptions(
  left: AgentModelCatalogOption,
  right: AgentModelCatalogOption,
): number {
  return (left.kind === 'custom' ? 1 : 0) - (right.kind === 'custom' ? 1 : 0)
    || (right.rankScore ?? 0) - (left.rankScore ?? 0)
    || compareOptionalOrder(left.sortOrder, right.sortOrder)
    || releaseStageOrder(left.releaseStage) - releaseStageOrder(right.releaseStage)
    || observedAtScore(right.sourceObservedAt) - observedAtScore(left.sourceObservedAt)
    || compareText(left.vendorName, right.vendorName)
    || compareText(left.label, right.label)
    || compareText(left.modelId, right.modelId)
    || compareText(left.id, right.id);
}

export function sortAgentModelCatalogOptions(
  models: readonly AgentModelCatalogOption[],
): AgentModelCatalogOption[] {
  return [...models].sort(compareAgentModelCatalogOptions);
}

function agentModelIdentity(
  model: Pick<AgentModelCatalogOption, 'modelId' | 'vendorCode'>,
): string {
  return `${model.vendorCode.trim().toLowerCase()}\u0000${model.modelId.trim().toLowerCase()}`;
}

/**
 * Database rows arrive without the curated mainstream ordering signals when
 * rank scores are missing, so their list falls back to the storage order
 * (often display name). Inherit the curated rank, order, and observed-at from
 * the mainstream fallback catalog by identity so vendor model lists always
 * place the newest models first. Rows that already carry a rank are untouched,
 * and rows without a mainstream match keep their database ordering.
 */
function enrichAgentModelOrderingFromMainstream(
  databaseModels: readonly AgentModelCatalogOption[],
  fallbackModels: readonly AgentModelCatalogOption[],
): AgentModelCatalogOption[] {
  const byCatalogKey = new Map<string, AgentModelCatalogOption>();
  const byIdentity = new Map<string, AgentModelCatalogOption>();
  for (const model of fallbackModels) {
    const catalogKey = model.catalogKey?.trim().toLowerCase();
    if (catalogKey) {
      byCatalogKey.set(catalogKey, model);
    }
    byIdentity.set(agentModelIdentity(model), model);
  }
  return databaseModels.map((model) => {
    if (typeof model.rankScore === 'number') {
      return model;
    }
    const catalogKey = model.catalogKey?.trim().toLowerCase();
    const curated = (catalogKey ? byCatalogKey.get(catalogKey) : undefined)
      ?? byIdentity.get(agentModelIdentity(model));
    if (!curated) {
      return model;
    }
    return {
      ...model,
      rankScore: curated.rankScore,
      sortOrder: curated.sortOrder,
      sourceObservedAt: curated.sourceObservedAt ?? model.sourceObservedAt,
    };
  });
}

/** A non-empty database page is authoritative and is never merged with fallback data. */
export function resolveAuthoritativeAgentModelCatalog(
  databaseModels: readonly AgentModelCatalogOption[],
  fallbackModels: readonly AgentModelCatalogOption[],
): AgentModelCatalogOption[] {
  return sortAgentModelCatalogOptions(
    databaseModels.length > 0
      ? enrichAgentModelOrderingFromMainstream(databaseModels, fallbackModels)
      : fallbackModels,
  );
}

export function compareModelAccessChannels(
  left: ModelAccessChannel,
  right: ModelAccessChannel,
): number {
  return compareOptionalOrder(left.sortOrder, right.sortOrder)
    || (left.kind === 'official' ? 0 : 1) - (right.kind === 'official' ? 0 : 1)
    || compareText(left.name, right.name)
    || compareText(left.id, right.id);
}

export function sortModelAccessChannels(
  channels: readonly ModelAccessChannel[],
): ModelAccessChannel[] {
  return [...channels].sort(compareModelAccessChannels);
}

export function sortModelOfferings(
  offerings: readonly ModelOffering[],
): ModelOffering[] {
  return [...offerings]
    .sort((left, right) => (
      compareText(left.vendorName, right.vendorName)
      || compareText(left.vendorCode, right.vendorCode)
    ))
    .map((offering) => ({
      ...offering,
      models: [...offering.models].sort((left, right) => (
        compareOptionalOrder(left.sortOrder, right.sortOrder)
        || compareText(left.displayName, right.displayName)
        || compareText(left.model, right.model)
      )),
    }));
}

export function findModelAccessChannels(
  model: AgentModelCatalogOption,
  channels: readonly ModelAccessChannel[],
): ModelAccessChannel[] {
  return sortModelAccessChannels(channels.filter((channel) => (
    Boolean(findChannelOffering(channel, model))
  )));
}

export function agentModelMatchesQuery(
  model: AgentModelCatalogOption,
  query: string,
  channels: readonly ModelAccessChannel[] = [],
): boolean {
  const relatedChannels = findModelAccessChannels(model, channels);
  return matchesTerms([
    model.label,
    model.modelId,
    model.description,
    model.vendorCode,
    model.vendorName,
    model.metadataLabel,
    model.releaseStage,
    ...(model.searchTerms ?? []),
    ...relatedChannels.flatMap((channel) => [
      channel.name,
      channel.code,
      channel.kind,
      channel.baseUrl,
      channel.description,
      ...(channel.searchTerms ?? []),
    ]),
  ], query);
}

export function modelAccessChannelMatchesQuery(
  channel: ModelAccessChannel,
  query: string,
): boolean {
  return matchesTerms([
    channel.name,
    channel.code,
    channel.kind,
    channel.baseUrl,
    channel.description,
    ...channel.supportedAgentProviderIds,
    ...(channel.searchTerms ?? []),
    ...channel.offerings.flatMap((offering) => [
      offering.vendorCode,
      offering.vendorName,
      ...offering.models.flatMap((model) => [
        model.catalogKey,
        model.model,
        model.displayName,
      ]),
    ]),
  ], query);
}

export function filterAgentModelCatalogOptions(
  models: readonly AgentModelCatalogOption[],
  query: string,
  channels: readonly ModelAccessChannel[] = [],
): AgentModelCatalogOption[] {
  return models.filter((model) => agentModelMatchesQuery(model, query, channels));
}

export function filterModelAccessChannels(
  channels: readonly ModelAccessChannel[],
  query: string,
): ModelAccessChannel[] {
  return channels.filter((channel) => modelAccessChannelMatchesQuery(channel, query));
}

export function createFallbackOfficialAccessChannels(
  models: readonly AgentModelCatalogOption[],
  presets: readonly OfficialModelVendorPreset[] = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS,
): ModelAccessChannel[] {
  const groups = new Map<string, {
    vendorCode: string;
    vendorName: string;
    models: AgentModelCatalogOption[];
  }>();
  for (const model of models) {
    const key = model.vendorCode.trim().toLowerCase();
    const current = groups.get(key);
    if (current) {
      current.models.push(model);
    } else {
      groups.set(key, {
        vendorCode: model.vendorCode,
        vendorName: model.vendorName,
        models: [model],
      });
    }
  }
  return [...presets]
    .sort((left, right) => left.sortOrder - right.sortOrder)
    .flatMap((preset) => {
      const group = groups.get(preset.vendorCode.trim().toLowerCase());
      if (!group && preset.models === undefined) {
        return [];
      }
      const sortedModels = preset.models === undefined
        ? sortAgentModelCatalogOptions(group?.models ?? [])
        : preset.models.flatMap((presetModel, modelIndex) => {
          const modelId = presetModel.model.trim();
          if (!modelId) {
            return [];
          }
          const matched = group?.models.find((model) => (
            (presetModel.catalogKey && model.catalogKey === presetModel.catalogKey)
            || model.modelId.trim().toLowerCase() === modelId.toLowerCase()
          ));
          return [{
            ...(matched ?? {
              id: presetModel.catalogKey || `${preset.vendorCode}/${modelId}`,
              catalogKey: presetModel.catalogKey,
              modelId,
              label: presetModel.displayName || modelId,
              source: 'database' as const,
              vendorCode: preset.vendorCode,
              vendorName: preset.vendorName,
            }),
            label: presetModel.displayName || matched?.label || modelId,
            sortOrder: presetModel.sortOrder ?? modelIndex,
          }];
        });
      const defaultModelId = preset.defaultModelId && sortedModels.some((model) => (
        model.modelId.trim().toLowerCase() === preset.defaultModelId?.trim().toLowerCase()
      ))
        ? preset.defaultModelId
        : sortedModels[0]?.modelId;
      return [{
        id: `official.${preset.vendorCode.trim().toLowerCase()}`,
        code: `official.${preset.vendorCode.trim().toLowerCase()}`,
        name: preset.channelName,
        kind: 'official',
        isCustom: false,
        source: 'fallback',
        modelCount: sortedModels.length,
        baseUrl: preset.baseUrl,
        defaultVendorCode: preset.vendorCode,
        defaultModelId,
        offerings: [{
          vendorCode: preset.vendorCode,
          vendorName: preset.vendorName,
          models: sortedModels.map((model, modelIndex) => ({
            catalogKey: model.catalogKey,
            model: model.modelId,
            displayName: model.label,
            modelOptionId: model.id,
            sortOrder: model.sortOrder ?? modelIndex,
          })),
        }],
        searchTerms: [preset.providerCode, preset.providerDisplayName, preset.protocol],
        sortOrder: preset.sortOrder,
        supportedAgentProviderIds: [...new Set(sortedModels.flatMap(
          (model) => model.supportedAgentProviderIds ?? [],
        ))],
        vendorCount: 1,
      } satisfies ModelAccessChannel];
    }).sort(compareModelAccessChannels);
}

export function resolveModelAccessChannels(
  databaseChannels: readonly ModelAccessChannel[],
  models: readonly AgentModelCatalogOption[],
  officialVendorPresets: readonly OfficialModelVendorPreset[] = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS,
): ModelAccessChannel[] {
  return databaseChannels.length > 0
    ? sortModelAccessChannels(databaseChannels)
    : createFallbackOfficialAccessChannels(models, officialVendorPresets);
}

export function resolveOfferingModel(
  offering: ModelOffering,
  offeredModel: ModelOfferingModel,
  models: readonly AgentModelCatalogOption[],
): AgentModelCatalogOption {
  const matched = models.find((model) => sameOfferingModel(offering, offeredModel, model));
  return matched ?? {
    id: offeredModel.modelOptionId
      ?? offeredModel.catalogKey
      ?? `${offering.vendorCode}/${offeredModel.model}`,
    catalogKey: offeredModel.catalogKey,
    label: offeredModel.displayName || offeredModel.model,
    modelId: offeredModel.model,
    source: 'database',
    vendorCode: offering.vendorCode,
    vendorName: offering.vendorName,
  };
}

export function createAgentModelAccessSelection(
  model: AgentModelCatalogOption,
  channels: readonly ModelAccessChannel[],
  preferredChannelId?: string,
): AgentModelAccessSelection | undefined {
  const matchingChannels = findModelAccessChannels(model, channels);
  const channel = matchingChannels.find((item) => item.id === preferredChannelId)
    ?? matchingChannels.find((item) => item.kind === 'official' && !item.disabled)
    ?? matchingChannels.find((item) => !item.disabled);
  if (!channel) {
    return undefined;
  }
  const match = findChannelOffering(channel, model);
  return match ? { channel, model, ...match } : undefined;
}

export function modelAccessChannelNeedsConfiguration(
  channel: ModelAccessChannel,
): boolean {
  return channel.apiKeyConfigured !== true;
}

export function createModelAccessChannelConfigurationTarget(
  selection: AgentModelAccessSelection,
): ModelAccessChannel {
  return {
    ...selection.channel,
    defaultVendorCode: selection.offering.vendorCode,
    defaultModelId: selection.offeredModel.model,
  };
}
