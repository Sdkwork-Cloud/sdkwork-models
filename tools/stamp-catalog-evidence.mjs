#!/usr/bin/env node
import { join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import {
  collectJsonFiles,
  officialSnapshotHash,
  projectRootFromTool,
  readJsonFile,
  writeJsonFile,
} from "./catalog-lib.mjs";

function parseArgs(argv) {
  const observedAtArg = argv.indexOf("--observed-at");
  const catalogVersionArg = argv.indexOf("--catalog-version");
  const vendorArg = argv.indexOf("--vendor");
  return {
    observedAt: observedAtArg >= 0 ? argv[observedAtArg + 1] : "2026-06-24T00:00:00Z",
    catalogVersion: catalogVersionArg >= 0 ? argv[catalogVersionArg + 1] : "2026.06.24.1",
    vendorCode: vendorArg >= 0 ? argv[vendorArg + 1] : null,
  };
}

function stampSources(value, observedAt) {
  if (Array.isArray(value)) {
    return value.map((item) => stampSources(item, observedAt));
  }
  if (value && typeof value === "object") {
    const next = {};
    for (const [key, child] of Object.entries(value)) {
      if (key === "source" && child && typeof child === "object" && child.sourceUrl && child.observedAt) {
        next[key] = { ...child, observedAt };
      } else {
        next[key] = stampSources(child, observedAt);
      }
    }
    return next;
  }
  return value;
}

function matchesVendorFilter(relativePath, vendorCode) {
  if (!vendorCode) {
    return true;
  }
  const segments = relativePath.split(/[\\/]/);
  return segments[0] === vendorCode;
}

function stampCatalogEvidence(root, options = {}) {
  const { observedAt, catalogVersion, vendorCode } = options;
  const modelsRoot = join(root, "models");
  const skipped = new Set(["index.json", "vendors.json", "meters.json"]);
  let stampedFileCount = 0;

  for (const file of collectJsonFiles(modelsRoot)) {
    const fileName = file.split(/[\\/]/).pop();
    if (skipped.has(fileName)) {
      continue;
    }
    const relativePath = relative(modelsRoot, file);
    if (!matchesVendorFilter(relativePath, vendorCode)) {
      continue;
    }
    const current = readJsonFile(file);
    const next = stampSources(current, observedAt);
    writeJsonFile(file, next);
    stampedFileCount += 1;
  }

  if (!vendorCode) {
    const manifest = readJsonFile(join(root, "sdkwork-models.json"));
    manifest.catalogVersion = catalogVersion;
    manifest.generatedAt = observedAt;
    writeJsonFile(join(root, "sdkwork-models.json"), manifest);

    const vendorSources = readJsonFile(join(root, "sources", "vendor-sources.json"));
    vendorSources.catalogVersion = catalogVersion;
    vendorSources.observedAt = observedAt;
    for (const vendor of vendorSources.vendors ?? []) {
      vendor.lastCheckedAt = observedAt;
    }
    writeJsonFile(join(root, "sources", "vendor-sources.json"), vendorSources);

    const officialVerificationPolicy = readJsonFile(join(root, "sources", "official-verification-policy.json"));
    officialVerificationPolicy.catalogVersion = catalogVersion;
    officialVerificationPolicy.generatedAt = observedAt;
    writeJsonFile(join(root, "sources", "official-verification-policy.json"), officialVerificationPolicy);

    const officialModelSnapshots = readJsonFile(join(root, "sources", "official-model-snapshots.json"));
    officialModelSnapshots.catalogVersion = catalogVersion;
    officialModelSnapshots.observedAt = observedAt;
    for (const snapshot of officialModelSnapshots.vendors ?? []) {
      snapshot.observedAt = observedAt;
      snapshot.sourceSnapshotHash = officialSnapshotHash(snapshot);
    }
    writeJsonFile(join(root, "sources", "official-model-snapshots.json"), officialModelSnapshots);
  } else {
    const vendorSources = readJsonFile(join(root, "sources", "vendor-sources.json"));
    for (const vendor of vendorSources.vendors ?? []) {
      if (vendor.vendorCode === vendorCode) {
        vendor.lastCheckedAt = observedAt;
      }
    }
    writeJsonFile(join(root, "sources", "vendor-sources.json"), vendorSources);

    const officialModelSnapshots = readJsonFile(join(root, "sources", "official-model-snapshots.json"));
    for (const snapshot of officialModelSnapshots.vendors ?? []) {
      if (snapshot.vendorCode !== vendorCode) {
        continue;
      }
      snapshot.observedAt = observedAt;
      snapshot.sourceSnapshotHash = officialSnapshotHash(snapshot);
    }
    writeJsonFile(join(root, "sources", "official-model-snapshots.json"), officialModelSnapshots);
  }

  return { stampedFileCount, observedAt, catalogVersion, vendorCode };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const result = stampCatalogEvidence(root, parseArgs(process.argv.slice(2)));
  console.log(JSON.stringify(result, null, 2));
}

export { stampCatalogEvidence };
