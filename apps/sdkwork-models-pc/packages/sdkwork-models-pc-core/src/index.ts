import { isBlank } from "@sdkwork/utils/string";
import { loadCatalog, listAvailableModels, listVendors, type ModelCatalog } from "@sdkwork/models-sdk";

export function resolveBrowserCatalogRoot(): string {
  const configured = process.env.SDKWORK_MODELS_CATALOG_ROOT;
  if (isBlank(configured)) {
    throw new Error("SDKWORK_MODELS_CATALOG_ROOT is not configured for the models PC app.");
  }
  const root = configured!.trim();
  if (root.startsWith("http://") || root.startsWith("https://")) {
    return root;
  }
  if (typeof window !== "undefined") {
    return `${window.location.origin}${root.startsWith("/") ? root : `/${root}`}`;
  }
  return root;
}

export async function loadRepositoryCatalog(): Promise<ModelCatalog> {
  return loadCatalog(resolveBrowserCatalogRoot());
}

export function summarizeCatalog(catalog: ModelCatalog) {
  return {
    catalogVersion: catalog.catalogVersion,
    vendorCount: listVendors(catalog).length,
    availableModelCount: listAvailableModels(catalog).length,
  };
}
