#!/usr/bin/env node
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { migrateOpenApiDocument } from "../../sdkwork-specs/tools/lib/migrate-openapi-legacy-envelope.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const clawRouterRoot = join(root, "..", "sdkwork-clawrouter");

const BACKEND_PATH_PREFIXES = [
  "/backend/v3/api/ai/model_vendors",
  "/backend/v3/api/ai/models",
  "/backend/v3/api/ai/model_mappings",
  "/backend/v3/api/ai/model_rankings",
  "/backend/v3/api/ai/resources",
  "/backend/v3/api/ai/resource_groups",
  "/backend/v3/api/ai/voices",
  "/backend/v3/api/ai/video_profiles",
];

const APP_PATH_PREFIXES = [
  "/app/v3/api/ai/model_vendors",
  "/app/v3/api/ai/models",
  "/app/v3/api/ai/model_rankings",
  "/app/v3/api/ai/voices",
  "/app/v3/api/ai/video_profiles",
];

function matchesPrefix(pathKey, prefixes) {
  return prefixes.some(
    (prefix) => pathKey === prefix || pathKey.startsWith(`${prefix}/`),
  );
}

function collectSchemaRefs(node, refs = new Set()) {
  if (node === null || node === undefined) {
    return refs;
  }
  if (Array.isArray(node)) {
    for (const item of node) {
      collectSchemaRefs(item, refs);
    }
    return refs;
  }
  if (typeof node !== "object") {
    return refs;
  }
  if (typeof node.$ref === "string" && node.$ref.startsWith("#/components/schemas/")) {
    refs.add(node.$ref.slice("#/components/schemas/".length));
  }
  for (const value of Object.values(node)) {
    collectSchemaRefs(value, refs);
  }
  return refs;
}

function expandSchemas(schemas, seedNames) {
  const selected = new Set(seedNames);
  let changed = true;
  while (changed) {
    changed = false;
    for (const name of [...selected]) {
      const schema = schemas[name];
      if (!schema) {
        continue;
      }
      for (const nested of collectSchemaRefs(schema)) {
        if (!selected.has(nested)) {
          selected.add(nested);
          changed = true;
        }
      }
    }
  }
  const ordered = [...selected].sort();
  return Object.fromEntries(ordered.map((name) => [name, schemas[name]]));
}

function extractSurface(sourcePath, pathPrefixes, title, serverUrl) {
  const source = JSON.parse(readFileSync(sourcePath, "utf8"));
  const paths = {};
  const seedSchemaNames = new Set();

  for (const [pathKey, pathItem] of Object.entries(source.paths || {})) {
    if (!matchesPrefix(pathKey, pathPrefixes)) {
      continue;
    }
    paths[pathKey] = pathItem;
    collectSchemaRefs(pathItem, seedSchemaNames);
  }

  const schemas = source.components?.schemas || {};
  const selectedSchemas = expandSchemas(schemas, seedSchemaNames);

  return {
    openapi: source.openapi || "3.1.0",
    info: {
      title,
      version: "0.1.0",
      description:
        "Intelligence catalog API owned by sdkwork-models. Materialized from composed host catalog mount OpenAPI authority.",
    },
    servers: [{ url: serverUrl }],
    paths,
    components: {
      ...(source.components?.securitySchemes
        ? { securitySchemes: source.components.securitySchemes }
        : {}),
      schemas: selectedSchemas,
    },
  };
}

function injectModelsListQueryParams(document) {
  for (const pathKey of Object.keys(document.paths || {})) {
    if (!pathKey.endsWith("/ai/models")) {
      continue;
    }
    const operation = document.paths[pathKey]?.get;
    if (!operation) {
      continue;
    }
    const parameters = operation.parameters || [];
    if (parameters.some((parameter) => parameter.name === "model_types")) {
      continue;
    }
    parameters.push({
      description:
        "Comma-separated model type labels (Chat, Image, Embedding, ...).",
      in: "query",
      name: "model_types",
      required: false,
      schema: {
        type: "string",
      },
    });
    operation.parameters = parameters;
  }
  return document;
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function mergeVoiceCatalogPaths(document, options) {
  const { templatePath, listPath, modelListPath, modelIdParam } = options;
  const template = document.paths?.[templatePath]?.get;
  if (!template) {
    return document;
  }

  const voiceQueryParams = [
    { in: "query", name: "vendor_code", required: false, schema: { type: "string" }, description: "Filter by vendor code." },
    { in: "query", name: "region_code", required: false, schema: { type: "string" }, description: "Filter by vendor region code." },
    { in: "query", name: "locale", required: false, schema: { type: "string" }, description: "BCP-47 locale filter." },
    { in: "query", name: "catalog_key", required: false, schema: { type: "string" }, description: "Filter by model catalog key vendor/modelId." },
    { in: "query", name: "model_id", required: false, schema: { type: "string" }, description: "Filter by model id when combined with vendor_code." },
    { in: "query", name: "q", required: false, schema: { type: "string" }, description: "Free-text search on displayName and voiceId." },
  ];

  const voicesList = cloneJson(template);
  voicesList.operationId = "voices.list";
  voicesList.summary = "List TTS voices";
  voicesList.description = "List catalog TTS voices with optional vendor, locale, and model filters.";
  voicesList.parameters = voiceQueryParams;
  voicesList["x-sdkwork-resource"] = "voices";

  const modelVoicesList = cloneJson(template);
  modelVoicesList.operationId = "modelVoices.list";
  modelVoicesList.summary = "List model TTS voices";
  modelVoicesList.description = "List TTS voices bound to a model catalog key.";
  modelVoicesList.parameters = [
    { in: "path", name: modelIdParam, required: true, schema: { type: "string" }, description: "Model id or catalog key vendor/modelId." },
    { in: "query", name: "vendor_code", required: false, schema: { type: "string" }, description: "Required when model id is not a catalog key." },
  ];
  modelVoicesList["x-sdkwork-resource"] = "modelVoices";

  document.paths[listPath] = { get: voicesList };
  document.paths[modelListPath] = { get: modelVoicesList };
  return document;
}

function mergeVoiceCatalogAppPaths(document) {
  return mergeVoiceCatalogPaths(document, {
    templatePath: "/app/v3/api/ai/model_rankings",
    listPath: "/app/v3/api/ai/voices",
    modelListPath: "/app/v3/api/ai/models/{modelId}/voices",
    modelIdParam: "modelId",
  });
}

function mergeVoiceCatalogBackendPaths(document) {
  return mergeVoiceCatalogPaths(document, {
    templatePath: "/backend/v3/api/ai/model_rankings",
    listPath: "/backend/v3/api/ai/voices",
    modelListPath: "/backend/v3/api/ai/models/{modelId}/voices",
    modelIdParam: "modelId",
  });
}

function mergeVideoProfileCatalogPaths(document, options) {
  const { templatePath, listPath, modelListPath, modelIdParam } = options;
  const template = document.paths?.[templatePath]?.get;
  if (!template) {
    return document;
  }

  const profileQueryParams = [
    { in: "query", name: "vendor_code", required: false, schema: { type: "string" }, description: "Filter by vendor code." },
    { in: "query", name: "region_code", required: false, schema: { type: "string" }, description: "Filter by vendor region code." },
    { in: "query", name: "catalog_key", required: false, schema: { type: "string" }, description: "Filter by model catalog key vendor/modelId." },
    { in: "query", name: "model_id", required: false, schema: { type: "string" }, description: "Filter by model id when combined with vendor_code." },
    { in: "query", name: "generation_mode", required: false, schema: { type: "string" }, description: "Filter by generation mode (text_to_video, image_to_video, ...)." },
    { in: "query", name: "duration_tier_code", required: false, schema: { type: "string" }, description: "Filter by canonical duration tier (dur_5s, dur_10s, ...)." },
    { in: "query", name: "resolution", required: false, schema: { type: "string" }, description: "Filter by output resolution (720p, 1080p, ...)." },
  ];

  const profilesList = cloneJson(template);
  profilesList.operationId = "videoProfiles.list";
  profilesList.summary = "List video generation profiles";
  profilesList.description = "List catalog video generation profiles with optional vendor, mode, duration, and resolution filters.";
  profilesList.parameters = profileQueryParams;
  profilesList["x-sdkwork-resource"] = "videoProfiles";

  const modelProfilesList = cloneJson(template);
  modelProfilesList.operationId = "modelVideoProfiles.list";
  modelProfilesList.summary = "List model video generation profiles";
  modelProfilesList.description = "List video generation profiles bound to a model catalog key.";
  modelProfilesList.parameters = [
    { in: "path", name: modelIdParam, required: true, schema: { type: "string" }, description: "Model id or catalog key vendor/modelId." },
    { in: "query", name: "vendor_code", required: false, schema: { type: "string" }, description: "Required when model id is not a catalog key." },
  ];
  modelProfilesList["x-sdkwork-resource"] = "modelVideoProfiles";

  document.paths[listPath] = { get: profilesList };
  document.paths[modelListPath] = { get: modelProfilesList };
  return document;
}

function mergeVideoProfileCatalogAppPaths(document) {
  return mergeVideoProfileCatalogPaths(document, {
    templatePath: "/app/v3/api/ai/model_rankings",
    listPath: "/app/v3/api/ai/video_profiles",
    modelListPath: "/app/v3/api/ai/models/{modelId}/video_profiles",
    modelIdParam: "modelId",
  });
}

function mergeVideoProfileCatalogBackendPaths(document) {
  return mergeVideoProfileCatalogPaths(document, {
    templatePath: "/backend/v3/api/ai/model_rankings",
    listPath: "/backend/v3/api/ai/video_profiles",
    modelListPath: "/backend/v3/api/ai/models/{modelId}/video_profiles",
    modelIdParam: "modelId",
  });
}

function modelCatalogSyncResultSchema() {
  return {
    type: "object",
    additionalProperties: false,
    description: "Catalog sync result returned by models.refresh.",
    properties: {
      synced: { type: "boolean" },
      source: { type: "string" },
      mode: { type: "string" },
      dryRun: { type: "boolean" },
      catalogVersion: { type: "string" },
      requestedCatalogVersion: { type: ["string", "null"] },
      catalogRoot: { type: ["string", "null"] },
      vendorCodes: { type: "array", items: { type: "string" } },
      sourceHash: { type: "string" },
      meterCount: { type: "integer", minimum: 0 },
      vendorCount: { type: "integer", minimum: 0 },
      familyCount: { type: "integer", minimum: 0 },
      modelCount: { type: "integer", minimum: 0 },
      capabilityCount: { type: "integer", minimum: 0 },
      priceCount: { type: "integer", minimum: 0 },
      rankingCount: { type: "integer", minimum: 0 },
      voiceCount: { type: "integer", minimum: 0 },
      voiceBindingCount: { type: "integer", minimum: 0 },
      videoProfileCount: { type: "integer", minimum: 0 },
      acceptedCount: { type: "integer", minimum: 0 },
      snapshotId: { type: ["string", "null"] },
      syncRunId: { type: ["string", "null"] },
      vendors: { type: "array", items: { type: "object", additionalProperties: true } },
      models: { type: "array", items: { type: "object", additionalProperties: true } },
    },
    required: [
      "synced",
      "source",
      "mode",
      "dryRun",
      "catalogVersion",
      "vendorCodes",
      "sourceHash",
      "meterCount",
      "vendorCount",
      "familyCount",
      "modelCount",
      "capabilityCount",
      "priceCount",
      "rankingCount",
      "voiceCount",
      "voiceBindingCount",
      "videoProfileCount",
      "acceptedCount",
      "vendors",
      "models",
    ],
  };
}

function mergeModelCatalogSyncSchema(document) {
  if (!document.components) {
    document.components = {};
  }
  if (!document.components.schemas) {
    document.components.schemas = {};
  }

  document.components.schemas.ModelCatalogSyncResult = modelCatalogSyncResultSchema();
  delete document.components.schemas.ModelsRefreshResult;

  const refreshPath = document.paths?.["/backend/v3/api/ai/models/refresh"];
  const successSchema = refreshPath?.post?.responses?.["200"]?.content?.["application/json"]?.schema;
  if (successSchema?.allOf) {
    for (const part of successSchema.allOf) {
      if (part?.properties?.data?.properties?.item) {
        delete part.properties.data.properties.item;
        part.properties.data = {
          allOf: [{ $ref: "#/components/schemas/ModelCatalogSyncResult" }],
        };
        break;
      }
    }
  }

  return document;
}

function serializeJson(document) {
  return `${JSON.stringify(document, null, 2)}\n`;
}

function writeJson(targetPath, document) {
  mkdirSync(dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, serializeJson(document), "utf8");
  console.log(`exported ${targetPath} (${Object.keys(document.paths).length} paths)`);
}

function assertCurrent(targetPath, document) {
  const expected = serializeJson(document);
  const actual = readFileSync(targetPath, "utf8");
  if (actual !== expected) {
    console.error(`openapi drift detected: ${targetPath}`);
    process.exit(1);
  }
  console.log(`openapi current: ${targetPath}`);
}

const checkOnly = process.argv.includes("--check");

const backendSource = join(
  clawRouterRoot,
  "generated/openapi/clawrouter-models-catalog-backend-openapi.json",
);
const appSource = join(
  clawRouterRoot,
  "generated/openapi/clawrouter-models-catalog-app-openapi.json",
);

const backendDocument = migrateOpenApiDocument(
  mergeModelCatalogSyncSchema(
    mergeVideoProfileCatalogBackendPaths(
      mergeVoiceCatalogBackendPaths(
        injectModelsListQueryParams(
          extractSurface(
            backendSource,
            BACKEND_PATH_PREFIXES,
            "SDKWork Models Backend API",
            "/backend/v3/api",
          ),
        ),
      ),
    ),
  ),
);
const appDocument = migrateOpenApiDocument(
  mergeVideoProfileCatalogAppPaths(
    mergeVoiceCatalogAppPaths(
      injectModelsListQueryParams(
        extractSurface(
          appSource,
          APP_PATH_PREFIXES,
          "SDKWork Models App API",
          "/app/v3/api",
        ),
      ),
    ),
  ),
);

const backendTarget = join(root, "apis/backend-api/intelligence/openapi.json");
const appTarget = join(root, "apis/app-api/intelligence/openapi.json");

if (checkOnly) {
  assertCurrent(backendTarget, backendDocument);
  assertCurrent(appTarget, appDocument);
} else {
  writeJson(backendTarget, backendDocument);
  writeJson(appTarget, appDocument);
}
