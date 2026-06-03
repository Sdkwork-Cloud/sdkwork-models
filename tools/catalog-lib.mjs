import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative } from "node:path";

export const DECIMAL_PATTERN = /^(0|[1-9][0-9]*)(\.[0-9]+)?$/;

export function readJsonFile(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

export function writeJsonFile(path, value) {
  writeFileSync(path, `${stableJson(value)}\n`, "utf8");
}

export function stableJson(value) {
  return `${JSON.stringify(sortJson(value), null, 2)}`;
}

export function sortJson(value) {
  if (Array.isArray(value)) {
    return value.map(sortJson);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, sortJson(value[key])]),
    );
  }
  return value;
}

export function sha256Text(text) {
  return createHash("sha256").update(text).digest("hex");
}

export function officialSnapshotHash(snapshot) {
  const { sourceSnapshotHash, ...canonicalSnapshot } = snapshot;
  return sha256Text(stableJson(canonicalSnapshot));
}

export function officialSnapshotVendorRegion(snapshot) {
  return `${snapshot.vendorCode}/${snapshot.regionCode ?? "global"}`;
}

export function officialSnapshotHashesByRegion(officialSnapshots) {
  return Object.fromEntries(
    [...(officialSnapshots.vendors ?? [])]
      .map((snapshot) => [officialSnapshotVendorRegion(snapshot), snapshot.sourceSnapshotHash ?? officialSnapshotHash(snapshot)])
      .sort(([left], [right]) => left.localeCompare(right)),
  );
}

export function isDecimalString(value) {
  return typeof value === "string" && DECIMAL_PATTERN.test(value);
}

export function issue(code, path, message, severity = "error") {
  return { code, path, message, severity };
}

export function collectRegionalCatalogDirectories(modelsRoot) {
  const regionDirs = [];
  for (const vendorEntry of readdirSync(modelsRoot, { withFileTypes: true })) {
    if (!vendorEntry.isDirectory()) {
      continue;
    }
    const vendorRoot = join(modelsRoot, vendorEntry.name);
    for (const regionEntry of readdirSync(vendorRoot, { withFileTypes: true })) {
      if (!regionEntry.isDirectory()) {
        continue;
      }
      const regionRoot = join(vendorRoot, regionEntry.name);
      if (statSync(join(regionRoot, "vendor.json")).isFile()) {
        regionDirs.push(regionRoot);
      }
    }
  }
  return regionDirs.sort();
}

export const collectVendorDirectories = collectRegionalCatalogDirectories;

export function collectJsonFiles(root) {
  const files = [];
  function visit(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(path);
        continue;
      }
      if (entry.isFile() && entry.name.endsWith(".json")) {
        files.push(path);
      }
    }
  }
  visit(root);
  return files.sort((left, right) => left.localeCompare(right));
}

function collectJsonFileRefs(modelsRoot, root) {
  return collectJsonFiles(root).map((path) => relative(modelsRoot, path).replace(/\\/g, "/"));
}

export function vendorCodeFromDirectory(regionDir) {
  return basename(dirname(regionDir));
}

export function regionCodeFromDirectory(regionDir) {
  return basename(regionDir);
}

export function catalogKey(vendorCode, modelId) {
  return `${vendorCode}/${modelId}`;
}

export function projectRootFromTool(importMetaUrl) {
  return dirname(dirname(new URL(importMetaUrl).pathname.replace(/^\/([A-Za-z]:)/, "$1")));
}

export function loadManifest(root) {
  return readJsonFile(join(root, "sdkwork-models.json"));
}

export function loadMeters(root) {
  return readJsonFile(join(root, "models", "meters.json")).meters ?? [];
}

export function loadVendorBundle(regionDir) {
  const vendorCode = vendorCodeFromDirectory(regionDir);
  const regionCode = regionCodeFromDirectory(regionDir);
  const vendor = readJsonFile(join(regionDir, "vendor.json"));
  const families = readJsonFile(join(regionDir, "families.json"));
  const models = collectJsonFiles(join(regionDir, "models")).map(readJsonFile);
  const pricing = collectJsonFiles(join(regionDir, "pricing")).map(readJsonFile);
  const rankingsPath = join(regionDir, "rankings.json");
  const rankings = statSync(rankingsPath).isFile()
    ? readJsonFile(rankingsPath)
    : { schemaVersion: vendor.schemaVersion, vendorCode, regionCode, snapshots: [] };
  return {
    vendorCode,
    regionCode,
    catalogRoot: regionDir,
    vendor,
    families,
    models,
    pricing,
    rankings,
  };
}

export function vendorHash(regionDir) {
  const files = [
    join(regionDir, "vendor.json"),
    join(regionDir, "families.json"),
    ...collectJsonFiles(join(regionDir, "models")),
    ...collectJsonFiles(join(regionDir, "pricing")),
    join(regionDir, "rankings.json"),
  ];
  const body = files
    .filter((path) => statSync(path).isFile())
    .map((path) => `${relative(dirname(dirname(regionDir)), path)}\n${stableJson(readJsonFile(path))}`)
    .join("\n");
  return sha256Text(body);
}

export function buildCatalogIndex(root) {
  const manifest = loadManifest(root);
  const modelsRoot = join(root, manifest.modelsRoot);
  const regionalCatalogDirs = collectRegionalCatalogDirectories(modelsRoot);
  const vendors = regionalCatalogDirs.map((regionDir) => {
    const { vendor, families, models, pricing, rankings, vendorCode, regionCode } = loadVendorBundle(regionDir);
    const modelFiles = collectJsonFileRefs(modelsRoot, join(regionDir, "models"));
    const pricingFiles = collectJsonFileRefs(modelsRoot, join(regionDir, "pricing"));
    return {
      vendorCode,
      regionCode,
      catalogKeyPrefix: `${vendorCode}/`,
      displayName: vendor.displayName,
      marketScope: vendor.marketScope,
      billingCurrency: vendor.billingCurrency,
      billingJurisdiction: vendor.billingJurisdiction,
      path: `${vendorCode}/${regionCode}/vendor.json`,
      familiesPath: `${vendorCode}/${regionCode}/families.json`,
      modelsPath: `${vendorCode}/${regionCode}/models`,
      modelFiles,
      pricingPath: `${vendorCode}/${regionCode}/pricing`,
      pricingFiles,
      rankingsPath: `${vendorCode}/${regionCode}/rankings.json`,
      modelCount: models.length,
      familyCount: families.families?.length ?? 0,
      pricingFileCount: pricing.length,
      rankingSnapshotCount: rankings.snapshots?.length ?? 0,
      sha256: vendorHash(regionDir),
    };
  });
  const vendorCount = new Set(vendors.map((vendor) => vendor.vendorCode)).size;
  const modelCount = vendors.reduce((sum, vendor) => sum + vendor.modelCount, 0);
  const pricingFileCount = vendors.reduce((sum, vendor) => sum + vendor.pricingFileCount, 0);
  return {
    schemaVersion: manifest.schemaVersion,
    catalogVersion: manifest.catalogVersion,
    generatedAt: manifest.generatedAt,
    vendorCount,
    regionCount: vendors.length,
    modelCount,
    pricingFileCount,
    vendors,
  };
}

export function buildVendorList(root) {
  const manifest = loadManifest(root);
  const regionalCatalogDirs = collectRegionalCatalogDirectories(join(root, manifest.modelsRoot));
  const vendorMap = new Map();
  for (const regionDir of regionalCatalogDirs) {
    const { vendor, models, pricing, rankings, vendorCode, regionCode } = loadVendorBundle(regionDir);
    const existing = vendorMap.get(vendorCode) ?? {
      vendorCode,
      displayName: vendor.displayName,
      legalName: vendor.legalName ?? vendor.displayName,
      vendorType: vendor.vendorType,
      capabilities: [],
      supportedProtocols: [],
      clientApiCompatibility: {},
      openSource: Boolean(vendor.openSource),
      sortOrder: vendor.sortOrder ?? 1000000,
      regions: [],
    };
    existing.capabilities = [
      ...new Set([...(existing.capabilities ?? []), ...(vendor.capabilities ?? [])]),
    ].sort();
    existing.supportedProtocols = [
      ...new Set([...(existing.supportedProtocols ?? []), ...(vendor.supportedProtocols ?? [])]),
    ].sort();
    existing.clientApiCompatibility = mergeClientApiCompatibility(
      existing.clientApiCompatibility,
      vendor.clientApiCompatibility ?? {},
    );
    existing.sortOrder = Math.min(existing.sortOrder, vendor.sortOrder ?? 1000000);
    existing.regions.push({
      regionCode,
      displayName: vendor.displayName,
      legalName: vendor.legalName ?? vendor.displayName,
      marketScope: vendor.marketScope,
      billingCurrency: vendor.billingCurrency,
      billingJurisdiction: vendor.billingJurisdiction,
      operatingRegions: vendor.operatingRegions ?? [],
      capabilities: vendor.capabilities ?? [],
      supportedProtocols: vendor.supportedProtocols ?? [],
      clientApiCompatibility: vendor.clientApiCompatibility ?? {},
      openSource: Boolean(vendor.openSource),
      sortOrder: vendor.sortOrder ?? 1000000,
      path: `${vendorCode}/${regionCode}/vendor.json`,
      modelCount: models.length,
      pricingFileCount: pricing.length,
      rankingSnapshotCount: rankings.snapshots?.length ?? 0,
    });
    existing.regions.sort((left, right) => left.regionCode.localeCompare(right.regionCode));
    vendorMap.set(vendorCode, existing);
  }
  return {
    schemaVersion: manifest.schemaVersion,
    catalogVersion: manifest.catalogVersion,
    vendors: [...vendorMap.values()].sort((left, right) => left.sortOrder - right.sortOrder || left.vendorCode.localeCompare(right.vendorCode)),
  };
}

export function loadCatalog(root) {
  const manifest = loadManifest(root);
  const modelsRoot = join(root, manifest.modelsRoot);
  return {
    manifest,
    meters: loadMeters(root),
    vendors: collectRegionalCatalogDirectories(modelsRoot).map(loadVendorBundle),
  };
}

export function modelFileName(modelId) {
  return `${modelIdPath(modelId)}.json`;
}

export function modelIdPath(modelId) {
  if (typeof modelId !== "string" || modelId.length === 0 || modelId.includes("\\")) {
    throw new Error(`invalid modelId path: ${modelId}`);
  }
  const segments = modelId.split("/");
  if (segments.some((segment) => segment.length === 0 || segment === "." || segment === "..")) {
    throw new Error(`invalid modelId path: ${modelId}`);
  }
  return segments.join("/");
}

function mergeClientApiCompatibility(left, right) {
  const merged = { ...(left ?? {}) };
  for (const [clientApiCode, item] of Object.entries(right ?? {})) {
    const previous = merged[clientApiCode];
    if (!previous || clientApiSupportRank(item.supportStatus) > clientApiSupportRank(previous.supportStatus)) {
      merged[clientApiCode] = item;
    }
  }
  return merged;
}

function clientApiSupportRank(status) {
  switch (status) {
    case "supported":
      return 3;
    case "partial":
      return 2;
    case "unsupported":
      return 1;
    default:
      return 0;
  }
}
