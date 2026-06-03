#!/usr/bin/env node
import { existsSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { loadCatalog, modelFileName, officialSnapshotHash, projectRootFromTool, readJsonFile } from "./catalog-lib.mjs";

function validateJsonSchema(value, schema, options = {}) {
  const issues = [];
  const defs = schema.$defs ?? {};
  const codePrefix = options.codePrefix ?? "schema";

  function addIssue(kind, path, message) {
    issues.push({ code: `${codePrefix}.${kind}`, path, message });
  }

  function resolveSchema(currentSchema) {
    if (!currentSchema?.$ref) {
      return currentSchema;
    }
    const prefix = "#/$defs/";
    if (!currentSchema.$ref.startsWith(prefix)) {
      return currentSchema;
    }
    return defs[currentSchema.$ref.slice(prefix.length)] ?? currentSchema;
  }

  function pointer(base, segment) {
    return `${base}/${String(segment).replaceAll("~", "~0").replaceAll("/", "~1")}`;
  }

  function typeMatches(currentValue, type) {
    if (type === "array") {
      return Array.isArray(currentValue);
    }
    if (type === "object") {
      return currentValue !== null && typeof currentValue === "object" && !Array.isArray(currentValue);
    }
    if (type === "integer") {
      return Number.isInteger(currentValue);
    }
    return typeof currentValue === type;
  }

  function check(currentValue, currentSchema, path) {
    const resolved = resolveSchema(currentSchema);
    if (!resolved) {
      return;
    }
    if (Array.isArray(resolved.enum) && !resolved.enum.includes(currentValue)) {
      addIssue("enum", path, `${path} must be one of ${resolved.enum.join(", ")}`);
    }
    if (resolved.const !== undefined && currentValue !== resolved.const) {
      addIssue("const", path, `${path} must be ${resolved.const}`);
    }
    if (resolved.type && !typeMatches(currentValue, resolved.type)) {
      addIssue("type", path, `${path} must be ${resolved.type}`);
      return;
    }
    if (typeof currentValue === "string") {
      if (resolved.minLength !== undefined && currentValue.length < resolved.minLength) {
        addIssue("min_length", path, `${path} must not be empty`);
      }
      if (resolved.pattern && !new RegExp(resolved.pattern).test(currentValue)) {
        addIssue("pattern", path, `${path} must match ${resolved.pattern}`);
      }
    }
    if (Array.isArray(currentValue)) {
      if (resolved.minItems !== undefined && currentValue.length < resolved.minItems) {
        addIssue("min_items", path, `${path} must contain at least ${resolved.minItems} item(s)`);
      }
      if (resolved.uniqueItems) {
        const seen = new Set();
        for (const [index, item] of currentValue.entries()) {
          const key = JSON.stringify(item);
          if (seen.has(key)) {
            addIssue("unique_items", pointer(path, index), `${path} must contain unique items`);
          }
          seen.add(key);
        }
      }
      if (resolved.items) {
        for (const [index, item] of currentValue.entries()) {
          check(item, resolved.items, pointer(path, index));
        }
      }
    }
    if (currentValue !== null && typeof currentValue === "object" && !Array.isArray(currentValue)) {
      for (const field of resolved.required ?? []) {
        if (currentValue[field] === undefined) {
          addIssue("required", pointer(path, field), `${field} is required`);
        }
      }
      const properties = resolved.properties ?? {};
      for (const [field, childValue] of Object.entries(currentValue)) {
        const childSchema = properties[field];
        if (!childSchema) {
          if (resolved.additionalProperties === false) {
            addIssue("additional_property", pointer(path, field), `${field} is not allowed`);
          }
          continue;
        }
        check(childValue, childSchema, pointer(path, field));
      }
    }
  }

  check(value, schema, options.path ?? "#");
  return issues;
}

export function auditCatalog(root, options = {}) {
  const asOf = options.asOf ?? new Date().toISOString().slice(0, 10);
  const sourcesPath = options.sourcesPath ?? "sources/vendor-sources.json";
  const officialSnapshotsPath = options.officialSnapshotsPath ?? "sources/official-model-snapshots.json";
  const officialVerificationPolicyPath = options.officialVerificationPolicyPath ?? "sources/official-verification-policy.json";
  const sources = readJsonFile(join(root, sourcesPath));
  const officialSnapshots = existsSync(join(root, officialSnapshotsPath))
    ? readJsonFile(join(root, officialSnapshotsPath))
    : { vendors: [] };
  const officialVerificationPolicyExists = existsSync(join(root, officialVerificationPolicyPath));
  const officialVerificationPolicy = officialVerificationPolicyExists
    ? readJsonFile(join(root, officialVerificationPolicyPath))
    : { requiredVerifiedVendorRegions: [] };
  const sourcesSchema = readJsonFile(join(root, "schemas/vendor-sources.schema.json"));
  const officialSnapshotsSchema = readJsonFile(join(root, "schemas/official-model-snapshot.schema.json"));
  const officialVerificationPolicySchema = readJsonFile(join(root, "schemas/official-verification-policy.schema.json"));
  const catalog = loadCatalog(root);
  const vendorRegionKey = (vendorCode, regionCode) => `${vendorCode}/${regionCode}`;
  const sourceKey = (vendor) => vendorRegionKey(vendor.vendorCode, vendor.regionCode ?? "global");
  const catalogVendors = new Map(catalog.vendors.map((vendor) => [vendorRegionKey(vendor.vendorCode, vendor.regionCode), vendor]));
  const errors = [];
  const warnings = [];
  const vendorReports = [];

  function addError(code, path, message) {
    errors.push({ code, path, message });
  }

  function addWarning(code, path, message) {
    warnings.push({ code, path, message });
  }

  for (const schemaIssue of validateJsonSchema(sources, sourcesSchema, {
    codePrefix: "schema.vendor_sources",
    path: sourcesPath,
  })) {
    addError(schemaIssue.code, schemaIssue.path, schemaIssue.message);
  }
  if (existsSync(join(root, officialSnapshotsPath))) {
    for (const schemaIssue of validateJsonSchema(officialSnapshots, officialSnapshotsSchema, {
      codePrefix: "schema.official_model_snapshots",
      path: officialSnapshotsPath,
    })) {
      addError(schemaIssue.code, schemaIssue.path, schemaIssue.message);
    }
  }
  if (!officialVerificationPolicyExists) {
    addError(
      "official_verification_policy.missing",
      officialVerificationPolicyPath,
      `${officialVerificationPolicyPath} is required for release-gated official source verification`,
    );
  } else {
    for (const schemaIssue of validateJsonSchema(officialVerificationPolicy, officialVerificationPolicySchema, {
      codePrefix: "schema.official_verification_policy",
      path: officialVerificationPolicyPath,
    })) {
      addError(schemaIssue.code, schemaIssue.path, schemaIssue.message);
    }
  }

  if (sources.schemaVersion && sources.schemaVersion !== catalog.manifest.schemaVersion) {
    addError(
      "vendor_sources.schema_version.mismatch",
      sourcesPath,
      `${sourcesPath} schemaVersion ${sources.schemaVersion} must match catalog schemaVersion ${catalog.manifest.schemaVersion}`,
    );
  }
  if (sources.catalogVersion && sources.catalogVersion !== catalog.manifest.catalogVersion) {
    addError(
      "vendor_sources.catalog_version.mismatch",
      sourcesPath,
      `${sourcesPath} catalogVersion ${sources.catalogVersion} must match catalogVersion ${catalog.manifest.catalogVersion}`,
    );
  }

  const vendorSources = new Map();
  for (const [sourceIndex, vendorSource] of (sources.vendors ?? []).entries()) {
    const regionalKey = sourceKey(vendorSource);
    if (vendorSources.has(regionalKey)) {
      addError(
        "vendor.source.duplicate",
        `${sourcesPath}#/vendors/${sourceIndex}`,
        `${regionalKey} has more than one vendor source declaration`,
      );
      continue;
    }
    vendorSources.set(regionalKey, vendorSource);
  }

  function sourceAllowed(vendorSpec, sourceUrl) {
    if (!sourceUrl) {
      return false;
    }
    const allowed = [
      vendorSpec.official?.modelsUrl,
      vendorSpec.official?.pricingUrl,
      ...(vendorSpec.official?.additionalUrls ?? []),
      ...(vendorSpec.references ?? []).map((reference) => reference.url),
    ].filter(Boolean);
    return allowed.some((url) => sourceUrl === url || sourceUrl.startsWith(url));
  }

  function officialSourceAllowed(vendorSpec, sourceUrl) {
    if (!sourceUrl) {
      return false;
    }
    const allowed = [
      vendorSpec.official?.pricingUrl,
      ...(vendorSpec.official?.additionalUrls ?? []),
    ].filter(Boolean);
    return allowed.some((url) => sourceUrl === url || sourceUrl.startsWith(url));
  }

  function urlMatchesDeclaredBoundary(declaredUrl, sourceUrl) {
    if (!declaredUrl || !sourceUrl) {
      return false;
    }
    const declared = String(declaredUrl).trim();
    const source = String(sourceUrl).trim();
    if (!declared || !source) {
      return false;
    }
    if (source === declared) {
      return true;
    }
    const boundary = declared.endsWith("/") ? declared : `${declared}/`;
    return source.startsWith(boundary) || source.startsWith(`${declared}?`) || source.startsWith(`${declared}#`);
  }

  function declaredOfficialSourceAllowed(vendorSpec, sourceUrl) {
    const allowed = [
      vendorSpec.official?.modelsUrl,
      vendorSpec.official?.pricingUrl,
      ...(vendorSpec.official?.additionalUrls ?? []),
    ].filter(Boolean);
    return allowed.some((url) => urlMatchesDeclaredBoundary(url, sourceUrl));
  }

  if (officialSnapshots.schemaVersion && officialSnapshots.schemaVersion !== catalog.manifest.schemaVersion) {
    addError(
      "official_snapshot.schema_version.mismatch",
      officialSnapshotsPath,
      `${officialSnapshotsPath} schemaVersion ${officialSnapshots.schemaVersion} must match catalog schemaVersion ${catalog.manifest.schemaVersion}`,
    );
  }
  if (officialSnapshots.catalogVersion && officialSnapshots.catalogVersion !== catalog.manifest.catalogVersion) {
    addError(
      "official_snapshot.catalog_version.mismatch",
      officialSnapshotsPath,
      `${officialSnapshotsPath} catalogVersion ${officialSnapshots.catalogVersion} must match catalogVersion ${catalog.manifest.catalogVersion}`,
    );
  }
  if (officialVerificationPolicy.schemaVersion && officialVerificationPolicy.schemaVersion !== catalog.manifest.schemaVersion) {
    addError(
      "official_verification_policy.schema_version.mismatch",
      officialVerificationPolicyPath,
      `${officialVerificationPolicyPath} schemaVersion ${officialVerificationPolicy.schemaVersion} must match catalog schemaVersion ${catalog.manifest.schemaVersion}`,
    );
  }
  if (officialVerificationPolicy.catalogVersion && officialVerificationPolicy.catalogVersion !== catalog.manifest.catalogVersion) {
    addError(
      "official_verification_policy.catalog_version.mismatch",
      officialVerificationPolicyPath,
      `${officialVerificationPolicyPath} catalogVersion ${officialVerificationPolicy.catalogVersion} must match catalogVersion ${catalog.manifest.catalogVersion}`,
    );
  }

  const officialSnapshotVendors = new Map();
  for (const [vendorIndex, snapshot] of (officialSnapshots.vendors ?? []).entries()) {
    const regionalKey = sourceKey(snapshot);
    const snapshotPath = `${officialSnapshotsPath}#/vendors/${vendorIndex}`;
    if (officialSnapshotVendors.has(regionalKey)) {
      addError(
        "official_snapshot.vendor.duplicate",
        snapshotPath,
        `${regionalKey} has more than one official snapshot entry`,
      );
      continue;
    }
    officialSnapshotVendors.set(regionalKey, snapshot);

    const expectedSnapshotHash = officialSnapshotHash(snapshot);
    if (snapshot.sourceSnapshotHash !== expectedSnapshotHash) {
      addError(
        "official_snapshot.hash.mismatch",
        `${snapshotPath}/sourceSnapshotHash`,
        `${regionalKey} sourceSnapshotHash must equal the canonical SHA-256 of the snapshot without sourceSnapshotHash`,
      );
    }

    const spec = vendorSources.get(regionalKey);
    const vendorBundle = catalogVendors.get(regionalKey);
    if (!spec) {
      addError(
        "official_snapshot.vendor_source.missing",
        snapshotPath,
        `${regionalKey} official snapshot has no matching vendor source declaration`,
      );
    }
    if (!vendorBundle) {
      addError(
        "official_snapshot.vendor_catalog.missing",
        snapshotPath,
        `${regionalKey} official snapshot has no matching catalog vendor region`,
      );
    }

    for (const [urlIndex, officialUrl] of (snapshot.officialUrls ?? []).entries()) {
      if (spec && !declaredOfficialSourceAllowed(spec, officialUrl)) {
        addError(
          "official_snapshot.url.unapproved",
          `${snapshotPath}/officialUrls/${urlIndex}`,
          `${regionalKey} official snapshot URL must be declared under official modelsUrl, pricingUrl, or additionalUrls`,
        );
      }
    }

    const seenModelIds = new Set();
    const catalogModelIds = new Set((vendorBundle?.models ?? []).map((model) => model.modelId));
    for (const [modelIndex, model] of (snapshot.models ?? []).entries()) {
      const modelId = model.modelId;
      const modelPath = `${snapshotPath}/models/${modelIndex}`;
      if (seenModelIds.has(modelId)) {
        addError(
          "official_snapshot.model.duplicate",
          modelPath,
          `${regionalKey} official snapshot repeats modelId ${modelId}`,
        );
      }
      seenModelIds.add(modelId);
      if (vendorBundle && !catalogModelIds.has(modelId)) {
        addError(
          "official_snapshot.model.unknown",
          modelPath,
          `${regionalKey} official snapshot references ${modelId}, but no matching model file exists`,
        );
      }
    }
  }

  const policyVendorRegions = new Set();
  for (const [policyIndex, requiredVendor] of (officialVerificationPolicy.requiredVerifiedVendorRegions ?? []).entries()) {
    const regionalKey = vendorRegionKey(requiredVendor.vendorCode, requiredVendor.regionCode ?? "global");
    const policyPath = `${officialVerificationPolicyPath}#/requiredVerifiedVendorRegions/${policyIndex}`;
    if (policyVendorRegions.has(regionalKey)) {
      addError(
        "official_verification.policy.duplicate",
        policyPath,
        `${regionalKey} is declared more than once in the official verification policy`,
      );
      continue;
    }
    policyVendorRegions.add(regionalKey);
  }

  for (const vendor of catalog.vendors) {
    const regionalKey = vendorRegionKey(vendor.vendorCode, vendor.regionCode);
    const spec = vendorSources.get(regionalKey);
    const pathPrefix = `models/${vendor.vendorCode}/${vendor.regionCode}`;
    if (!spec) {
      addError("vendor.source.missing", pathPrefix, `${regionalKey} is missing from ${sourcesPath}`);
      continue;
    }
    if (!spec.official?.modelsUrl) {
      addError("vendor.official.models_url.missing", `${sourcesPath}#/${regionalKey}`, "official modelsUrl is required");
    }
    if (!spec.official?.pricingUrl) {
      addError("vendor.official.pricing_url.missing", `${sourcesPath}#/${regionalKey}`, "official pricingUrl is required");
    }
    const requiredModels = spec.requiredModels ?? [];
    const supportedModels = spec.supportedModels ?? [];
    if (!Array.isArray(requiredModels) || !Array.isArray(supportedModels) || (requiredModels.length === 0 && supportedModels.length === 0)) {
      addError(
        "vendor.model_scope.empty",
        `${sourcesPath}#/${regionalKey}`,
        "at least one requiredModels or supportedModels entry is required",
      );
    }

    const modelIds = new Set(vendor.models.map((model) => model.modelId));
    const pricingIds = new Set(vendor.pricing.map((pricing) => pricing.modelId));
    const defaultModelIds = new Set(
      (vendor.families.families ?? []).map((family) => family.defaultModel).filter(Boolean),
    );
    const enabledModelIds = new Set(
      vendor.models
        .filter((model) => model.routingState === "enabled" && model.releaseStage !== "retired")
        .map((model) => model.modelId),
    );
    const snapshot = officialSnapshotVendors.get(regionalKey);
    const snapshotModelIds = new Set((snapshot?.models ?? []).map((model) => model.modelId));
    const mustHaveOfficialSnapshotIds = new Set([
      ...requiredModels,
      ...enabledModelIds,
      ...defaultModelIds,
    ]);
    const missingOfficialSnapshotModels = [...mustHaveOfficialSnapshotIds]
      .filter((modelId) => !snapshotModelIds.has(modelId))
      .sort();

    if (spec.verificationStatus === "official_verified") {
      if (!policyVendorRegions.has(regionalKey)) {
        addError(
          "official_verification.policy.missing",
          `${officialVerificationPolicyPath}#/requiredVerifiedVendorRegions`,
          `${regionalKey} is official_verified but is not covered by the official verification policy release gate`,
        );
      }
      if (!snapshot) {
        addError(
          "vendor.official_snapshot.missing",
          officialSnapshotsPath,
          `${regionalKey} is official_verified but has no independent official snapshot`,
        );
      }
      for (const modelId of missingOfficialSnapshotModels) {
        addError(
          "vendor.official_snapshot_model.missing",
          `${officialSnapshotsPath}#/${regionalKey}/models/${modelId}`,
          `${modelId} is required/enabled/default for ${regionalKey} but is missing from the official snapshot`,
        );
      }
    }

    for (const modelId of requiredModels) {
      if (!modelIds.has(modelId)) {
        addError("vendor.required_model.missing", `${pathPrefix}/models/${modelFileName(modelId)}`, `${modelId} is required by ${sourcesPath}`);
      }
      if (!pricingIds.has(modelId)) {
        addError("vendor.required_pricing.missing", `${pathPrefix}/pricing/${modelFileName(modelId)}`, `${modelId} pricing is required by ${sourcesPath}`);
      }
    }

    for (const modelId of supportedModels) {
      if (!modelIds.has(modelId)) {
        addError("vendor.supported_model.missing", `${pathPrefix}/models/${modelFileName(modelId)}`, `${modelId} is supported by ${sourcesPath}`);
      }
    }

    for (const model of vendor.models) {
      const modelPath = `${pathPrefix}/models/${modelFileName(model.modelId)}`;
      if (model.routingState === "enabled" && model.releaseStage !== "retired") {
        if (!sourceAllowed(spec, model.source?.sourceUrl)) {
          addError("model.source.unapproved", modelPath, `${model.modelId} sourceUrl must be declared in ${sourcesPath}`);
        }
      }
      if (model.modelId.includes("preview") && model.releaseStage === "active") {
        addWarning("model.preview.active", modelPath, `${model.modelId} contains preview but is marked active`);
      }
    }

    for (const pricing of vendor.pricing) {
      for (const [index, price] of (pricing.prices ?? []).entries()) {
        const pricePath = `${pathPrefix}/pricing/${modelFileName(pricing.modelId)}#/prices/${index}`;
        if (!sourceAllowed(spec, price.source?.sourceUrl)) {
          addError("pricing.source.unapproved", pricePath, `${pricing.modelId} price sourceUrl must be declared in ${sourcesPath}`);
        }
        if (price.priceSide === "official" && !officialSourceAllowed(spec, price.source?.sourceUrl)) {
          addWarning("pricing.official.indirect", pricePath, `${pricing.modelId} official price uses an indirect source URL`);
        }
      }
    }

    vendorReports.push({
      vendorCode: vendor.vendorCode,
      regionCode: vendor.regionCode,
      vendorRegion: regionalKey,
      verificationStatus: spec.verificationStatus ?? "unknown",
      modelCount: vendor.models.length,
      pricingFileCount: vendor.pricing.length,
      requiredModelCount: spec.requiredModels?.length ?? 0,
      officialSnapshotModelCount: snapshotModelIds.size,
      missingOfficialSnapshotModels,
      officialModelsUrl: spec.official?.modelsUrl,
      officialPricingUrl: spec.official?.pricingUrl,
    });
  }

  for (const [policyIndex, requiredVendor] of (officialVerificationPolicy.requiredVerifiedVendorRegions ?? []).entries()) {
    const regionalKey = vendorRegionKey(requiredVendor.vendorCode, requiredVendor.regionCode ?? "global");
    const policyPath = `${officialVerificationPolicyPath}#/requiredVerifiedVendorRegions/${policyIndex}`;
    const isDuplicatePolicyEntry =
      (officialVerificationPolicy.requiredVerifiedVendorRegions ?? [])
        .findIndex((vendor) => vendorRegionKey(vendor.vendorCode, vendor.regionCode ?? "global") === regionalKey) !== policyIndex;
    if (isDuplicatePolicyEntry) {
      continue;
    }
    const spec = vendorSources.get(regionalKey);
    const snapshot = officialSnapshotVendors.get(regionalKey);
    if (!catalogVendors.has(regionalKey)) {
      addError(
        "official_verification.vendor_catalog.missing",
        policyPath,
        `${regionalKey} is required by official verification policy but has no matching catalog vendor region`,
      );
      continue;
    }
    if (!spec) {
      addError(
        "official_verification.vendor_source.missing",
        policyPath,
        `${regionalKey} is required by official verification policy but has no matching source declaration`,
      );
      continue;
    }
    if (spec.verificationStatus !== "official_verified") {
      addError(
        "official_verification.required_status",
        policyPath,
        `${regionalKey} is required by official verification policy and must be official_verified`,
      );
    }
    if (!snapshot) {
      addError(
        "official_verification.required_snapshot",
        policyPath,
        `${regionalKey} is required by official verification policy and must have an independent official snapshot`,
      );
    }
  }

  for (const regionalKey of vendorSources.keys()) {
    const [vendorCode, regionCode] = regionalKey.split("/");
    const vendorDir = join(root, "models", vendorCode, regionCode);
    if (!existsSync(vendorDir)) {
      addError("vendor.directory.missing", `models/${regionalKey}`, `${regionalKey} source exists but regional vendor directory is missing`);
    }
  }

  const officialVerifiedSourceRegions = [...vendorSources.values()]
    .filter((vendor) => vendor.verificationStatus === "official_verified")
    .map((vendor) => sourceKey(vendor))
    .sort();
  const requiredVerifiedRegions = [...policyVendorRegions].sort();

  return {
    ok: errors.length === 0,
    asOf,
    generatedAt: catalog.manifest.generatedAt,
    catalogVersion: catalog.manifest.catalogVersion,
    vendorCount: new Set(catalog.vendors.map((vendor) => vendor.vendorCode)).size,
    regionCount: catalog.vendors.length,
    requiredVerifiedRegionCount: requiredVerifiedRegions.length,
    requiredVerifiedRegions,
    officialVerifiedSourceRegionCount: officialVerifiedSourceRegions.length,
    officialVerifiedSourceRegions,
    errors,
    warnings,
    vendors: vendorReports,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const args = process.argv.slice(2);
  const asOfArg = args.indexOf("--as-of");
  const sourcesArg = args.indexOf("--sources");
  const officialVerificationPolicyArg = args.indexOf("--official-verification-policy");
  const report = auditCatalog(root, {
    asOf: asOfArg >= 0 ? args[asOfArg + 1] : undefined,
    sourcesPath: sourcesArg >= 0 ? args[sourcesArg + 1] : undefined,
    officialVerificationPolicyPath: officialVerificationPolicyArg >= 0 ? args[officialVerificationPolicyArg + 1] : undefined,
  });
  console.log(JSON.stringify(report, null, 2));
  if (!report.ok) {
    process.exit(1);
  }
}
