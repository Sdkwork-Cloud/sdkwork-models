#!/usr/bin/env node
import { existsSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import {
  collectRegionalCatalogDirectories,
  loadMeters,
  loadVendorBundle,
  modelIdPath,
  projectRootFromTool,
  readJsonFile,
} from "./catalog-lib.mjs";
import { inferPricingIdentityFixes } from "./catalog-capability-rules.mjs";

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

export function alignModelPricing(root, options = {}) {
  const dryRun = options.dryRun === true;
  const manifest = readJsonFile(join(root, "sdkwork-models.json"));
  const modelsRoot = join(root, manifest.modelsRoot ?? "models");
  const meterCodes = new Set(loadMeters(root).map((meter) => meter.meterCode));
  const updates = [];
  const warnings = [];
  const rootNormalized = root.replace(/\\/g, "/");

  for (const regionDir of collectRegionalCatalogDirectories(modelsRoot)) {
    const bundle = loadVendorBundle(regionDir);
    const modelsById = new Map(bundle.models.map((model) => [model.modelId, model]));
    const pricingById = new Map(bundle.pricing.map((pricing) => [pricing.modelId, pricing]));

    for (const model of bundle.models) {
      const pricingRel = `models/${bundle.vendorCode}/${bundle.regionCode}/pricing/${modelIdPath(model.modelId)}.json`;
      const pricingPath = join(root, pricingRel);
      const pricing = pricingById.get(model.modelId);

      if (!pricing) {
        if (model.routingState === "enabled" || model.shelfState === "listed") {
          warnings.push({
            code: "pricing.file.missing",
            path: pricingRel,
            message: `${model.catalogKey} is routable but has no pricing file`,
          });
        }
        continue;
      }

      const fixes = inferPricingIdentityFixes(model, bundle.vendor, pricing);
      if (Object.keys(fixes).length > 0) {
        updates.push({ relPath: pricingRel, fixes });
        if (!dryRun && existsSync(pricingPath)) {
          writeJson(pricingPath, { ...pricing, ...fixes });
        }
      }

      for (const [index, price] of (pricing.prices ?? []).entries()) {
        if (!meterCodes.has(price.meterCode)) {
          warnings.push({
            code: "pricing.meter.unknown",
            path: `${pricingRel}#/prices/${index}`,
            message: `unknown meterCode ${price.meterCode}`,
          });
        }
      }
    }

    for (const pricing of bundle.pricing) {
      if (!modelsById.has(pricing.modelId)) {
        warnings.push({
          code: "pricing.model.orphan",
          path: `models/${bundle.vendorCode}/${bundle.regionCode}/pricing/${modelIdPath(pricing.modelId)}.json`,
          message: `pricing exists without model ${pricing.modelId}`,
        });
      }
    }
  }

  return { updates, warnings };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const dryRun = process.argv.includes("--dry-run");
  const { updates, warnings } = alignModelPricing(root, { dryRun });
  for (const update of updates) {
    console.log(
      `${dryRun ? "[dry-run] " : ""}${update.relPath}: ${Object.entries(update.fixes)
        .map(([key, value]) => `${key}=${JSON.stringify(value)}`)
        .join(", ")}`,
    );
  }
  for (const warning of warnings) {
    console.warn(`[warn] ${warning.path}: ${warning.message}`);
  }
  if (updates.length === 0 && warnings.length === 0) {
    console.log("align-model-pricing: pricing identity fields are aligned");
  } else {
    console.log(
      `${dryRun ? "Would update" : "Updated"} ${updates.length} pricing file(s); ${warnings.length} warning(s)`,
    );
  }
}
