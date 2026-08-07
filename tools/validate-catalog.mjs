#!/usr/bin/env node
import { existsSync, readdirSync, statSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { join } from "node:path";
import {
  buildCatalogIndex,
  buildVendorList,
  catalogKey,
  collectRegionalCatalogDirectories,
  isDecimalString,
  issue,
  loadManifest,
  loadMeters,
  loadVendorBundle,
  modelIdPath,
  projectRootFromTool,
  readJsonFile,
  stableJson,
  videoProfileCatalogKey,
} from "./catalog-lib.mjs";

const GENERATION_MODE_INPUTS = {
  text_to_video: ["text"],
  image_to_video: ["image"],
  reference_to_video: ["video", "image"],
  start_end_frame: ["image"],
  video_extension: ["video"],
  video_edit: ["video"],
  multi_shot: ["text"],
};

const DURATION_TIER_CODES = new Set([
  "dur_5s",
  "dur_6s",
  "dur_8s",
  "dur_10s",
  "dur_15s",
  "dur_30s",
  "dur_60s",
]);

const USAGE_SCOPES = new Set(["coding", "chat", "agent"]);

export function validateCatalog(root) {
  const issues = [];

  function requireFile(rel) {
    const path = join(root, rel);
    if (!existsSync(path) || !statSync(path).isFile()) {
      issues.push(issue("file.missing", rel, `${rel} is required`));
    }
  }

  for (const rel of [
    "sdkwork-models.json",
    "models/meters.json",
    "models/protocols.json",
    "models/vendors.json",
    "models/index.json",
    "schemas/catalog.schema.json",
    "schemas/index.schema.json",
    "schemas/model.schema.json",
    "schemas/pricing.schema.json",
    "schemas/protocol.schema.json",
    "schemas/voice.schema.json",
    "schemas/model-voice.schema.json",
    "schemas/model-video-profiles.schema.json",
  ]) {
    requireFile(rel);
  }

  const manifest = loadManifest(root);
  const modelsRoot = join(root, manifest.modelsRoot);
  const meterCodes = new Set(loadMeters(root).map((meter) => meter.meterCode));
  const protocolFile = readJsonFile(join(root, manifest.modelsRoot, "protocols.json"));
  const protocolCodes = new Set();
  const protocolFamilyByCode = new Map();
  for (const [index, protocol] of (protocolFile.protocols ?? []).entries()) {
    if (typeof protocol.protocolCode !== "string" || protocol.protocolCode.length === 0) {
      issues.push(issue("protocol.code.invalid", `models/protocols.json#/protocols/${index}/protocolCode`, "protocolCode must be a non-empty string"));
      continue;
    }
    if (protocolCodes.has(protocol.protocolCode)) {
      issues.push(issue("protocol.code.duplicate", `models/protocols.json#/protocols/${index}/protocolCode`, `${protocol.protocolCode} is duplicated`));
    }
    protocolCodes.add(protocol.protocolCode);
    if (typeof protocol.family === "string") {
      protocolFamilyByCode.set(protocol.protocolCode, protocol.family);
    }
  }
  // 兼容族标准路径规律:familyCode -> 允许的 pathPrefix 集合(models/protocols.json#/families)
  const familyPathPrefixes = new Map();
  for (const family of (protocolFile.families ?? [])) {
    if (typeof family.familyCode !== "string") {
      continue;
    }
    familyPathPrefixes.set(family.familyCode, new Set(family.standardPathPrefixes ?? []));
  }
  const routingResourceIndex = loadRoutingResourceIndex(root);
  const seenVendorRegions = new Set();
  const seenModels = new Map();

  for (const regionDir of collectRegionalCatalogDirectories(modelsRoot)) {
    const bundle = loadVendorBundle(regionDir);
    const pathPrefix = `models/${bundle.vendorCode}/${bundle.regionCode}`;
    if (bundle.vendor.vendorCode !== bundle.vendorCode) {
      issues.push(
        issue(
          "vendor.directory.mismatch",
          `${pathPrefix}/vendor.json#/vendorCode`,
          `vendorCode ${bundle.vendor.vendorCode} must match directory ${bundle.vendorCode}`,
        ),
      );
    }
    if (bundle.vendor.regionCode !== bundle.regionCode) {
      issues.push(
        issue(
          "vendor.region_directory.mismatch",
          `${pathPrefix}/vendor.json#/regionCode`,
          `regionCode ${bundle.vendor.regionCode} must match directory ${bundle.regionCode}`,
        ),
      );
    }
    if (bundle.vendor.vendorCode && /_(cn|global)$/.test(bundle.vendor.vendorCode)) {
      issues.push(issue("vendor.code.region_suffix", `${pathPrefix}/vendor.json#/vendorCode`, "vendorCode must be the unique vendor identity; put operating context in regionCode"));
    }
    const vendorRegionKey = `${bundle.vendor.vendorCode}/${bundle.vendor.regionCode}`;
    if (seenVendorRegions.has(vendorRegionKey)) {
      issues.push(issue("vendor_region.duplicate", pathPrefix, `${vendorRegionKey} is duplicated`));
    }
    seenVendorRegions.add(vendorRegionKey);
    for (const field of ["regionCode", "marketScope", "billingCurrency", "billingJurisdiction", "operatingRegions"]) {
      const value = bundle.vendor[field];
      if (value === undefined || value === null || (Array.isArray(value) && value.length === 0)) {
        issues.push(issue("vendor.operating_context.missing", `${pathPrefix}/vendor.json#/${field}`, `${field} is required for vendor operating and billing context`));
      }
    }
    const supportedProtocols = new Set();
    if (!Array.isArray(bundle.vendor.supportedProtocols) || bundle.vendor.supportedProtocols.length === 0) {
      issues.push(issue("vendor.protocols.missing", `${pathPrefix}/vendor.json#/supportedProtocols`, "supportedProtocols must declare at least one protocolCode"));
    }
    for (const [index, protocolCode] of (bundle.vendor.supportedProtocols ?? []).entries()) {
      if (!protocolCodes.has(protocolCode)) {
        issues.push(issue("vendor.protocol.unknown", `${pathPrefix}/vendor.json#/supportedProtocols/${index}`, `${protocolCode} is not defined in models/protocols.json`));
      }
      supportedProtocols.add(protocolCode);
    }
    validateApiEndpoints({
      vendor: bundle.vendor,
      protocolFamilyByCode,
      familyPathPrefixes,
      pathPrefix,
      issues,
    });
    validateClientApiCompatibility({
      vendor: bundle.vendor,
      protocolCodes,
      routingResourceIndex,
      pathPrefix,
      issues,
    });
    if (/\b(qwen|kling|hunyuan|bigmodel|seedance)\b/i.test(bundle.vendor.vendorCode.replace(/_/g, " "))) {
      issues.push(issue("vendor.code.product_line", `${pathPrefix}/vendor.json#/vendorCode`, "vendorCode must identify the unique vendor identity, not a product line"));
    }

    const familyCodes = new Set((bundle.families.families ?? []).map((family) => family.familyCode));
    if (bundle.families.vendorCode !== bundle.vendorCode) {
      issues.push(issue("family.vendor.mismatch", `${pathPrefix}/families.json`, "families vendorCode must match directory"));
    }
    if (bundle.families.regionCode !== bundle.regionCode) {
      issues.push(issue("family.region.mismatch", `${pathPrefix}/families.json`, "families regionCode must match directory"));
    }

    const modelsById = new Map(bundle.models.map((model) => [model.modelId, model]));
    const modelFilesById = new Map();
    for (const relPath of collectJsonFileRefsForDirectory(modelsRoot, join(regionDir, "models"))) {
      const model = readJsonFile(join(modelsRoot, relPath));
      if (typeof model.modelId === "string") {
        modelFilesById.set(model.modelId, `models/${relPath}`);
      }
    }
    const pricingFilesById = new Map();
    for (const relPath of collectJsonFileRefsForDirectory(modelsRoot, join(regionDir, "pricing"))) {
      const pricing = readJsonFile(join(modelsRoot, relPath));
      if (typeof pricing.modelId === "string") {
        pricingFilesById.set(pricing.modelId, `models/${relPath}`);
      }
    }
    const pricedModelIds = new Set(bundle.pricing.map((pricing) => pricing.modelId));
    for (const [index, family] of (bundle.families.families ?? []).entries()) {
      if (!family.defaultModel) {
        continue;
      }
      const defaultModel = modelsById.get(family.defaultModel);
      if (!defaultModel) {
        issues.push(issue("family.default_model.missing", `${pathPrefix}/families.json#/families/${index}/defaultModel`, `${family.defaultModel} is not defined for ${bundle.vendorCode}/${bundle.regionCode}`));
        continue;
      }
      if (defaultModel.routingState !== "enabled" || defaultModel.shelfState !== "listed") {
        issues.push(issue("family.default_model.not_routable", `${pathPrefix}/families.json#/families/${index}/defaultModel`, `${family.defaultModel} must be enabled and listed to be a family default`));
      }
      if (!pricedModelIds.has(family.defaultModel)) {
        issues.push(issue("family.default_model.pricing_missing", `${pathPrefix}/families.json#/families/${index}/defaultModel`, `${family.defaultModel} must have pricing to be a family default`));
      }
    }

    for (const model of bundle.models) {
      const modelPath = `${pathPrefix}/models/${safeModelIdPath(model.modelId, issues, `${pathPrefix}/models`) }.json`;
      const actualModelPath = modelFilesById.get(model.modelId);
      if (actualModelPath && actualModelPath !== modelPath) {
        issues.push(issue("model.path.mismatch", actualModelPath, `${model.modelId} must be stored at ${modelPath}`));
      }
      if (model.vendorCode !== bundle.vendorCode) {
        issues.push(issue("model.vendor.mismatch", `${modelPath}#/vendorCode`, "model vendorCode must match directory"));
      }
      if (model.regionCode !== bundle.regionCode) {
        issues.push(issue("model.region.mismatch", `${modelPath}#/regionCode`, "model regionCode must match directory"));
      }
      const expectedCatalogKey = catalogKey(bundle.vendorCode, model.modelId);
      if (model.catalogKey !== expectedCatalogKey) {
        issues.push(issue("model.catalog_key.mismatch", `${modelPath}#/catalogKey`, `catalogKey must be ${expectedCatalogKey}`));
      }
      if (!familyCodes.has(model.familyCode)) {
        issues.push(issue("model.family.missing", `${modelPath}#/familyCode`, `familyCode ${model.familyCode} is not defined`));
      }
      if (!model.source?.sourceUrl || !model.source?.observedAt) {
        issues.push(issue("model.source.missing", `${modelPath}#/source`, "model sourceUrl and observedAt are required"));
      }
      for (const capabilityField of ["supportsStreaming", "supportsTools", "supportsJsonSchema", "codingVisible"]) {
        if (typeof model[capabilityField] !== "boolean") {
          issues.push(
            issue(
              "model.capability_flag.missing",
              `${modelPath}#/${capabilityField}`,
              `${capabilityField} must be a boolean; run node tools/align-model-capabilities.mjs to backfill`,
            ),
          );
        }
      }
      if (!Array.isArray(model.usageScopes)) {
        issues.push(
          issue(
            "model.usage_scopes.missing",
            `${modelPath}#/usageScopes`,
            "usageScopes must be an array of product usage scopes; run node tools/align-model-capabilities.mjs to backfill",
          ),
        );
      } else {
        const unknownScopes = model.usageScopes.filter((scope) => !USAGE_SCOPES.has(scope));
        if (unknownScopes.length > 0) {
          issues.push(
            issue(
              "model.usage_scopes.unknown",
              `${modelPath}#/usageScopes`,
              `unknown usage scope(s): ${unknownScopes.join(", ")}; allowed: ${[...USAGE_SCOPES].sort().join(", ")}`,
            ),
          );
        }
      }
      if (!Array.isArray(model.inputModalities) || model.inputModalities.length === 0) {
        issues.push(issue("model.input_modalities.missing", `${modelPath}#/inputModalities`, "inputModalities must declare at least one modality"));
      }
      if (!Array.isArray(model.outputModalities) || model.outputModalities.length === 0) {
        issues.push(issue("model.output_modalities.missing", `${modelPath}#/outputModalities`, "outputModalities must declare at least one modality"));
      }
      if (!protocolCodes.has(model.apiFormat)) {
        issues.push(issue("model.protocol.unknown", `${modelPath}#/apiFormat`, `${model.apiFormat} is not defined in models/protocols.json`));
      } else if (!supportedProtocols.has(model.apiFormat)) {
        issues.push(issue("model.protocol.unsupported_by_vendor", `${modelPath}#/apiFormat`, `${bundle.vendorCode}/${bundle.regionCode} must include ${model.apiFormat} in supportedProtocols`));
      }
      const modelCatalogKey = expectedCatalogKey;
      if (seenModels.has(modelCatalogKey)) {
        const previous = seenModels.get(modelCatalogKey);
        const differences = modelIdentityDifferences(previous.model, model);
        if (differences.length > 0) {
          issues.push(
            issue(
              "model.identity_conflict",
              modelPath,
              `${modelCatalogKey} differs from ${previous.path} on ${differences.join(", ")}; region-specific data belongs in pricing, ranking, and provider endpoint resources`,
            ),
          );
        }
        continue;
      }
      seenModels.set(modelCatalogKey, { path: modelPath, model });
    }

    const vendorModelIds = new Set(bundle.models.map((model) => model.modelId));
    for (const model of bundle.models) {
      if (
        (model.routingState === "enabled" || model.shelfState === "listed" || model.releaseStage === "active") &&
        !pricedModelIds.has(model.modelId)
      ) {
        issues.push(
          issue(
            "model.pricing.required",
            `${pathPrefix}/models/${safeModelIdPath(model.modelId, issues, `${pathPrefix}/models`) }.json`,
            `${model.catalogKey} is enabled, listed, or active and must have a pricing file with billable rows`,
          ),
        );
      }
    }
    for (const pricing of bundle.pricing) {
      const pricingPath = `${pathPrefix}/pricing/${safeModelIdPath(pricing.modelId, issues, `${pathPrefix}/pricing`) }.json`;
      const actualPricingPath = pricingFilesById.get(pricing.modelId);
      if (actualPricingPath && actualPricingPath !== pricingPath) {
        issues.push(issue("pricing.path.mismatch", actualPricingPath, `${pricing.modelId} must be stored at ${pricingPath}`));
      }
      if (pricing.vendorCode !== bundle.vendorCode) {
        issues.push(issue("pricing.vendor.mismatch", `${pricingPath}#/vendorCode`, "pricing vendorCode must match directory"));
      }
      if (pricing.regionCode !== bundle.regionCode) {
        issues.push(issue("pricing.region.mismatch", `${pricingPath}#/regionCode`, "pricing regionCode must match directory"));
      }
      const expectedCatalogKey = catalogKey(bundle.vendorCode, pricing.modelId);
      if (pricing.catalogKey !== expectedCatalogKey) {
        issues.push(issue("pricing.catalog_key.mismatch", `${pricingPath}#/catalogKey`, `catalogKey must be ${expectedCatalogKey}`));
      }
      if (bundle.vendor.billingCurrency && pricing.currency !== bundle.vendor.billingCurrency) {
        issues.push(issue("pricing.currency.vendor_mismatch", `${pricingPath}#/currency`, `${pricing.modelId} currency must match vendor billingCurrency ${bundle.vendor.billingCurrency}`));
      }
      if (!vendorModelIds.has(pricing.modelId)) {
        issues.push(issue("pricing.model.missing", `${pricingPath}#/modelId`, `${pricing.modelId} is not defined for ${bundle.vendorCode}`));
      }
      const seenPriceIds = new Set();
      const seenPriceKeys = new Set();
      if (!Array.isArray(pricing.prices) || pricing.prices.length === 0) {
        issues.push(issue("pricing.prices.empty", `${pricingPath}#/prices`, `${pricing.modelId} pricing must contain at least one billable row`));
      }
      for (const [index, price] of (pricing.prices ?? []).entries()) {
        if (seenPriceIds.has(price.priceId)) {
          issues.push(issue("price.id.duplicate", `${pricingPath}#/prices/${index}/priceId`, `${price.priceId} is duplicated in ${pricing.modelId}`));
        }
        seenPriceIds.add(price.priceId);

        const priceKey = [
          price.priceSide,
          price.pricingScope ?? "model",
          price.meterCode,
          price.mediaType ?? "",
          price.mediaDirection ?? "",
          price.inputType ?? "",
          price.outputType ?? "",
          price.tierCode ?? "",
          price.currency ?? pricing.currency,
          price.minimumQuantity ?? "0",
          price.effectiveFrom,
        ].join("|");
        if (seenPriceKeys.has(priceKey)) {
          issues.push(issue("price.key.duplicate", `${pricingPath}#/prices/${index}`, `${pricing.modelId} has duplicate effective pricing key ${priceKey}`));
        }
        seenPriceKeys.add(priceKey);

        for (const field of ["unitSize", "unitPrice", "minimumQuantity"]) {
          if (!isDecimalString(price[field])) {
            issues.push(issue("price.decimal.invalid", `${pricingPath}#/prices/${index}/${field}`, `${field} must be a decimal string`));
          }
        }
        if (bundle.vendor.billingCurrency && (price.currency ?? pricing.currency) !== bundle.vendor.billingCurrency) {
          issues.push(issue("price.currency.vendor_mismatch", `${pricingPath}#/prices/${index}/currency`, `${price.priceId} currency must match vendor billingCurrency ${bundle.vendor.billingCurrency}`));
        }
        if (!meterCodes.has(price.meterCode)) {
          issues.push(issue("price.meter.missing", `${pricingPath}#/prices/${index}/meterCode`, `meterCode ${price.meterCode} is not defined`));
        }
        if (!price.source?.sourceUrl || !price.source?.observedAt || !price.effectiveFrom) {
          issues.push(issue("price.source.missing", `${pricingPath}#/prices/${index}`, "sourceUrl, observedAt, and effectiveFrom are required"));
        }
      }
    }

    for (const snapshot of bundle.rankings.snapshots ?? []) {
      for (const [index, item] of (snapshot.items ?? []).entries()) {
        if (!vendorModelIds.has(item.modelId)) {
          issues.push(issue("ranking.model.missing", `${pathPrefix}/rankings.json#/snapshots/${snapshot.snapshotDate}/items/${index}/modelId`, `${item.modelId} is not defined for ${bundle.vendorCode}`));
        }
      }
    }

    const voiceByKey = new Map();
    for (const [index, voice] of (bundle.voices ?? []).entries()) {
      const voicePath = `${pathPrefix}/voices.json#/voices/${index}`;
      const expectedVoiceKey = catalogKey(bundle.vendorCode, voice.voiceId);
      if (voice.catalogKey !== expectedVoiceKey) {
        issues.push(issue("voice.catalog_key.mismatch", `${voicePath}/catalogKey`, `catalogKey must be ${expectedVoiceKey}`));
      }
      if (voice.vendorCode !== bundle.vendorCode) {
        issues.push(issue("voice.vendor.mismatch", `${voicePath}/vendorCode`, "voice vendorCode must match directory"));
      }
      if (voice.regionCode !== bundle.regionCode) {
        issues.push(issue("voice.region.mismatch", `${voicePath}/regionCode`, "voice regionCode must match directory"));
      }
      if (!voice.source?.sourceUrl || !voice.source?.observedAt) {
        issues.push(issue("voice.source.missing", `${voicePath}/source`, "voice sourceUrl and observedAt are required"));
      }
      if (voiceByKey.has(voice.catalogKey)) {
        issues.push(issue("voice.catalog_key.duplicate", `${voicePath}/catalogKey`, `${voice.catalogKey} is duplicated in voices.json`));
      }
      voiceByKey.set(voice.catalogKey, voice);
    }

    for (const bindingFile of bundle.modelVoices ?? []) {
      const bindingPath = `${pathPrefix}/model-voices/${safeModelIdPath(bindingFile.modelId, issues, `${pathPrefix}/model-voices`) }.json`;
      if (bindingFile.vendorCode !== bundle.vendorCode) {
        issues.push(issue("model_voice.vendor.mismatch", `${bindingPath}#/vendorCode`, "model voice vendorCode must match directory"));
      }
      if (bindingFile.regionCode !== bundle.regionCode) {
        issues.push(issue("model_voice.region.mismatch", `${bindingPath}#/regionCode`, "model voice regionCode must match directory"));
      }
      const expectedModelKey = catalogKey(bundle.vendorCode, bindingFile.modelId);
      if (bindingFile.catalogKey !== expectedModelKey) {
        issues.push(issue("model_voice.catalog_key.mismatch", `${bindingPath}#/catalogKey`, `catalogKey must be ${expectedModelKey}`));
      }
      if (!vendorModelIds.has(bindingFile.modelId)) {
        issues.push(issue("model_voice.model.missing", `${bindingPath}#/modelId`, `${bindingFile.modelId} is not defined for ${bundle.vendorCode}`));
      }
      if (!bindingFile.source?.sourceUrl || !bindingFile.source?.observedAt) {
        issues.push(issue("model_voice.source.missing", `${bindingPath}#/source`, "sourceUrl and observedAt are required"));
      }
      let defaultCount = 0;
      for (const [index, binding] of (bindingFile.bindings ?? []).entries()) {
        if (!voiceByKey.has(binding.voiceKey)) {
          issues.push(issue("model_voice.binding.voice_missing", `${bindingPath}#/bindings/${index}/voiceKey`, `${binding.voiceKey} is not defined in ${pathPrefix}/voices.json`));
        }
        if (binding.voiceKey !== catalogKey(bundle.vendorCode, binding.voiceId)) {
          issues.push(issue("model_voice.binding.voice_key.mismatch", `${bindingPath}#/bindings/${index}/voiceKey`, `voiceKey must be ${catalogKey(bundle.vendorCode, binding.voiceId)}`));
        }
        if (binding.isDefault) {
          defaultCount += 1;
        }
      }
      if (defaultCount > 1) {
        issues.push(issue("model_voice.binding.default.duplicate", `${bindingPath}#/bindings`, "at most one binding may set isDefault"));
      }
    }

    const modelById = new Map(bundle.models.map((model) => [model.modelId, model]));
    const pricingTierCodesByModel = new Map();
    for (const pricing of bundle.pricing ?? []) {
      const tiers = new Set(
        (pricing.prices ?? [])
          .map((price) => price.tierCode)
          .filter((tierCode) => typeof tierCode === "string" && tierCode.length > 0),
      );
      pricingTierCodesByModel.set(pricing.modelId, tiers);
    }

    for (const profileFile of bundle.modelVideoProfiles ?? []) {
      const profilePath = `${pathPrefix}/model-video-profiles/${safeModelIdPath(profileFile.modelId, issues, `${pathPrefix}/model-video-profiles`)}.json`;
      if (profileFile.vendorCode !== bundle.vendorCode) {
        issues.push(issue("model_video_profile.vendor.mismatch", `${profilePath}#/vendorCode`, "model video profile vendorCode must match directory"));
      }
      if (profileFile.regionCode !== bundle.regionCode) {
        issues.push(issue("model_video_profile.region.mismatch", `${profilePath}#/regionCode`, "model video profile regionCode must match directory"));
      }
      const expectedModelKey = catalogKey(bundle.vendorCode, profileFile.modelId);
      if (profileFile.catalogKey !== expectedModelKey) {
        issues.push(issue("model_video_profile.catalog_key.mismatch", `${profilePath}#/catalogKey`, `catalogKey must be ${expectedModelKey}`));
      }
      if (!vendorModelIds.has(profileFile.modelId)) {
        issues.push(issue("model_video_profile.model.missing", `${profilePath}#/modelId`, `${profileFile.modelId} is not defined for ${bundle.vendorCode}`));
      }
      if (!profileFile.source?.sourceUrl || !profileFile.source?.observedAt) {
        issues.push(issue("model_video_profile.source.missing", `${profilePath}#/source`, "sourceUrl and observedAt are required"));
      }
      const model = modelById.get(profileFile.modelId);
      const profileCodes = new Set();
      let defaultCount = 0;
      for (const [index, profile] of (profileFile.profiles ?? []).entries()) {
        const itemPath = `${profilePath}#/profiles/${index}`;
        const expectedProfileKey = videoProfileCatalogKey(bundle.vendorCode, profileFile.modelId, profile.profileCode);
        if (profile.catalogKey !== expectedProfileKey) {
          issues.push(issue("model_video_profile.profile.catalog_key.mismatch", `${itemPath}/catalogKey`, `catalogKey must be ${expectedProfileKey}`));
        }
        if (profileCodes.has(profile.profileCode)) {
          issues.push(issue("model_video_profile.profile.duplicate", `${itemPath}/profileCode`, `${profile.profileCode} is duplicated`));
        }
        profileCodes.add(profile.profileCode);
        if (profile.isDefault) {
          defaultCount += 1;
        }
        if (model) {
          const requiredInputs = GENERATION_MODE_INPUTS[profile.generationMode] ?? [];
          if (
            requiredInputs.length > 0
            && !requiredInputs.some((modality) => (model.inputModalities ?? []).includes(modality))
          ) {
            issues.push(
              issue(
                "model_video_profile.generation_mode.input_mismatch",
                `${itemPath}/generationMode`,
                `${profile.generationMode} requires one of ${requiredInputs.join(", ")} input modalities`,
              ),
            );
          }
          if (model.primaryCapability !== "video" && !(model.capabilities ?? []).includes("video")) {
            issues.push(issue("model_video_profile.model.not_video", `${profilePath}#/modelId`, `${profileFile.modelId} is not a video model`));
          }
        }
        if (profile.durationPolicy === "fixed") {
          if (!Number.isInteger(profile.durationSeconds) || profile.durationSeconds <= 0) {
            issues.push(issue("model_video_profile.duration.fixed.missing", `${itemPath}/durationSeconds`, "durationSeconds is required for fixed durationPolicy"));
          }
          if (!DURATION_TIER_CODES.has(profile.durationTierCode)) {
            issues.push(issue("model_video_profile.duration.tier.invalid", `${itemPath}/durationTierCode`, "durationTierCode must use canonical dur_* vocabulary"));
          }
        } else if (profile.durationPolicy === "discrete") {
          if (!Array.isArray(profile.durationOptions) || profile.durationOptions.length === 0) {
            issues.push(issue("model_video_profile.duration.discrete.missing", `${itemPath}/durationOptions`, "durationOptions is required for discrete durationPolicy"));
          }
          if (!Array.isArray(profile.durationTierCodes) || profile.durationTierCodes.length !== profile.durationOptions?.length) {
            issues.push(issue("model_video_profile.duration.tier_codes.mismatch", `${itemPath}/durationTierCodes`, "durationTierCodes must align with durationOptions"));
          }
          for (const tierCode of profile.durationTierCodes ?? []) {
            if (!DURATION_TIER_CODES.has(tierCode)) {
              issues.push(issue("model_video_profile.duration.tier.invalid", `${itemPath}/durationTierCodes`, "durationTierCodes must use canonical dur_* vocabulary"));
            }
          }
        } else if (profile.durationPolicy === "range" || profile.durationPolicy === "continuous") {
          if (!Number.isInteger(profile.minDurationSeconds) || !Number.isInteger(profile.maxDurationSeconds)) {
            issues.push(issue("model_video_profile.duration.range.missing", `${itemPath}`, "minDurationSeconds and maxDurationSeconds are required for range/continuous durationPolicy"));
          } else if (profile.minDurationSeconds > profile.maxDurationSeconds) {
            issues.push(issue("model_video_profile.duration.range.invalid", `${itemPath}`, "minDurationSeconds must be <= maxDurationSeconds"));
          }
          if (!Number.isInteger(profile.durationStepSeconds) || profile.durationStepSeconds <= 0) {
            issues.push(issue("model_video_profile.duration.step.missing", `${itemPath}/durationStepSeconds`, "durationStepSeconds is required for range/continuous durationPolicy"));
          }
        }
        for (const tierCode of profile.pricingTierCodes ?? []) {
          const modelTiers = pricingTierCodesByModel.get(profileFile.modelId) ?? new Set();
          if (!modelTiers.has(tierCode)) {
            issues.push(issue("model_video_profile.pricing.tier.missing", `${itemPath}/pricingTierCodes`, `${tierCode} is not declared on ${expectedModelKey} pricing`));
          }
        }
      }
      if (defaultCount > 1) {
        issues.push(issue("model_video_profile.default.duplicate", `${profilePath}#/profiles`, "at most one profile may set isDefault"));
      }
    }

    const profileModelIds = new Set((bundle.modelVideoProfiles ?? []).map((file) => file.modelId));
    for (const model of bundle.models) {
      if (model.primaryCapability === "video" && !profileModelIds.has(model.modelId)) {
        issues.push(
          issue(
            "model_video_profile.model.required",
            `${pathPrefix}/model-video-profiles/${model.modelId}.json`,
            `${model.modelId} is a video model and requires a model-video-profiles file`,
          ),
        );
      }
    }
  }

  const expectedIndex = buildCatalogIndex(root);
  const expectedVendors = buildVendorList(root);
  const currentIndex = readJsonFile(join(root, "models", "index.json"));
  const currentVendors = readJsonFile(join(root, "models", "vendors.json"));

  const currentIndexVendors = new Map(
    (currentIndex.vendors ?? []).map((vendor, index) => [
      `${vendor.vendorCode}/${vendor.regionCode}`,
      { vendor, index },
    ]),
  );
  const expectedIndexVendors = new Map(
    (expectedIndex.vendors ?? []).map((vendor) => [`${vendor.vendorCode}/${vendor.regionCode}`, vendor]),
  );
  for (const [vendorRegionKey, expectedVendor] of expectedIndexVendors) {
    const current = currentIndexVendors.get(vendorRegionKey);
    if (!current) {
      issues.push(issue("index.vendor_region.missing", "models/index.json#/vendors", `${vendorRegionKey} is missing from models/index.json`));
      continue;
    }
    const path = `models/index.json#/vendors/${current.index}`;
    const checks = [
      ["path", "index.path.mismatch"],
      ["familiesPath", "index.families_path.mismatch"],
      ["modelsPath", "index.models_path.mismatch"],
      ["pricingPath", "index.pricing_path.mismatch"],
      ["rankingsPath", "index.rankings_path.mismatch"],
      ["catalogKeyPrefix", "index.catalog_key_prefix.mismatch"],
      ["modelCount", "index.model_count.mismatch"],
      ["pricingFileCount", "index.pricing_file_count.mismatch"],
      ["familyCount", "index.family_count.mismatch"],
      ["rankingSnapshotCount", "index.ranking_snapshot_count.mismatch"],
      ["sha256", "index.sha256.mismatch"],
    ];
    for (const [field, code] of checks) {
      if (stableJson(current.vendor[field]) !== stableJson(expectedVendor[field])) {
        issues.push(issue(code, `${path}/${field}`, `${vendorRegionKey} ${field} must match generated catalog index`));
      }
    }
    for (const [field, code] of [
      ["modelFiles", "index.model_files.mismatch"],
      ["pricingFiles", "index.pricing_files.mismatch"],
    ]) {
      if (stableJson(current.vendor[field]) !== stableJson(expectedVendor[field])) {
        issues.push(issue(code, `${path}/${field}`, `${vendorRegionKey} ${field} must exactly match published JSON files`));
      }
    }
  }
  for (const [vendorRegionKey, current] of currentIndexVendors) {
    if (!expectedIndexVendors.has(vendorRegionKey)) {
      issues.push(issue("index.vendor_region.extra", `models/index.json#/vendors/${current.index}`, `${vendorRegionKey} is not backed by a catalog directory`));
    }
  }
  for (const vendor of currentIndex.vendors ?? []) {
    const pathPrefix = `models/${vendor.vendorCode}/${vendor.regionCode}`;
    for (const [field, directory] of [
      ["modelFiles", "models"],
      ["pricingFiles", "pricing"],
    ]) {
      const files = vendor[field];
      if (!Array.isArray(files)) {
        issues.push(issue(`index.${field}.invalid`, `models/index.json#/${vendor.vendorCode}/${vendor.regionCode}/${field}`, `${field} must be an array`));
        continue;
      }
      for (const [index, relPath] of files.entries()) {
        if (typeof relPath !== "string") {
          issues.push(issue(`index.${field}.invalid`, `models/index.json#/${vendor.vendorCode}/${vendor.regionCode}/${field}/${index}`, `${field} entries must be strings`));
          continue;
        }
        if (!relPath.startsWith(`${vendor.vendorCode}/${vendor.regionCode}/${directory}/`)) {
          issues.push(issue(`index.${field}.path_scope`, `models/index.json#/${vendor.vendorCode}/${vendor.regionCode}/${field}/${index}`, `${relPath} must stay inside ${pathPrefix}/${directory}`));
        }
      }
    }
    if (vendor.catalogKeyPrefix !== `${vendor.vendorCode}/`) {
      issues.push(issue("index.catalog_key_prefix.identity", `models/index.json#/${vendor.vendorCode}/${vendor.regionCode}/catalogKeyPrefix`, "catalogKeyPrefix must use vendor/model identity and must not include regionCode"));
    }
  }

  if (stableJson(currentIndex) !== stableJson(expectedIndex)) {
    issues.push(issue("index.stale", "models/index.json", "models/index.json must be regenerated with tools/build-index.mjs"));
  }
  if (stableJson(currentVendors) !== stableJson(expectedVendors)) {
    issues.push(issue("vendors.stale", "models/vendors.json", "models/vendors.json must be regenerated with tools/build-index.mjs"));
  }

  return {
    ok: issues.every((item) => item.severity !== "error"),
    issues,
  };
}

function collectJsonFileRefsForDirectory(modelsRoot, root) {
  return collectNestedJsonFiles(root).map((path) => path.slice(modelsRoot.length + 1).replace(/\\/g, "/"));
}

function collectNestedJsonFiles(root) {
  const files = [];
  const entries = existsSync(root) ? readdirSync(root, { withFileTypes: true }) : [];
  for (const entry of entries) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectNestedJsonFiles(path));
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(".json")) {
      files.push(path);
    }
  }
  return files.sort((left, right) => left.localeCompare(right));
}

function safeModelIdPath(modelId, issues, path) {
  try {
    return modelIdPath(modelId);
  } catch {
    issues.push(issue("model_id.path.invalid", path, `${modelId} must be a safe relative modelId path`));
    return String(modelId ?? "__invalid__").replace(/\\/g, "/");
  }
}

function modelIdentityDifferences(left, right) {
  const fields = [
    "catalogKey",
    "modelId",
    "displayName",
    "vendorCode",
    "familyCode",
    "primaryCapability",
    "capabilities",
    "inputModalities",
    "outputModalities",
    "apiFormat",
    "contextTokens",
    "maxInputTokens",
    "maxOutputTokens",
    "supportsStreaming",
    "supportsTools",
    "supportsJsonSchema",
    "usageScopes",
    "codingVisible",
    "replacementModel",
  ];
  return fields.filter((field) => stableJson(left[field] ?? null) !== stableJson(right[field] ?? null));
}

const REQUIRED_CLIENT_APIS = {
  codex: {
    displayName: "Codex",
    defaultApiCode: "openai.codex.responses",
    defaultResourceCode: "api.openai.codex",
  },
  claude_code: {
    displayName: "Claude Code",
    defaultApiCode: "anthropic.claude_code",
    defaultResourceCode: "api.anthropic.claude_code",
  },
  gemini_cli: {
    displayName: "Gemini CLI",
    defaultApiCode: "gemini.generate_content",
    defaultResourceCode: "api.gemini.generate_content",
  },
};

function loadRoutingResourceIndex(root) {
  const resourcesRoot = join(root, "..", "ai-routing", "resources");
  const resourceCodes = new Set();
  const apiCodes = new Set();
  let available = false;
  for (const fileName of ["openai-resources.json", "vendor-native-resources.json"]) {
    const path = join(resourcesRoot, fileName);
    if (!existsSync(path)) {
      continue;
    }
    available = true;
    const payload = readJsonFile(path);
    for (const item of payload.items ?? []) {
      if (typeof item.resourceCode === "string") {
        resourceCodes.add(item.resourceCode);
      }
      if (typeof item.apiCode === "string") {
        apiCodes.add(item.apiCode);
      }
    }
  }
  return { apiCodes, resourceCodes, available };
}

function validateClientApiCompatibility({ vendor, protocolCodes, routingResourceIndex, pathPrefix, issues }) {
  const compatibility = vendor.clientApiCompatibility;
  if (!compatibility || typeof compatibility !== "object" || Array.isArray(compatibility)) {
    issues.push(issue("vendor.client_api_compatibility.missing", `${pathPrefix}/vendor.json#/clientApiCompatibility`, "clientApiCompatibility must declare codex, claude_code, and gemini_cli"));
    return;
  }
  for (const [clientApiCode, standard] of Object.entries(REQUIRED_CLIENT_APIS)) {
    const item = compatibility[clientApiCode];
    const itemPath = `${pathPrefix}/vendor.json#/clientApiCompatibility/${clientApiCode}`;
    if (!item || typeof item !== "object" || Array.isArray(item)) {
      issues.push(issue("vendor.client_api_compatibility.required", itemPath, `${clientApiCode} compatibility is required`));
      continue;
    }
    if (item.clientApiCode !== clientApiCode) {
      issues.push(issue("vendor.client_api_compatibility.code_mismatch", `${itemPath}/clientApiCode`, `clientApiCode must be ${clientApiCode}`));
    }
    if (item.displayName !== standard.displayName) {
      issues.push(issue("vendor.client_api_compatibility.display_name", `${itemPath}/displayName`, `${clientApiCode} displayName must be ${standard.displayName}`));
    }
    if (!["supported", "unsupported", "partial", "convert"].includes(item.supportStatus)) {
      issues.push(issue("vendor.client_api_compatibility.status", `${itemPath}/supportStatus`, "supportStatus must be supported, unsupported, partial, or convert"));
    }
    if (typeof item.notes !== "string" || item.notes.trim().length === 0) {
      issues.push(issue("vendor.client_api_compatibility.notes", `${itemPath}/notes`, "notes must explain the compatibility decision"));
    }
    if (!item.source?.sourceUrl || !item.source?.observedAt) {
      issues.push(issue("vendor.client_api_compatibility.source", `${itemPath}/source`, "sourceUrl and observedAt are required"));
    }
    for (const [field, codeSet, code] of [
      ["protocolCodes", protocolCodes, "vendor.client_api_compatibility.protocol_unknown"],
      ["apiCodes", routingResourceIndex.apiCodes, "vendor.client_api_compatibility.api_unknown"],
      ["resourceCodes", routingResourceIndex.resourceCodes, "vendor.client_api_compatibility.resource_unknown"],
    ]) {
      if (!Array.isArray(item[field])) {
        issues.push(issue("vendor.client_api_compatibility.array", `${itemPath}/${field}`, `${field} must be an array`));
        continue;
      }
      for (const [index, value] of item[field].entries()) {
        if (typeof value !== "string" || value.length === 0) {
          issues.push(issue("vendor.client_api_compatibility.value", `${itemPath}/${field}/${index}`, `${field} entries must be non-empty strings`));
        } else if ((field === "protocolCodes" || routingResourceIndex.available) && !codeSet.has(value)) {
          issues.push(issue(code, `${itemPath}/${field}/${index}`, `${value} is not defined`));
        }
      }
    }
    if (item.supportStatus === "supported" || item.supportStatus === "partial") {
      if (!item.apiCodes?.includes(standard.defaultApiCode)) {
        issues.push(issue("vendor.client_api_compatibility.default_api_missing", `${itemPath}/apiCodes`, `${clientApiCode} support must include ${standard.defaultApiCode}`));
      }
      if (!item.resourceCodes?.includes(standard.defaultResourceCode)) {
        issues.push(issue("vendor.client_api_compatibility.default_resource_missing", `${itemPath}/resourceCodes`, `${clientApiCode} support must include ${standard.defaultResourceCode}`));
      }
    }
  }
}

// 标准兼容族:vendor apiEndpoints 仅允许这些 family 作为 key
const API_ENDPOINT_FAMILIES = new Set(["openai", "anthropic", "google"]);

/**
 * 校验 vendor apiEndpoints 配置(兼容接口 Base URL 原始配置):
 * - family key 必须是标准兼容族(openai/anthropic/google)
 * - family 必须被 supportedProtocols 中同族协议声明支持
 * - host 必须是纯小写域名(无协议、无路径)
 * - pathPrefix 必须是该 family 标准集合(models/protocols.json#/families)中的值
 */
function validateApiEndpoints({ vendor, protocolFamilyByCode, familyPathPrefixes, pathPrefix, issues }) {
  const endpoints = vendor.apiEndpoints;
  if (endpoints === undefined || endpoints === null) {
    // 允许缺失:部分 vendor 无标准兼容端点(如纯音频/视频厂商)
    return;
  }
  if (typeof endpoints !== "object" || Array.isArray(endpoints)) {
    issues.push(issue("vendor.api_endpoints.invalid", `${pathPrefix}/vendor.json#/apiEndpoints`, "apiEndpoints must be an object keyed by protocol family"));
    return;
  }
  const supportedFamilies = new Set();
  for (const protocolCode of vendor.supportedProtocols ?? []) {
    const family = protocolFamilyByCode.get(protocolCode);
    if (family) {
      supportedFamilies.add(family);
    }
  }
  for (const [family, endpoint] of Object.entries(endpoints)) {
    const itemPath = `${pathPrefix}/vendor.json#/apiEndpoints/${family}`;
    if (!API_ENDPOINT_FAMILIES.has(family)) {
      issues.push(issue("vendor.api_endpoints.family.unknown", itemPath, `family ${family} is not a standard compatibility family; allowed: ${[...API_ENDPOINT_FAMILIES].join(", ")}`));
      continue;
    }
    if (!supportedFamilies.has(family)) {
      issues.push(issue("vendor.api_endpoint.unsupported_family", itemPath, `family ${family} must be supported by a protocolCode of the same family in supportedProtocols`));
    }
    if (!endpoint || typeof endpoint !== "object" || Array.isArray(endpoint)) {
      issues.push(issue("vendor.api_endpoint.invalid", itemPath, "apiEndpoint must be an object with host and pathPrefix"));
      continue;
    }
    if (typeof endpoint.host !== "string" || !/^[a-z0-9][a-z0-9.-]*$/.test(endpoint.host)) {
      issues.push(issue("vendor.api_endpoint.host.invalid", `${itemPath}/host`, "host must be a lowercase domain without scheme or path"));
    }
    const standardPathPrefixes = familyPathPrefixes.get(family);
    if (typeof endpoint.pathPrefix !== "string" || (endpoint.pathPrefix !== "" && (!endpoint.pathPrefix.startsWith("/") || endpoint.pathPrefix.endsWith("/")))) {
      issues.push(issue("vendor.api_endpoint.path.invalid", `${itemPath}/pathPrefix`, "pathPrefix must be empty or start with '/' and not end with '/'"));
    } else if (!standardPathPrefixes?.has(endpoint.pathPrefix)) {
      const allowed = standardPathPrefixes ? [...standardPathPrefixes].map((value) => JSON.stringify(value)).join(", ") : "(family not declared in models/protocols.json)";
      issues.push(issue("vendor.api_endpoint.path.not_standard", `${itemPath}/pathPrefix`, `pathPrefix ${JSON.stringify(endpoint.pathPrefix)} is not in the ${family} family standard set: ${allowed}`));
    }
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const payload = validateCatalog(projectRootFromTool(import.meta.url));
  console.log(JSON.stringify(payload, null, 2));
  if (!payload.ok) {
    process.exit(1);
  }
}
