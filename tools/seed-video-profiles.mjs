#!/usr/bin/env node
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import {
  catalogKey,
  collectRegionalCatalogDirectories,
  loadManifest,
  loadVendorBundle,
  projectRootFromTool,
  stableJson,
  videoProfileCatalogKey,
} from "./catalog-lib.mjs";

const GENERATION_MODES_BY_INPUT = {
  text: "text_to_video",
  image: "image_to_video",
  video: "reference_to_video",
};

const VENDOR_SOURCE_URLS = {
  alibaba: "https://help.aliyun.com/zh/model-studio/developer-reference/video-generation",
  bytedance: "https://www.volcengine.com/docs/82379",
  google: "https://ai.google.dev/gemini-api/docs/video",
  kuaishou: "https://app.klingai.com/global/dev/document-api",
  luma_ai: "https://docs.lumalabs.ai/docs/video-generation",
  minimax: "https://platform.minimax.io/docs/guides/pricing-video",
  openai: "https://platform.openai.com/docs/models/sora-2",
  pixverse: "https://docs.platform.pixverse.ai/",
  runway: "https://docs.dev.runwayml.com/guides/pricing/",
  vidu: "https://platform.vidu.com/docs/pricing",
  xai: "https://docs.x.ai/docs/models",
  zhipu: "https://open.bigmodel.cn/dev/api/videomodel/cogvideox",
};

function isVideoGenerationModel(model) {
  return model.primaryCapability === "video";
}

function pricingTierCodes(pricingRows, modelId) {
  const pricing = pricingRows.find((row) => row.modelId === modelId);
  return new Set(
    (pricing?.prices ?? [])
      .map((price) => price.tierCode)
      .filter((tierCode) => typeof tierCode === "string" && tierCode.length > 0),
  );
}

function durationTierCode(seconds) {
  return `dur_${seconds}s`;
}

function generationModesForModel(model, modelId) {
  const modes = new Set();
  for (const modality of model.inputModalities ?? []) {
    const mode = GENERATION_MODES_BY_INPUT[modality];
    if (mode) {
      modes.add(mode);
    }
  }
  if (modelId.includes("r2v") || modelId.includes("reference")) {
    modes.add("reference_to_video");
  }
  if (modelId.includes("kling") && (model.inputModalities ?? []).includes("video")) {
    modes.add("multi_shot");
  }
  if (modes.size === 0) {
    modes.add("text_to_video");
  }
  return [...modes];
}

function vendorDurationTemplate(vendorCode, modelId, tiers) {
  if (tiers.has("dur_5s") && tiers.has("dur_10s")) {
    return {
      policy: "fixed",
      durations: [5, 10],
      resolution: "720p",
      resolutionTierCode: "res_720p",
      usePricingTiers: true,
    };
  }
  if (vendorCode === "openai" || modelId.startsWith("sora-")) {
    return {
      policy: "continuous",
      min: 4,
      max: 12,
      step: 1,
      resolution: modelId.includes("pro") ? "1080p" : "720p",
      resolutionTierCode: modelId.includes("pro") ? "res_1080p" : "res_720p",
    };
  }
  if (vendorCode === "kuaishou" || modelId.includes("kling")) {
    return {
      policy: "range",
      min: 3,
      max: 15,
      step: 1,
      resolution: "1080p",
      resolutionTierCode: "res_1080p",
      outputAudio: true,
    };
  }
  if (vendorCode === "bytedance" || modelId.includes("seedance")) {
    return {
      policy: "range",
      min: 2,
      max: 12,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
      outputAudio: true,
    };
  }
  if (vendorCode === "minimax" || modelId.includes("hailuo")) {
    const fast = modelId.includes("fast");
    return {
      policy: "range",
      min: 6,
      max: fast ? 6 : 10,
      step: 1,
      resolution: "768p",
      resolutionTierCode: "res_768p",
    };
  }
  if (vendorCode === "google" || modelId.includes("veo")) {
    return {
      policy: "range",
      min: 5,
      max: 8,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
    };
  }
  if (vendorCode === "runway") {
    return {
      policy: "discrete",
      durationOptions: [5, 10],
      durationTierCodes: ["dur_5s", "dur_10s"],
      resolution: "720p",
      resolutionTierCode: "res_720p",
    };
  }
  if (vendorCode === "luma_ai" || modelId.includes("ray")) {
    return {
      policy: "range",
      min: 5,
      max: 9,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
    };
  }
  if (vendorCode === "pixverse") {
    return {
      policy: "range",
      min: 5,
      max: 8,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
    };
  }
  if (vendorCode === "alibaba" || modelId.includes("wan")) {
    return {
      policy: "range",
      min: 5,
      max: 10,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
      outputAudio: modelId.includes("wan2.6"),
    };
  }
  if (vendorCode === "xai" || modelId.includes("grok-imagine-video")) {
    return {
      policy: "range",
      min: 5,
      max: 15,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
    };
  }
  if (vendorCode === "vidu") {
    return {
      policy: "range",
      min: 5,
      max: 10,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
      outputAudio: true,
    };
  }
  if (vendorCode === "zhipu" || modelId.includes("cogvideo")) {
    return {
      policy: "range",
      min: 5,
      max: 10,
      step: 1,
      resolution: "720p",
      resolutionTierCode: "res_720p",
    };
  }
  return {
    policy: "range",
    min: 5,
    max: 10,
    step: 1,
    resolution: "720p",
    resolutionTierCode: "res_720p",
  };
}

function modeAbbrev(generationMode) {
  return {
    text_to_video: "t2v",
    image_to_video: "i2v",
    reference_to_video: "r2v",
    start_end_frame: "se2v",
    video_extension: "vext",
    video_edit: "vedit",
    multi_shot: "multi",
  }[generationMode] ?? generationMode;
}

function buildProfile(model, generationMode, template, sortOrder, isDefault) {
  const { vendorCode, modelId } = model;
  const modeLabel = {
    text_to_video: "Text to Video",
    image_to_video: "Image to Video",
    reference_to_video: "Reference to Video",
    start_end_frame: "Start-End Frame",
    video_extension: "Video Extension",
    video_edit: "Video Edit",
    multi_shot: "Multi-shot",
  }[generationMode] ?? generationMode;

  if (template.policy === "fixed") {
    return template.durations.flatMap((seconds, index) => {
      const tierCode = durationTierCode(seconds);
      const profileCode = `${modeAbbrev(generationMode)}_${seconds}s_${template.resolution}`;
      return {
        profileCode,
        catalogKey: videoProfileCatalogKey(vendorCode, modelId, profileCode),
        displayName: `${modeLabel} · ${seconds}s · ${template.resolution}`,
        generationMode,
        durationPolicy: "fixed",
        durationSeconds: seconds,
        durationTierCode: tierCode,
        resolution: template.resolution,
        resolutionTierCode: template.resolutionTierCode,
        aspectRatios: ["16:9", "9:16", "1:1"],
        outputAudio: template.outputAudio ?? false,
        isDefault: isDefault && index === 0,
        sortOrder: sortOrder + index * 10,
        ...(template.usePricingTiers ? { pricingTierCodes: [tierCode] } : {}),
        wireParameters: {
          duration: String(seconds),
          resolution: template.resolution,
        },
      };
    });
  }

  if (template.policy === "discrete") {
    const profileCode = `${modeAbbrev(generationMode)}_discrete_${template.resolution}`;
    return [
      {
        profileCode,
        catalogKey: videoProfileCatalogKey(vendorCode, modelId, profileCode),
        displayName: `${modeLabel} · ${template.durationOptions.join("/")}s · ${template.resolution}`,
        generationMode,
        durationPolicy: "discrete",
        durationOptions: template.durationOptions,
        durationTierCodes: template.durationTierCodes,
        resolution: template.resolution,
        resolutionTierCode: template.resolutionTierCode,
        aspectRatios: ["16:9", "9:16", "1:1"],
        outputAudio: template.outputAudio ?? false,
        isDefault,
        sortOrder,
        wireParameters: {
          duration: template.durationOptions.join(","),
          resolution: template.resolution,
        },
      },
    ];
  }

  const abbrev = template.policy === "continuous" ? "continuous" : "range";
  const profileCode = `${modeAbbrev(generationMode)}_${abbrev}_${template.resolution}`;
  const durationLabel = `${template.min}–${template.max}s`;
  return [
    {
      profileCode,
      catalogKey: videoProfileCatalogKey(vendorCode, modelId, profileCode),
      displayName: `${modeLabel} · ${durationLabel} · ${template.resolution}`,
      generationMode,
      durationPolicy: template.policy,
      minDurationSeconds: template.min,
      maxDurationSeconds: template.max,
      durationStepSeconds: template.step,
      resolution: template.resolution,
      resolutionTierCode: template.resolutionTierCode,
      aspectRatios: ["16:9", "9:16", "1:1"],
      outputAudio: template.outputAudio ?? false,
      isDefault,
      sortOrder,
      wireParameters: {
        duration: `${template.min}-${template.max}`,
        resolution: template.resolution,
      },
    },
  ];
}

function buildProfilesFile(model, pricingRows) {
  const tiers = pricingTierCodes(pricingRows, model.modelId);
  const template = vendorDurationTemplate(model.vendorCode, model.modelId, tiers);
  const modes = generationModesForModel(model, model.modelId);
  let sortOrder = 10;
  const profiles = [];
  for (const [index, generationMode] of modes.entries()) {
    const built = buildProfile(model, generationMode, template, sortOrder, index === 0);
    profiles.push(...built);
    sortOrder += built.length * 10;
  }
  const pricing = pricingRows.find((row) => row.modelId === model.modelId);
  return {
    schemaVersion: "1.0.0",
    vendorCode: model.vendorCode,
    regionCode: model.regionCode,
    modelId: model.modelId,
    catalogKey: catalogKey(model.vendorCode, model.modelId),
    profiles,
    source: {
      observedAt: "2026-07-04T00:00:00Z",
      sourceUrl:
        model.source?.sourceUrl ??
        pricing?.prices?.[0]?.source?.sourceUrl ??
        VENDOR_SOURCE_URLS[model.vendorCode] ??
        "https://sdkwork.cloud/models",
    },
  };
}

export function seedVideoProfiles(root, { dryRun = false } = {}) {
  const manifest = loadManifest(root);
  const created = [];
  const skipped = [];

  for (const regionDir of collectRegionalCatalogDirectories(join(root, "models"))) {
    const bundle = loadVendorBundle(regionDir);
    const profilesDir = join(regionDir, "model-video-profiles");
    if (!dryRun && !existsSync(profilesDir)) {
      mkdirSync(profilesDir, { recursive: true });
    }

    for (const model of bundle.models) {
      if (!isVideoGenerationModel(model)) {
        continue;
      }

      const profilePath = join(profilesDir, `${model.modelId}.json`);
      if (existsSync(profilePath)) {
        skipped.push(profilePath);
        continue;
      }

      const file = buildProfilesFile(model, bundle.pricing);
      if (!dryRun) {
        writeFileSync(profilePath, `${stableJson(file)}\n`, "utf8");
      }
      created.push(profilePath);
    }
  }

  return { created, skipped, catalogVersion: manifest.catalogVersion };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = projectRootFromTool(import.meta.url);
  const dryRun = process.argv.includes("--dry-run");
  const { created, skipped } = seedVideoProfiles(root, { dryRun });
  console.log(
    `seed-video-profiles: created ${created.length}, skipped ${skipped.length}${dryRun ? " (dry-run)" : ""}`,
  );
  for (const path of created) {
    console.log(`  + ${path.replace(root, "").replace(/^[/\\]/, "")}`);
  }
}
