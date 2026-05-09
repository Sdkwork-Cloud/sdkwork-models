#!/usr/bin/env node
import { existsSync } from "node:fs";
import { join } from "node:path";
import {
  buildCatalogIndex,
  officialSnapshotHashesByRegion,
  projectRootFromTool,
  readJsonFile,
  sha256Text,
  stableJson,
  writeJsonFile,
} from "./catalog-lib.mjs";
import { validateCatalog } from "./validate-catalog.mjs";
import { createFreshnessReport } from "./freshness-report.mjs";
import { auditCatalog } from "./catalog-audit.mjs";

const root = projectRootFromTool(import.meta.url);
const args = process.argv.slice(2);
const check = args.includes("--check");
const asOfArg = args.indexOf("--as-of");
const asOf = asOfArg >= 0 ? args[asOfArg + 1] : undefined;

const expectedIndex = buildCatalogIndex(root);
const currentIndex = readJsonFile(join(root, "models", "index.json"));
if (stableJson(currentIndex) !== stableJson(expectedIndex)) {
  console.error("models/index.json is not current");
  process.exit(1);
}

const validation = validateCatalog(root);
if (!validation.ok) {
  console.error(JSON.stringify(validation, null, 2));
  process.exit(1);
}
const freshness = createFreshnessReport(root, {
  policyPath: "catalog-freshness-policy.json",
  asOf,
});
if (!freshness.ok) {
  console.error(JSON.stringify(freshness, null, 2));
  process.exit(1);
}
const sourceAudit = auditCatalog(root, { asOf });
if (!sourceAudit.ok) {
  console.error(JSON.stringify(sourceAudit, null, 2));
  process.exit(1);
}
const manifest = readJsonFile(join(root, "sdkwork-models.json"));
const vendorSources = readJsonFile(join(root, "sources", "vendor-sources.json"));
const officialModelSnapshots = readJsonFile(join(root, "sources", "official-model-snapshots.json"));
const officialVerificationPolicy = readJsonFile(join(root, "sources", "official-verification-policy.json"));
const releasePath = join(root, "releases", `${manifest.catalogVersion}.json`);
const release = {
  schemaVersion: manifest.schemaVersion,
  catalogVersion: manifest.catalogVersion,
  generatedAt: manifest.generatedAt,
  indexSha256: sha256Text(stableJson(expectedIndex)),
  sourceEvidenceSha256: {
    vendorSources: sha256Text(stableJson(vendorSources)),
    officialModelSnapshots: sha256Text(stableJson(officialModelSnapshots)),
    officialVerificationPolicy: sha256Text(stableJson(officialVerificationPolicy)),
    officialSnapshotHashes: officialSnapshotHashesByRegion(officialModelSnapshots),
  },
  validation: {
    ok: validation.ok,
    issueCount: validation.issues?.length ?? 0,
  },
  freshnessReport: freshness,
  sourceAudit: {
    ok: sourceAudit.ok,
    errorCount: sourceAudit.errors?.length ?? 0,
    warningCount: sourceAudit.warnings?.length ?? 0,
    vendorCount: sourceAudit.vendorCount,
    regionCount: sourceAudit.regionCount,
    requiredVerifiedRegionCount: sourceAudit.requiredVerifiedRegionCount,
    requiredVerifiedRegions: sourceAudit.requiredVerifiedRegions,
    officialVerifiedSourceRegionCount: sourceAudit.officialVerifiedSourceRegionCount,
    officialVerifiedSourceRegions: sourceAudit.officialVerifiedSourceRegions,
  },
  vendorChanges: expectedIndex.vendors.map((vendor) => ({
    vendorCode: vendor.vendorCode,
    regionCode: vendor.regionCode,
    modelCount: vendor.modelCount,
    pricingFileCount: vendor.pricingFileCount,
    sha256: vendor.sha256,
  })),
};

if (check) {
  if (!existsSync(releasePath)) {
    console.error(`${releasePath} is missing`);
    process.exit(1);
  }
  const current = readJsonFile(releasePath);
  if (stableJson(current) !== stableJson(release)) {
    console.error(`${releasePath} is not current`);
    process.exit(1);
  }
  console.log(`sdkwork-models release ${manifest.catalogVersion} is current`);
  process.exit(0);
}

writeJsonFile(releasePath, release);
console.log(`sdkwork-models release ${manifest.catalogVersion} written`);
