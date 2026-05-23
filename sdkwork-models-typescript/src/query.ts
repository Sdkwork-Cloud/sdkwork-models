import type {
  BillingMeter,
  ModelCatalog,
  ModelInfo,
  ModelPrice,
  ModelVendorIdentity,
  ProtocolStandard,
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

export function catalogKey(vendorCode: string, regionCode: string, modelId: string): string {
  return `${vendorCode}/${regionCode}/${modelId}`;
}

export function listMeters(catalog: ModelCatalog): BillingMeter[] {
  return catalog.meters;
}

export function findMeter(catalog: ModelCatalog, meterCode: string): BillingMeter | undefined {
  return catalog.meters.find((meter) => meter.meterCode === meterCode);
}

export function listModels(catalog: ModelCatalog, filter: ModelFilter = {}): ModelInfo[] {
  return catalog.vendors
    .flatMap((vendor) => vendor.models)
    .filter((model) => !filter.vendorCode || model.vendorCode === filter.vendorCode)
    .filter((model) => !filter.regionCode || model.regionCode === filter.regionCode)
    .filter((model) => !filter.familyCode || model.familyCode === filter.familyCode)
    .filter((model) => !filter.capability || model.capabilities.includes(filter.capability as never))
    .filter((model) => !filter.inputModality || model.inputModalities.includes(filter.inputModality as never))
    .filter((model) => !filter.outputModality || model.outputModalities.includes(filter.outputModality as never))
    .filter((model) => !filter.releaseStage || model.releaseStage === filter.releaseStage)
    .filter((model) => !filter.shelfState || model.shelfState === filter.shelfState)
    .filter((model) => !filter.routingState || model.routingState === filter.routingState)
    .filter((model) => !filter.apiFormat || model.apiFormat === filter.apiFormat);
}

export function listAvailableModels(catalog: ModelCatalog, filter: ModelFilter = {}): ModelInfo[] {
  return listModels(catalog, { ...filter, routingState: "enabled", shelfState: "listed" })
    .filter((model) => getModelPrices(catalog, model.catalogKey).length > 0);
}

export function findModel(catalog: ModelCatalog, catalogKeyValue: string): ModelInfo | undefined {
  const [vendorCode, regionCode, modelId] = splitCatalogKey(catalogKeyValue);
  if (!vendorCode || !regionCode || !modelId) {
    return undefined;
  }
  return findModelByVendorRegion(catalog, vendorCode, regionCode, modelId);
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
  const [vendorCode, regionCode, modelId] = splitCatalogKey(catalogKeyValue);
  if (!vendorCode || !regionCode || !modelId) {
    return [];
  }
  return catalog.vendors
    .filter((vendor) => vendor.vendorCode === vendorCode && vendor.regionCode === regionCode)
    .flatMap((vendor) => vendor.pricing)
    .find((pricing) => pricing.modelId === modelId && pricing.regionCode === regionCode)
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

export function listModelsByProtocol(catalog: ModelCatalog, protocolCode: string): ModelInfo[] {
  return listModels(catalog, { apiFormat: protocolCode });
}

function splitCatalogKey(value: string): [string | undefined, string | undefined, string | undefined] {
  const parts = value.split("/");
  if (parts.length !== 3 || parts.some((part) => part.length === 0)) {
    return [undefined, undefined, undefined];
  }
  return [parts[0], parts[1], parts[2]];
}
