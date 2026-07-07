#!/usr/bin/env node
import { existsSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { projectRootFromTool, readJsonFile } from "./catalog-lib.mjs";
import { inferMissingModelCapabilities } from "./catalog-capability-rules.mjs";

function collectRegionalModelFiles(modelsRoot) {
  const files = [];
  for (const vendorEntry of readdirSync(modelsRoot, { withFileTypes: true })) {
    if (!vendorEntry.isDirectory() || vendorEntry.name.startsWith(".")) {
      continue;
    }
    const vendorDir = join(modelsRoot, vendorEntry.name);
    for (const regionEntry of readdirSync(vendorDir, { withFileTypes: true })) {
      if (!regionEntry.isDirectory()) {
        continue;
      }
      const modelsDir = join(vendorDir, regionEntry.name, "models");
      if (!existsSync(modelsDir) || !statSync(modelsDir).isDirectory()) {
        continue;
      }
      for (const fileEntry of readdirSync(modelsDir, { withFileTypes: true })) {
        if (fileEntry.isFile() && fileEntry.name.endsWith(".json")) {
          files.push(join(modelsDir, fileEntry.name));
        }
      }
    }
  }
  return files.sort((left, right) => left.localeCompare(right));
}

function writeModelFile(path, model) {
  writeFileSync(path, `${JSON.stringify(model, null, 2)}\n`, "utf8");
}

export function alignModelCapabilities(root, options = {}) {
  const dryRun = options.dryRun === true;
  const manifest = readJsonFile(join(root, "sdkwork-models.json"));
  const modelsRoot = join(root, manifest.modelsRoot ?? "models");
  const updates = [];
  const rootNormalized = root.replace(/\\/g, "/");

  for (const path of collectRegionalModelFiles(modelsRoot)) {
    const model = readJsonFile(path);
    const inferred = inferMissingModelCapabilities(model);
    if (Object.keys(inferred).length === 0) {
      continue;
    }
    const relPath = path.replace(/\\/g, "/").replace(`${rootNormalized}/`, "");
    updates.push({ relPath, inferred });
    if (!dryRun) {
      writeModelFile(path, { ...model, ...inferred });
    }
  }

  return updates;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const dryRun = process.argv.includes("--dry-run");
  const updates = alignModelCapabilities(root, { dryRun });
  if (updates.length === 0) {
    console.log("align-model-capabilities: all model files already declare supports* flags");
    process.exit(0);
  }
  for (const update of updates) {
    console.log(
      `${dryRun ? "[dry-run] " : ""}${update.relPath}: ${Object.entries(update.inferred)
        .map(([key, value]) => `${key}=${value}`)
        .join(", ")}`,
    );
  }
  console.log(`${dryRun ? "Would update" : "Updated"} ${updates.length} model file(s)`);
}
