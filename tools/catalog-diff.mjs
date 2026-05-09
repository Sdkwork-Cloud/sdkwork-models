#!/usr/bin/env node
import { loadCatalog, projectRootFromTool } from "./catalog-lib.mjs";

const args = process.argv.slice(2);
const fromArg = args.indexOf("--from");
const toArg = args.indexOf("--to");
const fromRoot = fromArg >= 0 ? args[fromArg + 1] : null;
const toRoot = toArg >= 0 ? args[toArg + 1] : projectRootFromTool(import.meta.url);

function modelMap(catalog) {
  const map = new Map();
  for (const vendor of catalog.vendors) {
    for (const model of vendor.models) {
      map.set(`${vendor.vendorCode}/${vendor.regionCode}/${model.modelId}`, {
        vendorCode: vendor.vendorCode,
        regionCode: vendor.regionCode,
        model,
      });
    }
  }
  return map;
}

function priceMap(catalog) {
  const map = new Map();
  for (const vendor of catalog.vendors) {
    for (const pricing of vendor.pricing) {
      for (const price of pricing.prices ?? []) {
        map.set(`${vendor.vendorCode}/${vendor.regionCode}/${pricing.modelId}:${price.meterCode}:${price.priceSide}`, price);
      }
    }
  }
  return map;
}

if (!fromRoot) {
  const to = loadCatalog(toRoot);
  console.log(
    JSON.stringify(
      {
        fromCatalogVersion: null,
        toCatalogVersion: to.manifest.catalogVersion,
        vendorChanges: to.vendors.map((vendor) => ({
          vendorCode: vendor.vendorCode,
          regionCode: vendor.regionCode,
          addedModels: vendor.models.map((model) => model.modelId),
          changedModels: [],
          deprecatedModels: [],
          retiredModels: [],
          priceChanges: [],
        })),
      },
      null,
      2,
    ),
  );
  process.exit(0);
}

const from = loadCatalog(fromRoot);
const to = loadCatalog(toRoot);
const fromModels = modelMap(from);
const toModels = modelMap(to);
const fromPrices = priceMap(from);
const toPrices = priceMap(to);
const vendorRegions = new Set([...from.vendors, ...to.vendors].map((vendor) => `${vendor.vendorCode}/${vendor.regionCode}`));
const vendorChanges = [];

for (const vendorRegion of [...vendorRegions].sort()) {
  const [vendorCode, regionCode] = vendorRegion.split("/");
  const addedModels = [];
  const changedModels = [];
  const deprecatedModels = [];
  const retiredModels = [];
  const priceChanges = [];
  for (const [modelId, entry] of toModels) {
    if (entry.vendorCode !== vendorCode || entry.regionCode !== regionCode) continue;
    if (!fromModels.has(modelId)) {
      addedModels.push(entry.model.modelId);
    } else if (JSON.stringify(fromModels.get(modelId).model) !== JSON.stringify(entry.model)) {
      changedModels.push(entry.model.modelId);
    }
    if (entry.model.lifecycle === "deprecated") deprecatedModels.push(entry.model.modelId);
    if (entry.model.lifecycle === "retired") retiredModels.push(entry.model.modelId);
  }
  for (const [priceKey, price] of toPrices) {
    const [catalogModelKey, meterCode] = priceKey.split(":");
    const model = toModels.get(catalogModelKey);
    if (model?.vendorCode !== vendorCode || model?.regionCode !== regionCode) continue;
    const fromPrice = fromPrices.get(priceKey);
    if (fromPrice && fromPrice.unitPrice !== price.unitPrice) {
      priceChanges.push({
        modelId: model.model.modelId,
        meterCode,
        fromUnitPrice: fromPrice.unitPrice,
        toUnitPrice: price.unitPrice,
      });
    }
  }
  vendorChanges.push({ vendorCode, regionCode, addedModels, changedModels, deprecatedModels, retiredModels, priceChanges });
}

console.log(
  JSON.stringify(
    {
      fromCatalogVersion: from.manifest.catalogVersion,
      toCatalogVersion: to.manifest.catalogVersion,
      vendorChanges,
    },
    null,
    2,
  ),
);
