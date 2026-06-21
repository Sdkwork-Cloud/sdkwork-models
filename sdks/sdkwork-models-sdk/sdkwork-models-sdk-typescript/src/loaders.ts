import type { ModelCatalog, VendorCatalog } from "./types.js";

export async function loadCatalog(root: string): Promise<ModelCatalog> {
  const source = catalogSource(root);
  const manifest = await readJson(source, "sdkwork-models.json");
  const meters = (await readJson(source, "models/meters.json")).meters;
  const protocols = (await readJson(source, "models/protocols.json")).protocols;
  const index = await readJson(source, "models/index.json");
  const vendors: VendorCatalog[] = [];
  for (const vendor of index.vendors ?? []) {
    vendors.push(await loadVendorCatalog(root, vendor.vendorCode, vendor.regionCode));
  }
  return {
    catalogVersion: manifest.catalogVersion,
    schemaVersion: manifest.schemaVersion,
    meters,
    protocols,
    vendors,
  };
}

export async function loadVendorCatalog(root: string, vendorCode: string, regionCode: string): Promise<VendorCatalog> {
  const source = catalogSource(root);
  const index = await readJson(source, "models/index.json");
  const vendorIndex = index.vendors.find(
    (item: { vendorCode: string; regionCode: string }) => item.vendorCode === vendorCode && item.regionCode === regionCode,
  );
  if (!vendorIndex) {
    throw new Error(`vendor region ${vendorCode}/${regionCode} is not indexed`);
  }
  const vendor = await readJson(source, `models/${vendorIndex.path}`);
  const models = await Promise.all((vendorIndex.modelFiles ?? []).map((path: string) => readJson(source, `models/${path}`)));
  const pricing = await Promise.all((vendorIndex.pricingFiles ?? []).map((path: string) => readJson(source, `models/${path}`)));
  return { vendorCode, regionCode, vendor, models, pricing };
}

export async function loadBundledCatalog(): Promise<ModelCatalog> {
  const configuredRoot = runtimeEnv().SDKWORK_MODELS_CATALOG_ROOT;
  if (configuredRoot && configuredRoot.trim().length > 0) {
    return loadCatalog(configuredRoot);
  }
  return loadCatalog(await resolveDefaultCatalogRoot());
}

interface CatalogSource {
  root: string;
  remote: boolean;
}

function catalogSource(root: string): CatalogSource {
  return { root: root.replace(/[\\/]+$/, ""), remote: isRemoteUrl(root) };
}

function isRemoteUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

function runtimeEnv(): Record<string, string | undefined> {
  return (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env ?? {};
}

async function resolveDefaultCatalogRoot(): Promise<string> {
  const fs = await import("node:fs");
  const { dirname, join } = await import("node:path");
  const { fileURLToPath } = await import("node:url");
  let current = dirname(fileURLToPath(import.meta.url));
  for (let depth = 0; depth < 8; depth += 1) {
    if (fs.existsSync(join(current, "sdkwork-models.json"))) {
      return current;
    }
    const parent = dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }
  return "data/sdkwork-models";
}

async function readJson(source: CatalogSource, path: string): Promise<any> {
  if (source.remote) {
    const response = await fetch(`${source.root}/${path}`);
    if (!response.ok) {
      throw new Error(`failed to fetch sdkwork-models catalog file ${path}: ${response.status}`);
    }
    return response.json();
  }
  const { readFile } = await import("node:fs/promises");
  const { join } = await import("node:path");
  return JSON.parse(await readFile(join(source.root, ...path.split("/")), "utf8"));
}
