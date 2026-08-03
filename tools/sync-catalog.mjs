#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { loadCatalog, loadMeters, projectRootFromTool, readJsonFile } from "./catalog-lib.mjs";
import { alignModelCapabilities } from "./align-model-capabilities.mjs";
import { alignModelPricing } from "./align-model-pricing.mjs";
import {
  inferMissingModelCapabilities,
  isBillableModel,
  pricingMeterAllowedForModel,
} from "./catalog-capability-rules.mjs";
import { validateCatalog } from "./validate-catalog.mjs";

function runNodeScript(root, scriptName) {
  const scriptPath = join(root, "tools", scriptName);
  const result = spawnSync(process.execPath, [scriptPath], {
    cwd: root,
    encoding: "utf8",
    stdio: "pipe",
  });
  return {
    ok: result.status === 0,
    status: result.status ?? 1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

export function syncCatalog(root, options = {}) {
  const dryRun = options.dryRun === true;
  const skipIndex = options.skipIndex === true;
  const skipValidate = options.skipValidate === true;

  const capabilityUpdates = alignModelCapabilities(root, { dryRun });
  const { updates: pricingUpdates, warnings: pricingWarnings } = alignModelPricing(root, { dryRun });

  let indexResult = { ok: true, stdout: "skipped" };
  if (!dryRun && !skipIndex) {
    indexResult = runNodeScript(root, "build-index.mjs");
  }

  const catalog = loadCatalog(root);
  const meterModalities = new Map(loadMeters(root).map((meter) => [meter.meterCode, meter.modality]));
  const vendorSources = readJsonFile(join(root, "sources/vendor-sources.json"));
  const sourceByRegion = new Map(
    (vendorSources.vendors ?? []).map((vendor) => [`${vendor.vendorCode}/${vendor.regionCode}`, vendor]),
  );

  const report = {
    dryRun,
    capabilityUpdates: capabilityUpdates.length,
    pricingUpdates: pricingUpdates.length,
    pricingWarnings: pricingWarnings.length,
    models: catalog.vendors.reduce((count, vendor) => count + vendor.models.length, 0),
    pricingFiles: catalog.vendors.reduce((count, vendor) => count + vendor.pricing.length, 0),
    missingCapabilityFlags: 0,
    missingPricingForBillable: [],
    missingRequiredModels: [],
    meterMismatches: [],
    indexRebuilt: !dryRun && !skipIndex && indexResult.ok,
  };

  for (const vendor of catalog.vendors) {
    const regionalKey = `${vendor.vendorCode}/${vendor.regionCode}`;
    const source = sourceByRegion.get(regionalKey);
    const modelIds = new Set(vendor.models.map((model) => model.modelId));
    const pricingIds = new Set(vendor.pricing.map((pricing) => pricing.modelId));

    for (const modelId of source?.requiredModels ?? []) {
      if (!modelIds.has(modelId)) {
        report.missingRequiredModels.push(`${regionalKey}/${modelId} (model)`);
      }
      if (!pricingIds.has(modelId)) {
        report.missingRequiredModels.push(`${regionalKey}/${modelId} (pricing)`);
      }
    }

    for (const model of vendor.models) {
      for (const field of ["supportsStreaming", "supportsTools", "supportsJsonSchema", "codingVisible"]) {
        if (typeof model[field] !== "boolean") {
          report.missingCapabilityFlags += 1;
        }
      }
      if (!Array.isArray(model.usageScopes)) {
        report.missingCapabilityFlags += 1;
      }
      if (Object.keys(inferMissingModelCapabilities(model)).length > 0) {
        report.missingCapabilityFlags += 1;
      }
      if (isBillableModel(model) && !pricingIds.has(model.modelId)) {
        report.missingPricingForBillable.push(model.catalogKey);
      }

      const pricing = vendor.pricing.find((entry) => entry.modelId === model.modelId);
      for (const price of pricing?.prices ?? []) {
        if (!pricingMeterAllowedForModel(model, price.meterCode, meterModalities)) {
          report.meterMismatches.push({
            catalogKey: model.catalogKey,
            meterCode: price.meterCode,
            primaryCapability: model.primaryCapability,
          });
        }
      }
    }
  }

  let validation = { ok: true, issues: [] };
  if (!dryRun && !skipValidate) {
    validation = validateCatalog(root);
  }

  return {
    report,
    capabilityUpdates,
    pricingUpdates,
    pricingWarnings,
    indexResult,
    validation,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const dryRun = process.argv.includes("--dry-run");
  const result = syncCatalog(root, { dryRun });

  console.log("sdkwork-models catalog sync");
  console.log(`  models: ${result.report.models}`);
  console.log(`  pricing files: ${result.report.pricingFiles}`);
  console.log(`  capability updates: ${result.report.capabilityUpdates}`);
  console.log(`  pricing identity updates: ${result.report.pricingUpdates}`);
  console.log(`  pricing warnings: ${result.report.pricingWarnings}`);
  console.log(`  missing capability flags: ${result.report.missingCapabilityFlags}`);
  console.log(`  billable models missing pricing: ${result.report.missingPricingForBillable.length}`);
  console.log(`  required model/pricing gaps: ${result.report.missingRequiredModels.length}`);
  console.log(`  meter/capability mismatches: ${result.report.meterMismatches.length}`);

  if (result.capabilityUpdates.length > 0) {
    console.log("\nCapability updates:");
    for (const update of result.capabilityUpdates) {
      console.log(
        `  ${update.relPath}: ${Object.entries(update.inferred)
          .map(([key, value]) => `${key}=${value}`)
          .join(", ")}`,
      );
    }
  }

  if (result.pricingUpdates.length > 0) {
    console.log("\nPricing identity updates:");
    for (const update of result.pricingUpdates) {
      console.log(
        `  ${update.relPath}: ${Object.entries(update.fixes)
          .map(([key, value]) => `${key}=${JSON.stringify(value)}`)
          .join(", ")}`,
      );
    }
  }

  if (result.report.missingRequiredModels.length > 0) {
    console.log("\nRequired source contract gaps:");
    for (const gap of result.report.missingRequiredModels.slice(0, 20)) {
      console.log(`  ${gap}`);
    }
    if (result.report.missingRequiredModels.length > 20) {
      console.log(`  ... and ${result.report.missingRequiredModels.length - 20} more`);
    }
  }

  if (result.report.meterMismatches.length > 0) {
    console.log("\nMeter/capability mismatches (review manually):");
    for (const item of result.report.meterMismatches.slice(0, 15)) {
      console.log(`  ${item.catalogKey}: ${item.meterCode} vs ${item.primaryCapability}`);
    }
  }

  if (!dryRun) {
    if (!result.indexResult.ok) {
      console.error(result.indexResult.stderr || result.indexResult.stdout);
      process.exit(1);
    }
    if (!result.validation.ok) {
      console.error(JSON.stringify(result.validation, null, 2));
      process.exit(1);
    }
    console.log("\nIndex rebuilt and catalog validation passed.");
  }

  const hasBlockers =
    result.report.missingCapabilityFlags > 0
    || result.report.missingPricingForBillable.length > 0
    || result.report.missingRequiredModels.length > 0;

  process.exit(hasBlockers ? 1 : 0);
}
