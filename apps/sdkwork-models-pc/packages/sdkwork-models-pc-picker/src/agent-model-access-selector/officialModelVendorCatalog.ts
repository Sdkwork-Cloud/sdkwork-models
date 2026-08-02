import { sortAgentModelCatalogOptions } from './agentModelAccessCatalog.ts';
import type {
  AgentModelCatalogOption,
  ModelAccessChannel,
  ModelAccessChannelConfigurationDraft,
  OfficialModelVendorPreset,
} from './agentModelAccessSelectorTypes';
import { createModelOfferingConfigurationDraft } from './modelAccessChannelConfigurationValidation.ts';
import { SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS } from './officialModelVendorPresets.generated.ts';

export type OfficialModelVendorCatalogEntry = Omit<
  OfficialModelVendorPreset,
  'models' | 'defaultModelId'
> & {
  models: readonly AgentModelCatalogOption[];
  defaultModelId?: string;
};

function sameCode(left: string, right: string): boolean {
  return left.trim().toLowerCase() === right.trim().toLowerCase();
}

function comparePresetOrder(
  left: OfficialModelVendorPreset,
  right: OfficialModelVendorPreset,
): number {
  return left.sortOrder - right.sortOrder
    || left.vendorName.localeCompare(right.vendorName)
    || left.vendorCode.localeCompare(right.vendorCode);
}

function resolvePresetModels(
  catalogModels: readonly AgentModelCatalogOption[],
  preset: OfficialModelVendorPreset,
): AgentModelCatalogOption[] {
  if (preset.models === undefined) {
    return sortAgentModelCatalogOptions(catalogModels.filter((model) => (
      sameCode(model.vendorCode, preset.vendorCode)
    )));
  }

  const seen = new Set<string>();
  return preset.models.flatMap((presetModel, index) => {
    const modelId = presetModel.model.trim();
    const identity = modelId.toLowerCase();
    if (!modelId || seen.has(identity)) {
      return [];
    }
    seen.add(identity);
    const catalogKey = presetModel.catalogKey?.trim();
    const matched = catalogModels.find((model) => (
      sameCode(model.vendorCode, preset.vendorCode)
      && (
        (catalogKey && model.catalogKey === catalogKey)
        || sameCode(model.modelId, modelId)
      )
    ));
    const displayName = presetModel.displayName.trim() || matched?.label || modelId;
    return [{
      ...(matched ?? {
        id: catalogKey || `${preset.vendorCode}/${modelId}`,
        catalogKey,
        modelId,
        source: 'database' as const,
        vendorCode: preset.vendorCode,
        vendorName: preset.vendorName,
      }),
      label: displayName,
      sortOrder: presetModel.sortOrder ?? index,
    }];
  });
}

/** A non-empty runtime preset result is authoritative; empty uses generated fallback. */
export function resolveOfficialModelVendorPresets(
  runtimePresets?: readonly OfficialModelVendorPreset[],
  fallbackPresets: readonly OfficialModelVendorPreset[] = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS,
): OfficialModelVendorPreset[] {
  const source = runtimePresets && runtimePresets.length > 0
    ? runtimePresets
    : fallbackPresets;
  const seenVendors = new Set<string>();
  return [...source]
    .sort(comparePresetOrder)
    .filter((preset) => {
      const identity = preset.vendorCode.trim().toLowerCase();
      if (!identity || seenVendors.has(identity)) {
        return false;
      }
      seenVendors.add(identity);
      return true;
    });
}

export function resolveOfficialModelVendorCatalog(
  models: readonly AgentModelCatalogOption[],
  presets: readonly OfficialModelVendorPreset[] = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS,
): OfficialModelVendorCatalogEntry[] {
  return resolveOfficialModelVendorPresets(presets)
    .map((preset) => {
      const vendorModels = resolvePresetModels(models, preset);
      const configuredDefaultModelId = preset.defaultModelId?.trim();
      const defaultModelId = configuredDefaultModelId && vendorModels.some((model) => (
        sameCode(model.modelId, configuredDefaultModelId)
      ))
        ? configuredDefaultModelId
        : vendorModels[0]?.modelId;
      return {
        ...preset,
        models: vendorModels,
        defaultModelId,
      };
    });
}

export function findOfficialModelVendorPreset(
  vendorCode: string,
  presets: readonly OfficialModelVendorPreset[] = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS,
): OfficialModelVendorPreset | undefined {
  return resolveOfficialModelVendorPresets(presets).find((preset) => (
    sameCode(preset.vendorCode, vendorCode)
  ));
}

export function applyOfficialModelVendorCatalogEntry(
  draft: ModelAccessChannelConfigurationDraft,
  entry: OfficialModelVendorCatalogEntry,
): ModelAccessChannelConfigurationDraft {
  const offering = createModelOfferingConfigurationDraft(
    entry.vendorCode,
    entry.vendorName,
    entry.models,
  );
  const currentDefaultIsAvailable = sameCode(
    draft.defaultVendorCode,
    entry.vendorCode,
  ) && offering.modelIds.some((modelId) => sameCode(modelId, draft.defaultModelId));
  const channelId = draft.channelId.trim();
  return {
    ...draft,
    channelId: !channelId || /^official\.[a-z0-9._-]+$/u.test(channelId)
      ? `official.${entry.vendorCode}`
      : channelId,
    kind: 'official',
    name: entry.channelName,
    baseUrl: entry.baseUrl,
    offerings: [offering],
    defaultVendorCode: entry.vendorCode,
    defaultModelId: currentDefaultIsAvailable
      ? draft.defaultModelId
      : entry.defaultModelId ?? '',
  };
}

export function isConfiguredOfficialModelAccessChannel(
  channel: ModelAccessChannel,
  presets: readonly OfficialModelVendorPreset[] = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS,
): boolean {
  if (channel.kind !== 'official') {
    return true;
  }
  return channel.offerings.length === 1
    && Boolean(findOfficialModelVendorPreset(channel.offerings[0]?.vendorCode ?? '', presets));
}
