#!/usr/bin/env node
import { loadCatalog, projectRootFromTool } from "./catalog-lib.mjs";

const catalog = loadCatalog(projectRootFromTool(import.meta.url));
const summary = {
  catalogVersion: catalog.manifest.catalogVersion,
  meters: catalog.meters.length,
  vendors: new Set(catalog.vendors.map((vendor) => vendor.vendorCode)).size,
  regions: catalog.vendors.length,
  models: catalog.vendors.reduce((sum, vendor) => sum + vendor.models.length, 0),
  pricingFiles: catalog.vendors.reduce((sum, vendor) => sum + vendor.pricing.length, 0),
};
console.log(JSON.stringify(summary, null, 2));
