#!/usr/bin/env node
import { join } from "node:path";
import {
  buildCatalogIndex,
  buildVendorList,
  projectRootFromTool,
  readJsonFile,
  stableJson,
  writeJsonFile,
} from "./catalog-lib.mjs";

const root = projectRootFromTool(import.meta.url);
const check = process.argv.includes("--check");
const indexPath = join(root, "models", "index.json");
const vendorsPath = join(root, "models", "vendors.json");
const nextIndex = buildCatalogIndex(root);
const nextVendors = buildVendorList(root);

if (check) {
  const currentIndex = readJsonFile(indexPath);
  const currentVendors = readJsonFile(vendorsPath);
  const ok =
    stableJson(currentIndex) === stableJson(nextIndex) &&
    stableJson(currentVendors) === stableJson(nextVendors);
  if (!ok) {
    console.error("sdkwork-models index is not current");
    process.exit(1);
  }
  console.log("sdkwork-models index is current");
  process.exit(0);
}

writeJsonFile(indexPath, nextIndex);
writeJsonFile(vendorsPath, nextVendors);
console.log("sdkwork-models index rebuilt");
