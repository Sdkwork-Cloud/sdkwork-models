import type {
  BillingMeter,
  ClientApiCompatibility,
  ModelCatalog,
  ModelInfo,
  ModelPrice,
  ModelVendorIdentity,
  ProtocolStandard,
  TtsVoice,
  VideoGenerationProfile,
  VendorRegionRef,
} from "./types.js";

export interface ModelFilter {
  vendorCode?: string;
  regionCode?: string;
  familyCode?: string;
  capability?: string;
  inputModality?: string;
  outputModality?: string;
  releaseStage?: string;
  shelfState?: string;
  routingState?: string;
  apiFormat?: string;
}

export function listVendors(catalog: ModelCatalog): ModelVendorIdentity[] {
  const vendors = new Map<string, ModelVendorIdentity>();
  for (const regionCatalog of catalog.vendors) {
    const vendor = regionCatalog.vendor;
    if (vendors.has(vendor.vendorCode)) {
      continue;
    }
    vendors.set(vendor.vendorCode, {
      vendorCode: vendor.vendorCode,
      displayName: vendor.displayName,
      legalName: vendor.legalName,
      vendorType: vendor.vendorType,
      capabilities: [...vendor.capabilities],
      supportedProtocols: [...(vendor.supportedProtocols ?? [])],
      clientApiCompatibility: { ...(vendor.clientApiCompatibility ?? {}) },
      openSource: vendor.openSource,
    });
  }
  return [...vendors.values()];
}

export function listVendorRegions(catalog: ModelCatalog): VendorRegionRef[] {
  return catalog.vendors.map((regionCatalog) => ({
    vendorCode: regionCatalog.vendorCode,
    regionCode: regionCatalog.regionCode,
  }));
}

export function catalogKey(vendorCode: string, modelId: string): string {
  return `${vendorCode}/${modelId}`;
}

export function listMeters(catalog: ModelCatalog): BillingMeter[] {
  return catalog.meters;
}

export function findMeter(catalog: ModelCatalog, meterCode: string): BillingMeter | undefined {
  return catalog.meters.find((meter) => meter.meterCode === meterCode);
}

export function listModels(catalog: ModelCatalog, filter: ModelFilter = {}): ModelInfo[] {
  const matches = catalog.vendors
    .flatMap((vendor) => vendor.models.map((model) => ({
      model,
      hasRegionPricing: vendor.pricing.some((pricing) => pricing.modelId === model.modelId && pricing.prices.length > 0),
    })))
    .filter(({ model }) => !filter.vendorCode || model.vendorCode === filter.vendorCode)
    .filter(({ model }) => !filter.regionCode || model.regionCode === filter.regionCode)
    .filter(({ model }) => !filter.familyCode || model.familyCode === filter.familyCode)
    .filter(({ model }) => !filter.capability || model.capabilities.includes(filter.capability as never))
    .filter(({ model }) => !filter.inputModality || model.inputModalities.includes(filter.inputModality as never))
    .filter(({ model }) => !filter.outputModality || model.outputModalities.includes(filter.outputModality as never))
    .filter(({ model }) => !filter.releaseStage || model.releaseStage === filter.releaseStage)
    .filter(({ model }) => !filter.shelfState || model.shelfState === filter.shelfState)
    .filter(({ model }) => !filter.routingState || model.routingState === filter.routingState)
    .filter(({ model }) => !filter.apiFormat || model.apiFormat === filter.apiFormat);
  if (filter.regionCode) {
    return matches.map(({ model }) => model);
  }
  const deduped = new Map<string, { model: ModelInfo; hasRegionPricing: boolean }>();
  for (const item of matches) {
    const existing = deduped.get(item.model.catalogKey);
    if (!existing || modelIdentityScore(item) > modelIdentityScore(existing)) {
      deduped.set(item.model.catalogKey, item);
    }
  }
  return [...deduped.values()].map(({ model }) => model);
}

export function listAvailableModels(catalog: ModelCatalog, filter: ModelFilter = {}): ModelInfo[] {
  return listModels(catalog, { ...filter, routingState: "enabled", shelfState: "listed" })
    .filter((model) => getModelRegionPrices(catalog, model.catalogKey, model.regionCode).length > 0);
}

export function findModel(catalog: ModelCatalog, catalogKeyValue: string): ModelInfo | undefined {
  const [vendorCode, modelId] = splitCatalogKey(catalogKeyValue);
  if (!vendorCode || !modelId) {
    return undefined;
  }
  return listModels(catalog)
    .find((model) => model.vendorCode === vendorCode && model.modelId === modelId);
}

export function findModelByVendorRegion(
  catalog: ModelCatalog,
  vendorCode: string,
  regionCode: string,
  modelId: string,
): ModelInfo | undefined {
  return listModels(catalog, { vendorCode, regionCode }).find((model) => model.modelId === modelId);
}

export function getModelPrices(catalog: ModelCatalog, catalogKeyValue: string): ModelPrice[] {
  const [vendorCode, modelId] = splitCatalogKey(catalogKeyValue);
  if (!vendorCode || !modelId) {
    return [];
  }
  return catalog.vendors
    .filter((vendor) => vendor.vendorCode === vendorCode)
    .flatMap((vendor) => vendor.pricing)
    .find((pricing) => pricing.modelId === modelId)
    ?.prices ?? [];
}

export function getModelRegionPrices(catalog: ModelCatalog, catalogKeyValue: string, regionCode: string): ModelPrice[] {
  const [vendorCode, modelId] = splitCatalogKey(catalogKeyValue);
  if (!vendorCode || !modelId) {
    return [];
  }
  return catalog.vendors
    .filter((vendor) => vendor.vendorCode === vendorCode && vendor.regionCode === regionCode)
    .flatMap((vendor) => vendor.pricing)
    .find((pricing) => pricing.modelId === modelId)
    ?.prices ?? [];
}

export function getBestReferencePrice(
  catalog: ModelCatalog,
  catalogKeyValue: string,
  meterCode: string,
): ModelPrice | undefined {
  return getModelPrices(catalog, catalogKeyValue).find((price) => price.meterCode === meterCode);
}

export function listModelsByCapability(catalog: ModelCatalog, capability: string): ModelInfo[] {
  return listModels(catalog, { capability });
}

export function listModelsByModality(
  catalog: ModelCatalog,
  inputModality: string,
  outputModality: string,
): ModelInfo[] {
  return listModels(catalog, { inputModality, outputModality });
}

export function listProtocols(catalog: ModelCatalog): ProtocolStandard[] {
  return catalog.protocols;
}

export function findProtocol(catalog: ModelCatalog, protocolCode: string): ProtocolStandard | undefined {
  return catalog.protocols.find((p) => p.protocolCode === protocolCode);
}

export function listProtocolsByVendor(catalog: ModelCatalog, vendorCode: string): ProtocolStandard[] {
  const vendorIdentity = catalog.vendors
    .map((vc) => vc.vendor)
    .find((vendor) => vendor.vendorCode === vendorCode);
  if (!vendorIdentity) {
    return [];
  }
  const supported = new Set(vendorIdentity.supportedProtocols ?? []);
  return catalog.protocols.filter((p) => supported.has(p.protocolCode));
}

export function listClientApiCompatibilityByVendor(catalog: ModelCatalog, vendorCode: string): ClientApiCompatibility[] {
  const vendorIdentity = catalog.vendors
    .map((vc) => vc.vendor)
    .find((vendor) => vendor.vendorCode === vendorCode);
  if (!vendorIdentity) {
    return [];
  }
  return Object.values(vendorIdentity.clientApiCompatibility ?? {});
}

export function listModelsByProtocol(catalog: ModelCatalog, protocolCode: string): ModelInfo[] {
  return listModels(catalog, { apiFormat: protocolCode });
}

export interface VoiceFilter {
  vendorCode?: string;
  regionCode?: string;
  locale?: string;
  modelCatalogKey?: string;
  q?: string;
}

export function voiceCatalogKey(vendorCode: string, voiceId: string): string {
  return `${vendorCode}/${voiceId}`;
}

export function listVoices(catalog: ModelCatalog, filter: VoiceFilter = {}): TtsVoice[] {
  return catalog.vendors
    .flatMap((vendor) => vendor.voices ?? [])
    .filter((voice) => !filter.vendorCode || voice.vendorCode === filter.vendorCode)
    .filter((voice) => !filter.regionCode || voice.regionCode === filter.regionCode)
    .filter((voice) => {
      if (!filter.locale) return true;
      return voice.primaryLocale === filter.locale || (voice.supportedLocales ?? []).includes(filter.locale);
    })
    .filter((voice) => {
      if (!filter.q) return true;
      const query = filter.q.toLowerCase();
      return voice.displayName.toLowerCase().includes(query) || voice.voiceId.toLowerCase().includes(query);
    })
    .filter((voice) => {
      if (!filter.modelCatalogKey) return true;
      return catalog.vendors.some((vendor) =>
        (vendor.modelVoiceBindings ?? []).some(
          (binding) =>
            binding.catalogKey === filter.modelCatalogKey &&
            binding.bindings.some((entry) => entry.voiceKey === voice.catalogKey),
        ),
      );
    });
}

export function listVoicesForModel(catalog: ModelCatalog, modelCatalogKey: string): TtsVoice[] {
  return listVoices(catalog, { modelCatalogKey });
}

export function listModelsForVoice(catalog: ModelCatalog, voiceCatalogKeyValue: string): ModelInfo[] {
  const modelKeys = new Set(
    catalog.vendors.flatMap((vendor) =>
      (vendor.modelVoiceBindings ?? [])
        .filter((binding) => binding.bindings.some((entry) => entry.voiceKey === voiceCatalogKeyValue))
        .map((binding) => binding.catalogKey),
    ),
  );
  return listModels(catalog).filter((model) => modelKeys.has(model.catalogKey));
}

export interface VideoProfileFilter {
  vendorCode?: string;
  regionCode?: string;
  modelCatalogKey?: string;
  generationMode?: VideoGenerationProfile["generationMode"];
  durationTierCode?: string;
  resolution?: string;
}

export function videoProfileCatalogKey(vendorCode: string, modelId: string, profileCode: string): string {
  return `${vendorCode}/${modelId}/${profileCode}`;
}

export function listVideoProfiles(catalog: ModelCatalog, filter: VideoProfileFilter = {}): VideoGenerationProfile[] {
  const profiles: VideoGenerationProfile[] = [];
  for (const vendor of catalog.vendors) {
    if (filter.vendorCode && vendor.vendorCode !== filter.vendorCode) continue;
    if (filter.regionCode && vendor.regionCode !== filter.regionCode) continue;
    for (const file of vendor.modelVideoProfiles ?? []) {
      if (filter.modelCatalogKey && file.catalogKey !== filter.modelCatalogKey) continue;
      for (const profile of file.profiles ?? []) {
        if (filter.generationMode && profile.generationMode !== filter.generationMode) continue;
        if (
          filter.durationTierCode
          && profile.durationTierCode !== filter.durationTierCode
          && !(profile.durationTierCodes ?? []).includes(filter.durationTierCode)
        ) {
          continue;
        }
        if (filter.resolution && profile.resolution !== filter.resolution) continue;
        profiles.push(profile);
      }
    }
  }
  return profiles;
}

export function listVideoProfilesForModel(catalog: ModelCatalog, modelCatalogKey: string): VideoGenerationProfile[] {
  return listVideoProfiles(catalog, { modelCatalogKey });
}

export function findVideoProfile(catalog: ModelCatalog, profileCatalogKey: string): VideoGenerationProfile | undefined {
  for (const vendor of catalog.vendors) {
    for (const file of vendor.modelVideoProfiles ?? []) {
      const profile = (file.profiles ?? []).find((entry) => entry.catalogKey === profileCatalogKey);
      if (profile) return profile;
    }
  }
  return undefined;
}

function splitCatalogKey(value: string): [string | undefined, string | undefined] {
  const separatorIndex = value.indexOf("/");
  if (separatorIndex <= 0 || separatorIndex === value.length - 1) {
    return [undefined, undefined];
  }
  return [value.slice(0, separatorIndex), value.slice(separatorIndex + 1)];
}

function modelIdentityScore(item: { model: ModelInfo; hasRegionPricing: boolean }): number {
  let score = 0;
  if (item.hasRegionPricing) score += 100;
  if (item.model.routingState === "enabled") score += 40;
  if (item.model.shelfState === "listed") score += 20;
  if (item.model.releaseStage === "active") score += 10;
  if (item.model.lifecycle === "current" || item.model.lifecycle === "preview") score += 5;
  if (item.model.regionCode === "global") score += 1;
  return score;
}
