#!/usr/bin/env node
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { loadCatalog, projectRootFromTool, readJsonFile } from "./catalog-lib.mjs";

export function createFreshnessReport(root, options = {}) {
  const policyPath = options.policyPath ?? "catalog-freshness-policy.json";
  const catalog = loadCatalog(root);
  const asOf = options.asOf ?? (
    options.asOfCatalogGeneratedAt ? catalog.manifest.generatedAt?.slice(0, 10) : undefined
  );
  const asOfDate = asOf
    ? new Date(`${asOf}T00:00:00Z`)
    : new Date();
  const policy = readJsonFile(join(root, policyPath));
  const staleSources = [];
  const warnings = [];

  function vendorRegionKey(vendorCode, regionCode) {
    return `${vendorCode}/${regionCode}`;
  }

  function matchingRule(scope, vendorCode, regionCode) {
    const rules = policy.rules ?? [];
    const regionalKey = vendorRegionKey(vendorCode, regionCode);
    return (
      rules.find((rule) => rule.scope === scope && rule.vendorCode === vendorCode && rule.regionCode === regionCode) ??
      rules.find((rule) => rule.scope === scope && rule.vendorRegion === regionalKey) ??
      rules.find((rule) => rule.scope === scope && rule.vendorCode === vendorCode) ??
      rules.find((rule) => rule.scope === scope && rule.vendorCode === "*") ?? {
        maxSourceAgeDays: policy.defaultMaxSourceAgeDays ?? 30,
        severity: "error",
      }
    );
  }

  function daysBetween(left, right) {
    return Math.floor((left.getTime() - right.getTime()) / 86_400_000);
  }

  function checkSource(scope, vendorCode, regionCode, ref, source) {
    if (!source?.observedAt) {
      staleSources.push({ scope, vendorCode, regionCode, ref, reason: "missing_observed_at", severity: "error" });
      return;
    }
    const observedAt = new Date(source.observedAt);
    const rule = matchingRule(scope, vendorCode, regionCode);
    const ageDays = daysBetween(asOfDate, observedAt);
    if (ageDays > rule.maxSourceAgeDays) {
      const item = {
        scope,
        vendorCode,
        regionCode,
        ref,
        observedAt: source.observedAt,
        ageDays,
        maxSourceAgeDays: rule.maxSourceAgeDays,
        severity: rule.severity,
      };
      if (rule.severity === "warning") {
        warnings.push(item);
      } else {
        staleSources.push(item);
      }
    }
  }

  for (const vendor of catalog.vendors) {
    checkSource("model", vendor.vendorCode, vendor.regionCode, `${vendor.vendorCode}/${vendor.regionCode}/vendor.json`, vendor.vendor.source);
    for (const model of vendor.models) {
      checkSource("model", vendor.vendorCode, vendor.regionCode, `${vendor.vendorCode}/${vendor.regionCode}/${model.modelId}`, model.source);
    }
    for (const pricing of vendor.pricing) {
      for (const price of pricing.prices ?? []) {
        checkSource("pricing", vendor.vendorCode, vendor.regionCode, `${vendor.vendorCode}/${vendor.regionCode}/${pricing.modelId}/${price.meterCode}`, price.source);
      }
    }
  }

  return {
    ok: staleSources.length === 0,
    generatedAt: catalog.manifest.generatedAt,
    asOf: asOfDate.toISOString().slice(0, 10),
    staleSources,
    warnings,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const args = process.argv.slice(2);
  const policyArg = args.indexOf("--max-age-policy");
  const asOfArg = args.indexOf("--as-of");
  const asOfCatalogGeneratedAt = args.includes("--as-of-catalog-generated-at");
  const report = createFreshnessReport(root, {
    policyPath: policyArg >= 0 ? args[policyArg + 1] : "catalog-freshness-policy.json",
    asOf: asOfArg >= 0 ? args[asOfArg + 1] : undefined,
    asOfCatalogGeneratedAt,
  });
  console.log(JSON.stringify(report, null, 2));
  if (!report.ok) {
    process.exit(1);
  }
}
