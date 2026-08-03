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
  "/app/v3/api/ai/model_access_channels",
  "/app/v3/api/ai/model_rankings",
  "/app/v3/api/ai/voices",
  "/app/v3/api/ai/video_profiles",
];

const AI_RESOURCE_TYPES = [
  "vendor",
  "modality",
  "api_endpoint",
  "model",
  "model_api",
  "bundle",
  "model_access_channel",
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

function mergeAiResourceGroupMemberPaths(document) {
  const updateTemplate = operationAt(
    document,
    "/backend/v3/api/ai/resource_groups/{groupId}",
    "patch",
  );
  const deleteTemplate = operationAt(
    document,
    "/backend/v3/api/ai/resource_groups/{groupId}",
    "delete",
  );
  if (!updateTemplate || !deleteTemplate) {
    return document;
  }
  const pathParameters = [
    {
      in: "path",
      name: "groupId",
      required: true,
      schema: int64StringSchema(false, 1),
      description: "Tenant-owned AI resource group id.",
    },
    {
      in: "path",
      name: "resourceCode",
      required: true,
      schema: safeCodeSchema(192, false),
      description: "AI API resource code.",
    },
  ];
  const updateOperation = cloneJson(updateTemplate);
  updateOperation.operationId = "resourceGroups.resources.update";
  updateOperation.summary = "Assign or update a resource-group member";
  updateOperation.description =
    "Idempotently assign one API resource to a tenant-owned manual resource group, or update its role and sort order. The group row is locked and the membership mutation and audit event commit in one transaction.";
  updateOperation.parameters = pathParameters;
  updateOperation["x-contract-kind"] = "update";
  updateOperation["x-sdkwork-resource"] = "resourceGroups.resources";

  const deleteOperation = cloneJson(deleteTemplate);
  deleteOperation.operationId = "resourceGroups.resources.delete";
  deleteOperation.summary = "Detach a resource-group member";
  deleteOperation.description =
    "Idempotently detach one API resource from a tenant-owned manual resource group in the same transaction as its audit event.";
  deleteOperation.parameters = cloneJson(pathParameters);
  deleteOperation["x-contract-kind"] = "delete";
  deleteOperation["x-sdkwork-resource"] = "resourceGroups.resources";

  document.paths[
    "/backend/v3/api/ai/resource_groups/{groupId}/resources/{resourceCode}"
  ] = {
    put: updateOperation,
    delete: deleteOperation,
  };
  return document;
}

function mergeAiResourceWritePaths(document) {
  const createTemplate = operationAt(
    document,
    "/backend/v3/api/ai/resource_groups",
    "post",
  );
  const updateTemplate = operationAt(
    document,
    "/backend/v3/api/ai/resource_groups/{groupId}",
    "patch",
  );
  if (!createTemplate || !updateTemplate) {
    return document;
  }

  const createOperation = cloneJson(createTemplate);
  createOperation.operationId = "resources.create";
  createOperation.summary = "Create AI resource";
  createOperation.description =
    "Create a tenant-owned AI resource, including model access channels and their public routing metadata. Credentials are not accepted by this catalog API.";
  createOperation.parameters = [];
  createOperation["x-contract-kind"] = "create";
  createOperation["x-read-sources"] = ["ai_resource"];
  createOperation["x-write-tables"] = ["ai_resource", "ai_resource_group_item", "ops_audit_log"];
  createOperation["x-sdkwork-resource"] = "resources";
  createOperation["x-source-file"] =
    "../sdkwork-models/crates/sdkwork-models-catalog-service/src/api/admin_ai_resource.rs";

  const updateOperation = cloneJson(updateTemplate);
  updateOperation.operationId = "resources.update";
  updateOperation.summary = "Update AI resource";
  updateOperation.description =
    "Update a tenant-owned AI resource and its public model access channel metadata. Credentials are not accepted by this catalog API.";
  updateOperation.parameters = [
    {
      in: "path",
      name: "resourceId",
      required: true,
      schema: int64StringSchema(false, 1),
      description: "Tenant-owned AI resource id.",
    },
  ];
  updateOperation["x-contract-kind"] = "update";
  updateOperation["x-read-sources"] = ["ai_resource"];
  updateOperation["x-write-tables"] = ["ai_resource", "ai_resource_group_item", "ops_audit_log"];
  updateOperation["x-sdkwork-resource"] = "resources";
  updateOperation["x-source-file"] =
    "../sdkwork-models/crates/sdkwork-models-catalog-service/src/api/admin_ai_resource.rs";

  document.paths["/backend/v3/api/ai/resources"] = {
    ...(document.paths["/backend/v3/api/ai/resources"] ?? {}),
    post: createOperation,
  };
  document.paths["/backend/v3/api/ai/resources/{resourceId}"] = {
    put: updateOperation,
  };
  return document;
}

function modelAccessChannelModelSchema() {
  return objectSchema(
    {
      catalogKey: visibleAsciiStringSchema(256, false),
      model: visibleAsciiStringSchema(128, false),
      displayName: stringSchema(128, 1),
      contextTokens: {
        type: ["integer", "null"],
        format: "int32",
        minimum: 1,
        maximum: 100000000,
      },
      maxOutputTokens: {
        type: ["integer", "null"],
        format: "int32",
        minimum: 1,
        maximum: 100000000,
      },
      toolCallRounds: {
        type: ["integer", "null"],
        format: "int32",
        minimum: 1,
        maximum: 10000,
      },
      supportsMultimodal: { type: ["boolean", "null"] },
    },
    ["catalogKey", "model", "displayName"],
    "One model exposed by a model access channel.",
  );
}

function modelAccessChannelModelInputSchema() {
  return objectSchema(
    {
      modelId: visibleAsciiStringSchema(128, false),
      displayName: nullableStringSchema(128),
    },
    ["modelId"],
    "One model selected for a model access channel. Capability metadata is derived from sdkwork-models.",
  );
}

function modelAccessChannelOfferingSchema() {
  return objectSchema(
    {
      vendorCode: safeCodeSchema(64, false),
      vendorName: stringSchema(128, 1),
      models: arraySchema(schemaRef("AppModelAccessChannelModel"), 512),
    },
    ["vendorCode", "vendorName", "models"],
    "Models from one vendor exposed by an official endpoint or relay station.",
  );
}

function modelAccessChannelItemSchema() {
  return objectSchema(
    {
      id: int64StringSchema(false, 1),
      code: safeCodeSchema(192, false),
      name: stringSchema(128, 1),
      kind: enumStringSchema(["official", "relay", "custom"], false),
      baseUrl: {
        type: "string",
        format: "uri",
        maxLength: 2048,
        description: "Public HTTP(S) API base URL. Credentials are never returned by this API.",
      },
      description: nullableStringSchema(512),
      defaultVendorCode: safeCodeSchema(64, false),
      defaultModelId: visibleAsciiStringSchema(128, false),
      supportedAgentProviderIds: arraySchema(
        safeCodeSchema(128, false),
        128,
        "Agent provider ids that can consume this channel.",
      ),
      offerings: arraySchema(schemaRef("AppModelAccessChannelOffering"), 512),
      vendorCount: int32Schema(0),
      modelCount: int32Schema(0),
      sortOrder: int64StringSchema(true, 0),
    },
    [
      "id",
      "code",
      "name",
      "kind",
      "baseUrl",
      "defaultVendorCode",
      "defaultModelId",
      "supportedAgentProviderIds",
      "offerings",
      "vendorCount",
      "modelCount",
    ],
    "Official model endpoint or relay station returned by the app catalog API.",
  );
}

function modelAccessChannelOfferingInputSchema() {
  return objectSchema(
    {
      vendorCode: safeCodeSchema(64, false),
      vendorName: stringSchema(128, 1),
      modelIds: arraySchema(visibleAsciiStringSchema(128, false), 512),
      models: arraySchema(schemaRef("AppModelAccessChannelModelInput"), 512),
    },
    ["vendorCode", "vendorName"],
    "One vendor and its supported model ids for an official endpoint or relay station.",
  );
}

function modelAccessChannelUpsertRequestSchema() {
  return objectSchema(
    {
      name: stringSchema(128, 1),
      kind: enumStringSchema(["official", "relay", "custom"], false),
      baseUrl: {
        type: "string",
        format: "uri",
        minLength: 1,
        maxLength: 2048,
        description: "Public HTTP(S) API base URL. API keys are forbidden.",
      },
      description: nullableStringSchema(512),
      offerings: arraySchema(schemaRef("AppModelAccessChannelOfferingInput"), 32),
      defaultVendorCode: safeCodeSchema(64, false),
      defaultModelId: visibleAsciiStringSchema(128, false),
      supportedAgentProviderIds: arraySchema(safeCodeSchema(128, false), 128),
    },
    [
      "name",
      "kind",
      "baseUrl",
      "offerings",
      "defaultVendorCode",
      "defaultModelId",
      "supportedAgentProviderIds",
    ],
    "Public model access channel metadata. Credentials are intentionally forbidden.",
  );
}

function modelAccessChannelItemDataSchema() {
  return objectSchema(
    { item: schemaRef("AppModelAccessChannelItem") },
    ["item"],
    "Saved model access channel.",
  );
}

function modelAccessChannelUpsertResultSchema() {
  return {
    allOf: [
      schemaRef("SdkWorkApiResponse"),
      objectSchema(
        {
          data: {
            allOf: [schemaRef("AppModelAccessChannelItemData")],
            description: "Saved model access channel data.",
          },
        },
        ["data"],
      ),
    ],
    description: "Model access channel upsert result.",
    "x-operation-id": "modelAccessChannels.upsert",
  };
}

function modelAccessChannelsPageSchema() {
  return objectSchema(
    {
      items: arraySchema(schemaRef("AppModelAccessChannelItem"), 200),
      pageInfo: {
        allOf: [schemaRef("PageInfo")],
        description: "Offset pagination metadata for model access channels.",
      },
    },
    ["items", "pageInfo"],
    "Paginated official endpoints and relay stations.",
  );
}

function modelAccessChannelsListResultSchema() {
  return {
    allOf: [
      schemaRef("SdkWorkApiResponse"),
      objectSchema(
        {
          data: {
            allOf: [schemaRef("AppModelAccessChannelsPage")],
            description: "Model access channel page.",
          },
        },
        ["data"],
      ),
    ],
    description: "Model access channel list result.",
    "x-operation-id": "modelAccessChannels.list",
  };
}

function modelAccessChannelPresetSchema() {
  return objectSchema(
    {
      providerCode: safeCodeSchema(128, false),
      providerDisplayName: stringSchema(128, 1),
      protocol: safeCodeSchema(64, false),
      vendorCode: safeCodeSchema(64, false),
      vendorName: stringSchema(128, 1),
      channelName: stringSchema(128, 1),
      baseUrl: {
        type: "string",
        format: "uri",
        maxLength: 2048,
      },
      models: arraySchema(schemaRef("AppModelAccessChannelModel"), 512),
      defaultModelId: visibleAsciiStringSchema(128, true),
      sortOrder: int32Schema(0),
    },
    [
      "providerCode",
      "providerDisplayName",
      "protocol",
      "vendorCode",
      "vendorName",
      "channelName",
      "baseUrl",
      "models",
      "sortOrder",
    ],
    "Official provider preset used to create a model access channel.",
  );
}

function modelAccessChannelPresetsPageSchema() {
  return objectSchema(
    { items: arraySchema(schemaRef("AppModelAccessChannelPreset"), 128) },
    ["items"],
    "Official provider presets and their current public model lists.",
  );
}

function modelAccessChannelPresetsListResultSchema() {
  return {
    allOf: [
      schemaRef("SdkWorkApiResponse"),
      objectSchema(
        {
          data: {
            allOf: [schemaRef("AppModelAccessChannelPresetsPage")],
            description: "Official provider preset page.",
          },
        },
        ["data"],
      ),
    ],
    description: "Official provider preset list result.",
    "x-operation-id": "modelAccessChannelPresets.list",
  };
}

function mergeModelAccessChannelAppPath(document) {
  if (!document.components) {
    document.components = {};
  }
  if (!document.components.schemas) {
    document.components.schemas = {};
  }
  Object.assign(document.components.schemas, {
    AppModelAccessChannelModel: modelAccessChannelModelSchema(),
    AppModelAccessChannelModelInput: modelAccessChannelModelInputSchema(),
    AppModelAccessChannelOffering: modelAccessChannelOfferingSchema(),
    AppModelAccessChannelItem: modelAccessChannelItemSchema(),
    AppModelAccessChannelOfferingInput: modelAccessChannelOfferingInputSchema(),
    AppModelAccessChannelUpsertRequest: modelAccessChannelUpsertRequestSchema(),
    AppModelAccessChannelItemData: modelAccessChannelItemDataSchema(),
    AppModelAccessChannelsPage: modelAccessChannelsPageSchema(),
    ModelAccessChannelsListResult: modelAccessChannelsListResultSchema(),
    ModelAccessChannelUpsertResult: modelAccessChannelUpsertResultSchema(),
    AppModelAccessChannelPreset: modelAccessChannelPresetSchema(),
    AppModelAccessChannelPresetsPage: modelAccessChannelPresetsPageSchema(),
    ModelAccessChannelPresetsListResult: modelAccessChannelPresetsListResultSchema(),
  });

  const modelItemSchema = document.components.schemas.AppModelCatalogItem;
  if (modelItemSchema?.properties) {
    modelItemSchema.properties.supportedAgentProviderIds = arraySchema(
      safeCodeSchema(128, false),
      128,
      "Agent Config SPI provider ids that can consume this model.",
    );
    modelItemSchema.required = [
      ...new Set([
        ...(modelItemSchema.required ?? []),
        "supportedAgentProviderIds",
      ]),
    ];
    modelItemSchema.properties.usageScopes = arraySchema(
      enumStringSchema(["coding", "chat", "agent"], false),
      3,
      "Product usage scopes where the model can be applied, e.g. coding (code IDE/agent surfaces).",
    );
    modelItemSchema.properties.codingVisible = booleanSchema(
      "Whether the model is shown in code IDE surfaces. Defaults to true.",
    );
    modelItemSchema.required = [
      ...new Set([...(modelItemSchema.required ?? []), "usageScopes", "codingVisible"]),
    ];
  }

  const modelListOperation = document.paths?.["/app/v3/api/ai/models"]?.get;
  const operation = cloneJson(modelListOperation ?? {});
  operation.operationId = "modelAccessChannels.list";
  operation.summary = "List model access channels";
  operation.description =
    "List active official model endpoints and multi-vendor relay stations from the database. The response contains public routing metadata only and never exposes API keys.";
  operation.parameters = [
    ...offsetListQueryParameters(
      "Free-text search on channel code, name, description, base URL, vendor, and model.",
    ),
    {
      in: "query",
      name: "kind",
      required: false,
      schema: enumStringSchema(["official", "relay", "custom"], false),
      description: "Filter by official endpoint or relay station.",
    },
    {
      in: "query",
      name: "vendor_code",
      required: false,
      schema: safeCodeSchema(64, false),
      description: "Filter channels that expose at least one model from this vendor.",
    },
    {
      in: "query",
      name: "agent_provider_id",
      required: false,
      schema: safeCodeSchema(128, false),
      description: "Filter channels compatible with an Agent Config SPI provider.",
    },
  ];
  operation.responses = {
    "200": operationSuccessResponse(
      "#/components/schemas/ModelAccessChannelsListResult",
      "OK",
    ),
    ...keepProblemResponses(modelListOperation?.responses),
  };
  operation["x-contract-kind"] = "read";
  operation["x-read-sources"] = [
    "ai_resource",
    "ai_resource_group",
    "ai_resource_group_item",
    "ai_model_vendor",
    "ai_model",
  ];
  operation["x-write-tables"] = [];
  operation["x-file-targets"] = [];
  operation["x-route-scope"] = "public";
  operation["x-sdk-domain"] = "intelligence";
  operation["x-sdkwork-domain"] = "intelligence";
  operation["x-sdkwork-owner"] = "sdkwork-models";
  operation["x-sdkwork-api-authority"] = "sdkwork-models-app-api";
  operation["x-sdkwork-source-route-crate"] = "sdkwork-routes-models-catalog-app-api";
  operation["x-sdkwork-resource"] = "modelAccessChannels";
  operation["x-source-file"] =
    "../sdkwork-models/crates/sdkwork-models-catalog-service/src/api/app_models.rs";
  document.paths["/app/v3/api/ai/model_access_channels"] = { get: operation };
  const upsertOperation = cloneJson(operation);
  upsertOperation.operationId = "modelAccessChannels.upsert";
  upsertOperation.summary = "Create or update a model access channel";
  upsertOperation.description =
    "Create or update tenant-owned public metadata for an official endpoint or multi-vendor relay station. API keys are forbidden and must be configured through sdkwork-agents Config SPI.";
  upsertOperation.parameters = [
    {
      in: "path",
      name: "channelCode",
      required: true,
      schema: safeCodeSchema(96, false),
      description: "Stable tenant-owned model access channel code.",
    },
  ];
  upsertOperation.requestBody = jsonRequestBody(
    "#/components/schemas/AppModelAccessChannelUpsertRequest",
    "Public model access channel metadata without credentials.",
  );
  upsertOperation.responses = {
    "200": operationDataSuccessResponse(
      "#/components/schemas/AppModelAccessChannelItemData",
      "Model access channel saved.",
    ),
    ...keepProblemResponses(operation.responses),
  };
  upsertOperation["x-contract-kind"] = "update";
  upsertOperation["x-read-sources"] = ["ai_resource", "ai_resource_group_item"];
  upsertOperation["x-write-tables"] = [
    "ai_resource",
    "ai_resource_group",
    "ai_resource_group_item",
    "ops_audit_log",
  ];
  upsertOperation["x-sdkwork-resource"] = "modelAccessChannels";
  document.paths["/app/v3/api/ai/model_access_channels/{channelCode}"] = {
    put: upsertOperation,
  };
  const presetOperation = cloneJson(modelListOperation ?? {});
  presetOperation.operationId = "modelAccessChannelPresets.list";
  presetOperation.summary = "List official model access channel presets";
  presetOperation.description =
    "List official provider presets from the clawrouter overlay and the authoritative public model catalog. Presets contain public base URLs only; credentials are configured through sdkwork-agents Config SPI.";
  presetOperation.parameters = [];
  presetOperation.responses = {
    "200": operationSuccessResponse(
      "#/components/schemas/ModelAccessChannelPresetsListResult",
      "OK",
    ),
    ...keepProblemResponses(modelListOperation?.responses),
  };
  presetOperation["x-contract-kind"] = "read";
  presetOperation["x-read-sources"] = ["ai_model_vendor", "ai_model"];
  presetOperation["x-write-tables"] = [];
  presetOperation["x-file-targets"] = [];
  presetOperation["x-route-scope"] = "public";
  presetOperation["x-sdk-domain"] = "intelligence";
  presetOperation["x-sdkwork-domain"] = "intelligence";
  presetOperation["x-sdkwork-owner"] = "sdkwork-models";
  presetOperation["x-sdkwork-api-authority"] = "sdkwork-models-app-api";
  presetOperation["x-sdkwork-source-route-crate"] =
    "sdkwork-routes-models-catalog-app-api";
  presetOperation["x-sdkwork-resource"] = "modelAccessChannelPresets";
  presetOperation["x-source-file"] =
    "../sdkwork-models/crates/sdkwork-models-catalog-service/src/api/app_models.rs";
  document.paths["/app/v3/api/ai/model_access_channel_presets"] = {
    get: presetOperation,
  };
  return document;
}

function modelCatalogSyncResultSchema() {
  return {
    type: "object",
    additionalProperties: false,
    description: "Catalog sync result returned by models.sync.",
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

  const syncPath = document.paths?.["/backend/v3/api/ai/models/sync"];
  const successSchema = syncPath?.post?.responses?.["200"]?.content?.["application/json"]?.schema;
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

function operationSuccessResponse(schemaRef, description) {
  return {
    description,
    content: {
      "application/json": {
        schema: { $ref: schemaRef },
      },
    },
  };
}

function operationDataSuccessResponse(schemaRef, description) {
  return {
    description,
    content: {
      "application/json": {
        schema: {
          allOf: [
            { $ref: "#/components/schemas/SdkWorkApiResponse" },
            {
              type: "object",
              required: ["data"],
              properties: {
                data: {
                  allOf: [{ $ref: schemaRef }],
                },
              },
            },
          ],
        },
      },
    },
  };
}

function operationResourceItemSuccessResponse(itemSchemaName, description) {
  return {
    description,
    content: {
      "application/json": {
        schema: {
          allOf: [
            { $ref: "#/components/schemas/SdkWorkApiResponse" },
            {
              type: "object",
              additionalProperties: false,
              required: ["data"],
              properties: {
                data: {
                  type: "object",
                  additionalProperties: false,
                  required: ["item"],
                  properties: {
                    item: schemaRef(itemSchemaName),
                  },
                },
              },
            },
          ],
        },
      },
    },
  };
}

function jsonRequestBody(schemaRef, description) {
  return {
    required: true,
    description,
    content: {
      "application/json": {
        schema: { $ref: schemaRef },
      },
    },
  };
}

function schemaRef(name) {
  return { $ref: `#/components/schemas/${name}` };
}

function objectSchema(properties, required = [], description = undefined) {
  return {
    type: "object",
    additionalProperties: false,
    ...(description ? { description } : {}),
    properties,
    ...(required.length > 0 ? { required } : {}),
  };
}

function stringSchema(maxLength, minLength = undefined, description = undefined) {
  return {
    type: "string",
    ...(minLength === undefined ? {} : { minLength }),
    maxLength,
    ...(description ? { description } : {}),
  };
}

function nullableStringSchema(maxLength, description = undefined) {
  return {
    type: ["string", "null"],
    maxLength,
    ...(description ? { description } : {}),
  };
}

function safeCodeSchema(maxLength, nullable = false, description = undefined) {
  return {
    type: nullable ? ["string", "null"] : "string",
    maxLength,
    pattern: "^[A-Za-z0-9._:-]+$",
    ...(description ? { description } : {}),
  };
}

function optionalSafeCodeSchema(maxLength) {
  return {
    type: ["string", "null"],
    maxLength,
    pattern: "^[A-Za-z0-9._:-]*$",
  };
}

function visibleAsciiStringSchema(maxLength, nullable = true, description = undefined) {
  return {
    type: nullable ? ["string", "null"] : "string",
    maxLength,
    pattern: "^[\\x20-\\x7E]*$",
    ...(description ? { description } : {}),
  };
}

function optionalPositiveInt64StringSchema() {
  return {
    type: ["string", "null"],
    pattern: "^[1-9][0-9]*$",
  };
}

function int64StringSchema(nullable = true, minimum = 0, description = undefined) {
  const pattern = minimum > 0 ? "^[1-9][0-9]*$" : "^(0|[1-9][0-9]*)$";
  return {
    type: nullable ? ["string", "null"] : "string",
    format: "int64",
    pattern,
    "x-sdkwork-int64-string": true,
    "x-sdkwork-rust-type": "i64",
    ...(description ? { description } : {}),
  };
}

function int32Schema(minimum = undefined, maximum = undefined, description = undefined) {
  return {
    type: "integer",
    format: "int32",
    ...(minimum === undefined ? {} : { minimum }),
    ...(maximum === undefined ? {} : { maximum }),
    ...(description ? { description } : {}),
  };
}

function decimalStringSchema(nullable = true, description = undefined) {
  return {
    type: nullable ? ["string", "null"] : "string",
    pattern: "^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$",
    "x-sdkwork-decimal-string": true,
    ...(description ? { description } : {}),
  };
}

function booleanSchema(description = undefined) {
  return {
    type: "boolean",
    ...(description ? { description } : {}),
  };
}

function arraySchema(items, maxItems = undefined, description = undefined) {
  return {
    type: "array",
    items,
    ...(maxItems === undefined ? {} : { maxItems }),
    ...(description ? { description } : {}),
  };
}

function enumStringSchema(values, nullable = false, description = undefined) {
  return {
    type: nullable ? ["string", "null"] : "string",
    enum: nullable ? [...values, null] : values,
    ...(description ? { description } : {}),
  };
}

function metadataStringArraySchema(description) {
  return arraySchema(stringSchema(128, 1), 128, description);
}

function adminModelVendorCreateRequestSchema() {
  return objectSchema(
    {
      vendorCode: {
        ...optionalSafeCodeSchema(64),
        description: "Optional vendor code. The backend generates a code when omitted.",
      },
      name: stringSchema(128, 1, "Vendor display name."),
      status: enumStringSchema(["active", "disabled", "inactive"], true),
      color: nullableStringSchema(32),
      description: nullableStringSchema(512),
    },
    ["name"],
    "Request body for creating an AI model vendor.",
  );
}

function adminAiModelRegionPriceRequestSchema() {
  return objectSchema(
    {
      regionCode: safeCodeSchema(32, false),
      currency: safeCodeSchema(16, false),
      priceIn: decimalStringSchema(true),
      priceOut: decimalStringSchema(true),
      cacheReadPrice: decimalStringSchema(true),
      cacheWritePrice: decimalStringSchema(true),
    },
    ["regionCode", "currency"],
    "Regional model pricing input. Decimal wire fields are encoded as strings.",
  );
}

function adminAiModelCreateRequestSchema() {
  return objectSchema(
    {
      vendorId: int64StringSchema(false, 1),
      model: visibleAsciiStringSchema(128, false),
      displayName: nullableStringSchema(128),
      type: enumStringSchema(["chat", "image", "embedding", "audio", "video", "rerank", "moderation"], false),
      regionPrices: arraySchema(schemaRef("AdminAiModelRegionPriceRequest"), 200),
      contextTokens: int64StringSchema(false, 1),
      description: nullableStringSchema(2048),
      modalities: metadataStringArraySchema("Supported model modalities."),
      inputModalities: metadataStringArraySchema("Supported input modalities."),
      outputModalities: metadataStringArraySchema("Supported output modalities."),
      apiFormat: safeCodeSchema(64, true),
      capabilityIntro: nullableStringSchema(2048),
      limitations: metadataStringArraySchema("Model limitations."),
      supportedLanguages: metadataStringArraySchema("Supported natural languages."),
      useCases: metadataStringArraySchema("Recommended use cases."),
      trainingDataCutoff: nullableStringSchema(64),
      maxOutputTokens: int64StringSchema(true, 1),
      supportsStreaming: booleanSchema(),
      supportsTools: booleanSchema(),
      supportsJsonSchema: booleanSchema(),
      usageScopes: arraySchema(
        enumStringSchema(["coding", "chat", "agent"], false),
        3,
        "Product usage scopes where the model can be applied.",
      ),
      codingVisible: booleanSchema("Whether the model is shown in code IDE surfaces."),
      releaseStage: int32Schema(1, 3),
      shelfState: int32Schema(1, 3),
      routingState: int32Schema(0, 2),
      replacementModel: visibleAsciiStringSchema(128, true),
    },
    ["vendorId", "model", "type", "regionPrices", "contextTokens"],
    "Request body for creating an AI model catalog entry.",
  );
}

function adminAiModelUpdateRequestSchema() {
  return objectSchema(
    {
      vendorId: int64StringSchema(true, 1),
      model: visibleAsciiStringSchema(128, true),
      displayName: nullableStringSchema(128),
      type: enumStringSchema(["chat", "image", "embedding", "audio", "video", "rerank", "moderation"], true),
      regionPrices: arraySchema(schemaRef("AdminAiModelRegionPriceRequest"), 200),
      status: enumStringSchema(["active", "disabled", "inactive"], true),
      contextTokens: int64StringSchema(true, 1),
      description: nullableStringSchema(2048),
      modalities: metadataStringArraySchema("Supported model modalities."),
      inputModalities: metadataStringArraySchema("Supported input modalities."),
      outputModalities: metadataStringArraySchema("Supported output modalities."),
      apiFormat: safeCodeSchema(64, true),
      capabilityIntro: nullableStringSchema(2048),
      limitations: metadataStringArraySchema("Model limitations."),
      supportedLanguages: metadataStringArraySchema("Supported natural languages."),
      useCases: metadataStringArraySchema("Recommended use cases."),
      trainingDataCutoff: nullableStringSchema(64),
      maxOutputTokens: int64StringSchema(true, 1),
      supportsStreaming: booleanSchema(),
      supportsTools: booleanSchema(),
      supportsJsonSchema: booleanSchema(),
      usageScopes: arraySchema(
        enumStringSchema(["coding", "chat", "agent"], false),
        3,
        "Product usage scopes where the model can be applied.",
      ),
      codingVisible: booleanSchema("Whether the model is shown in code IDE surfaces."),
      releaseStage: int32Schema(1, 3),
      shelfState: int32Schema(1, 3),
      routingState: int32Schema(0, 2),
      replacementModel: visibleAsciiStringSchema(128, true),
    },
    [],
    "Request body for patching an AI model catalog entry.",
  );
}

function adminModelCatalogSyncRequestSchema() {
  return objectSchema(
    {
      source: safeCodeSchema(64, true),
      mode: enumStringSchema(
        ["official_refresh", "vendor_refresh", "catalog_version_refresh", "dry_run"],
        false,
      ),
      vendorCodes: arraySchema(safeCodeSchema(64, false), 200),
      force: booleanSchema(),
      catalogRoot: nullableStringSchema(512),
      catalogVersion: nullableStringSchema(128),
    },
    [],
    "Request body for refreshing the AI model catalog.",
  );
}

function modelMappingsPageSchema() {
  return objectSchema(
    {
      items: arraySchema(
        {
          type: "object",
          additionalProperties: true,
          description: "Model mapping rule response item.",
        },
        200,
        "Model mapping rule items.",
      ),
      pageInfo: {
        allOf: [schemaRef("PageInfo")],
        description: "Offset pagination metadata for model mappings.",
      },
    },
    ["items", "pageInfo"],
    "Paginated model mapping rules returned by modelMappings.list.",
  );
}

function adminModelMappingRuleBindingInputSchema() {
  return objectSchema(
    {
      id: int64StringSchema(true, 1),
      bindingType: safeCodeSchema(64, false),
      bindingId: int64StringSchema(true, 1),
      bindingCode: safeCodeSchema(128, true),
      bindingName: nullableStringSchema(128),
      enabled: booleanSchema(),
    },
    ["bindingType"],
    "Model mapping rule binding input.",
  );
}

function adminModelMappingRuleItemInputSchema(requiredForCreate) {
  return objectSchema(
    {
      id: int64StringSchema(true, 1),
      sourceModel: visibleAsciiStringSchema(256, true),
      sourceCatalogKey: visibleAsciiStringSchema(512, true),
      targetModel: visibleAsciiStringSchema(256, true),
      targetCatalogKey: visibleAsciiStringSchema(512, true),
      targetProviderModel: visibleAsciiStringSchema(256, true),
      targetProviderNativeModel: visibleAsciiStringSchema(256, true),
      enabled: booleanSchema(),
    },
    requiredForCreate ? ["sourceModel", "targetModel"] : [],
    "Model mapping rule item input.",
  );
}

function adminModelMappingCreateRequestSchema() {
  return objectSchema(
    {
      sourceVendorId: int64StringSchema(true, 1),
      sourceVendorCode: safeCodeSchema(64, false),
      targetVendorId: int64StringSchema(true, 1),
      targetVendorCode: safeCodeSchema(64, false),
      mappingMode: enumStringSchema(["alias"], true),
      matchType: enumStringSchema(["exact"], true),
      enabled: booleanSchema(),
      bindings: arraySchema(schemaRef("AdminModelMappingRuleBindingInput"), 200),
      mappingItems: arraySchema(schemaRef("AdminModelMappingRuleItemInput"), 1000),
    },
    ["sourceVendorCode", "targetVendorCode", "bindings", "mappingItems"],
    "Request body for creating a model mapping rule.",
  );
}

function adminModelMappingUpdateRequestSchema() {
  return objectSchema(
    {
      sourceVendorId: int64StringSchema(true, 1),
      sourceVendorCode: safeCodeSchema(64, true),
      targetVendorId: int64StringSchema(true, 1),
      targetVendorCode: safeCodeSchema(64, true),
      mappingMode: enumStringSchema(["alias"], true),
      matchType: enumStringSchema(["exact"], true),
      enabled: booleanSchema(),
      bindings: arraySchema(schemaRef("AdminModelMappingRuleBindingInput"), 200),
      mappingItems: arraySchema(schemaRef("AdminModelMappingRuleItemInput"), 1000),
    },
    [],
    "Request body for updating a model mapping rule.",
  );
}

function adminModelMappingResolveRequestSchema() {
  return {
    type: "object",
    additionalProperties: false,
    description: "Request body for resolving a model mapping rule without mutating catalog state.",
    properties: {
      sourceModel: {
        type: "string",
        minLength: 1,
        maxLength: 256,
        description: "Source model identifier to resolve.",
      },
      vendorCode: {
        ...optionalSafeCodeSchema(64),
        description: "Optional source vendor code scope.",
      },
      channelId: {
        ...optionalPositiveInt64StringSchema(),
        description: "Optional channel id scope encoded as int64 string.",
      },
      channelCode: {
        ...optionalSafeCodeSchema(64),
        description: "Optional channel code scope.",
      },
      providerAccountId: {
        ...optionalPositiveInt64StringSchema(),
        description: "Optional provider account id scope encoded as int64 string.",
      },
      providerAccountCode: {
        ...optionalSafeCodeSchema(128),
        description: "Optional provider account binding code scope.",
      },
    },
    required: ["sourceModel"],
  };
}

function modelRankingRefreshTriggerRequestSchema() {
  return objectSchema(
    {
      rankScope: safeCodeSchema(80, true),
      snapshotPeriod: enumStringSchema(["hourly", "daily", "weekly", "monthly"], true),
      limit: int64StringSchema(true, 1),
      lookbackDays: int64StringSchema(true, 1),
      refreshIntervalSeconds: int64StringSchema(true, 1),
      cacheMaxAgeSeconds: int64StringSchema(true, 1),
    },
    [],
    "Request body for manually refreshing model ranking snapshots.",
  );
}

function modelRankingRefreshTriggerResponseSchema() {
  return objectSchema(
    {
      triggered: booleanSchema(),
      status: enumStringSchema(["succeeded", "empty"], false),
      tenantId: int64StringSchema(false, 0),
      organizationId: int64StringSchema(false, 0),
      rankScope: safeCodeSchema(80, false),
      snapshotDate: stringSchema(32, 1),
      snapshotPeriod: enumStringSchema(["hourly", "daily", "weekly", "monthly"], false),
      windowStart: stringSchema(64, 1),
      windowEnd: stringSchema(64, 1),
      generatedCount: int64StringSchema(false, 0),
      sourceCount: int64StringSchema(false, 0),
      refreshIntervalSeconds: int64StringSchema(false, 1),
      cacheMaxAgeSeconds: int64StringSchema(false, 1),
      nextRefreshAt: stringSchema(64, 1),
    },
    [
      "triggered",
      "status",
      "tenantId",
      "organizationId",
      "rankScope",
      "snapshotDate",
      "snapshotPeriod",
      "windowStart",
      "windowEnd",
      "generatedCount",
      "sourceCount",
      "refreshIntervalSeconds",
      "cacheMaxAgeSeconds",
      "nextRefreshAt",
    ],
    "Response data returned after manually refreshing model ranking snapshots.",
  );
}

function adminAiResourceMemberInputSchema() {
  return objectSchema(
    {
      memberResourceCode: safeCodeSchema(192, false),
      memberRole: enumStringSchema(["included", "optional", "fallback"], true),
      required: booleanSchema(),
      sortOrder: int64StringSchema(true, 0),
    },
    ["memberResourceCode"],
    "AI resource composition member input.",
  );
}

function adminAiResourceCreateRequestSchema() {
  return objectSchema(
    {
      resourceCode: safeCodeSchema(192, false),
      resourceType: enumStringSchema(AI_RESOURCE_TYPES, false),
      displayName: stringSchema(128, 1),
      vendorCode: safeCodeSchema(64, true),
      modalityCode: safeCodeSchema(64, true),
      apiEndpointCode: safeCodeSchema(128, true),
      catalogKey: visibleAsciiStringSchema(256, true),
      model: visibleAsciiStringSchema(128, true),
      providerNativeModel: visibleAsciiStringSchema(256, true),
      accessChannelKind: enumStringSchema(["official", "relay", "custom"], true),
      baseUrl: visibleAsciiStringSchema(2048, true),
      defaultVendorCode: safeCodeSchema(64, true),
      defaultModelId: visibleAsciiStringSchema(128, true),
      supportedAgentProviderIds: arraySchema(safeCodeSchema(128, false), 128),
      description: nullableStringSchema(512),
      compositionMode: enumStringSchema(["single", "any", "all"], true),
      status: enumStringSchema(["active", "disabled", "inactive"], true),
      sortOrder: int64StringSchema(true, 0),
      members: arraySchema(schemaRef("AdminAiResourceMemberInput"), 512),
    },
    ["resourceCode", "resourceType", "displayName"],
    "Request body for creating an AI resource.",
  );
}

function adminAiResourceUpdateRequestSchema() {
  return objectSchema(
    {
      resourceCode: safeCodeSchema(192, true),
      resourceType: enumStringSchema(AI_RESOURCE_TYPES, true),
      displayName: nullableStringSchema(128),
      vendorCode: safeCodeSchema(64, true),
      modalityCode: safeCodeSchema(64, true),
      apiEndpointCode: safeCodeSchema(128, true),
      catalogKey: visibleAsciiStringSchema(256, true),
      model: visibleAsciiStringSchema(128, true),
      providerNativeModel: visibleAsciiStringSchema(256, true),
      accessChannelKind: enumStringSchema(["official", "relay", "custom"], true),
      baseUrl: visibleAsciiStringSchema(2048, true),
      defaultVendorCode: safeCodeSchema(64, true),
      defaultModelId: visibleAsciiStringSchema(128, true),
      supportedAgentProviderIds: arraySchema(safeCodeSchema(128, false), 128),
      description: nullableStringSchema(512),
      compositionMode: enumStringSchema(["single", "any", "all"], true),
      status: enumStringSchema(["active", "disabled", "inactive"], true),
      sortOrder: int64StringSchema(true, 0),
      members: arraySchema(schemaRef("AdminAiResourceMemberInput"), 512),
    },
    [],
    "Request body for updating an AI resource.",
  );
}

function adminAiResourceMemberItemSchema() {
  return objectSchema(
    {
      parentResourceCode: safeCodeSchema(192, false),
      memberResourceCode: safeCodeSchema(192, false),
      memberRole: enumStringSchema(["included", "optional", "fallback"], false),
      required: booleanSchema(),
      sortOrder: int64StringSchema(true, 0),
    },
    ["parentResourceCode", "memberResourceCode", "memberRole", "required"],
    "AI resource composition member returned by the management API.",
  );
}

function adminAiResourceItemSchema() {
  return objectSchema(
    {
      id: int64StringSchema(false, 1),
      resourceCode: safeCodeSchema(192, false),
      resourceType: enumStringSchema(AI_RESOURCE_TYPES, false),
      displayName: stringSchema(128, 1),
      vendorCode: safeCodeSchema(64, true),
      modalityCode: safeCodeSchema(64, true),
      apiEndpointCode: safeCodeSchema(128, true),
      catalogKey: visibleAsciiStringSchema(256, true),
      model: visibleAsciiStringSchema(128, true),
      providerNativeModel: visibleAsciiStringSchema(256, true),
      accessChannelKind: enumStringSchema(["official", "relay", "custom"], true),
      baseUrl: visibleAsciiStringSchema(2048, true),
      defaultVendorCode: safeCodeSchema(64, true),
      defaultModelId: visibleAsciiStringSchema(128, true),
      supportedAgentProviderIds: arraySchema(safeCodeSchema(128, false), 128),
      description: nullableStringSchema(512),
      capability: safeCodeSchema(128, true),
      capabilities: arraySchema(safeCodeSchema(128, false), 128),
      compositionMode: enumStringSchema(["single", "any", "all"], false),
      status: enumStringSchema(["active", "disabled", "inactive"], false),
      sortOrder: int64StringSchema(true, 0),
      members: arraySchema(schemaRef("AdminAiResourceMemberItem"), 512),
    },
    [
      "id",
      "resourceCode",
      "resourceType",
      "displayName",
      "capabilities",
      "compositionMode",
      "status",
      "members",
    ],
    "AI resource returned by the management API.",
  );
}

function adminAiResourceGroupItemSchema() {
  return objectSchema(
    {
      id: int64StringSchema(false, 1),
      groupCode: safeCodeSchema(128, false),
      groupName: stringSchema(128, 1),
      groupType: enumStringSchema(["api_group"], false),
      selectionMode: enumStringSchema(["manual", "all", "any", "dynamic_all_api"], false),
      description: nullableStringSchema(512),
      vendorCodes: arraySchema(safeCodeSchema(64, false), 128),
      capability: safeCodeSchema(128, true),
      capabilities: arraySchema(safeCodeSchema(128, false), 128),
      sortOrder: int64StringSchema(true, 0),
      status: enumStringSchema(["active", "disabled", "inactive"], false),
      resourceCount: int64StringSchema(false, 0),
      dynamic: booleanSchema(),
    },
    [
      "id",
      "groupCode",
      "groupName",
      "groupType",
      "selectionMode",
      "vendorCodes",
      "capabilities",
      "status",
      "resourceCount",
      "dynamic",
    ],
    "AI resource group returned by the management API.",
  );
}

function adminAiResourceGroupResourceItemSchema() {
  return objectSchema(
    {
      id: int64StringSchema(false, 1),
      resourceCode: safeCodeSchema(192, false),
      resourceType: enumStringSchema(AI_RESOURCE_TYPES, false),
      displayName: stringSchema(128, 1),
      vendorCode: safeCodeSchema(64, true),
      modalityCode: safeCodeSchema(64, true),
      apiEndpointCode: safeCodeSchema(128, true),
      catalogKey: visibleAsciiStringSchema(256, true),
      model: visibleAsciiStringSchema(128, true),
      providerNativeModel: visibleAsciiStringSchema(256, true),
      status: enumStringSchema(["active", "disabled", "inactive"], false),
      sortOrder: int64StringSchema(true, 0),
      memberRole: enumStringSchema(["included", "optional", "fallback"], false),
    },
    ["id", "resourceCode", "resourceType", "displayName", "status", "memberRole"],
    "AI resource projected through a resource-group membership.",
  );
}

function offsetPageSchema(itemSchemaName, description) {
  return objectSchema(
    {
      items: arraySchema(schemaRef(itemSchemaName), 200),
      pageInfo: {
        allOf: [schemaRef("OffsetPageInfo")],
        description: "Offset pagination metadata.",
      },
    },
    ["items", "pageInfo"],
    description,
  );
}

function offsetPageInfoSchema() {
  return objectSchema(
    {
      mode: enumStringSchema(["offset"], false),
      page: int32Schema(1),
      pageSize: int32Schema(1, 200),
      totalItems: int64StringSchema(false, 0),
      totalPages: int32Schema(0),
      hasMore: booleanSchema(),
    },
    ["mode", "page", "pageSize", "totalItems", "totalPages", "hasMore"],
    "Required metadata for an offset-paginated SDKWork list response.",
  );
}

function adminAiResourceGroupMemberInputSchema() {
  return objectSchema(
    {
      resourceCode: safeCodeSchema(192, false),
      itemRole: enumStringSchema(["included", "optional", "fallback"], true),
      sortOrder: int64StringSchema(true, 0),
    },
    ["resourceCode"],
    "AI resource group member input.",
  );
}

function adminAiResourceGroupMemberUpdateRequestSchema() {
  return objectSchema(
    {
      itemRole: enumStringSchema(["included", "optional", "fallback"], true),
      sortOrder: int64StringSchema(true, 0),
    },
    [],
    "Full request body for assigning or updating one AI resource-group member.",
  );
}

function adminAiResourceGroupCreateRequestSchema() {
  return objectSchema(
    {
      groupCode: safeCodeSchema(128, false),
      groupName: stringSchema(128, 1),
      groupType: enumStringSchema(["api_group"], true),
      selectionMode: enumStringSchema(["manual", "all", "any", "dynamic_all_api"], true),
      description: nullableStringSchema(512),
      sortOrder: int64StringSchema(true, 0),
      status: enumStringSchema(["active", "disabled", "inactive"], true),
      members: arraySchema(schemaRef("AdminAiResourceGroupMemberInput"), 512),
    },
    ["groupCode", "groupName"],
    "Request body for creating an AI resource group.",
  );
}

function adminAiResourceGroupUpdateRequestSchema() {
  return objectSchema(
    {
      groupCode: safeCodeSchema(128, true),
      groupName: nullableStringSchema(128),
      groupType: enumStringSchema(["api_group"], true),
      selectionMode: enumStringSchema(["manual", "all", "any", "dynamic_all_api"], true),
      description: nullableStringSchema(512),
      sortOrder: int64StringSchema(true, 0),
      status: enumStringSchema(["active", "disabled", "inactive"], true),
      members: arraySchema(schemaRef("AdminAiResourceGroupMemberInput"), 512),
    },
    [],
    "Request body for updating an AI resource group.",
  );
}

function keepProblemResponses(responses) {
  return Object.fromEntries(
    Object.entries(responses || {}).filter(([statusCode]) => !String(statusCode).startsWith("2")),
  );
}

function standardizeCreateOperation(operation) {
  if (!operation?.responses) {
    return;
  }
  operation.responses = {
    "201": operationSuccessResponse(
      "#/components/schemas/SdkWorkResourceResponse",
      "Created",
    ),
    ...keepProblemResponses(operation.responses),
  };
}

function standardizeDeleteOperation(operation) {
  if (!operation) {
    return;
  }
  operation.responses = {
    "204": {
      description: "No Content",
    },
    ...keepProblemResponses(operation.responses),
  };
}

function standardizeUpdateOperation(operation) {
  if (!operation?.responses) {
    return;
  }
  operation.responses = {
    "200": operationSuccessResponse(
      "#/components/schemas/SdkWorkResourceResponse",
      "OK",
    ),
    ...keepProblemResponses(operation.responses),
  };
}

function recordDataSuccessResponse(description) {
  return {
    description,
    content: {
      "application/json": {
        schema: {
          allOf: [
            { $ref: "#/components/schemas/SdkWorkApiResponse" },
            {
              type: "object",
              required: ["data"],
              properties: {
                data: {
                  type: "object",
                  additionalProperties: true,
                },
              },
            },
          ],
        },
      },
    },
  };
}

function ensureBackendWriteSchemas(document) {
  if (!document.components) {
    document.components = {};
  }
  if (!document.components.schemas) {
    document.components.schemas = {};
  }
  Object.assign(document.components.schemas, {
    AdminModelVendorCreateRequest: adminModelVendorCreateRequestSchema(),
    AdminAiModelRegionPriceRequest: adminAiModelRegionPriceRequestSchema(),
    AdminAiModelCreateRequest: adminAiModelCreateRequestSchema(),
    AdminAiModelUpdateRequest: adminAiModelUpdateRequestSchema(),
    AdminModelCatalogSyncRequest: adminModelCatalogSyncRequestSchema(),
    AdminModelMappingRuleBindingInput: adminModelMappingRuleBindingInputSchema(),
    AdminModelMappingRuleItemInput: adminModelMappingRuleItemInputSchema(true),
    AdminModelMappingCreateRequest: adminModelMappingCreateRequestSchema(),
    AdminModelMappingUpdateRequest: adminModelMappingUpdateRequestSchema(),
    AdminModelMappingResolveRequest: adminModelMappingResolveRequestSchema(),
    ModelMappingsPage: modelMappingsPageSchema(),
    ModelRankingRefreshTriggerRequest: modelRankingRefreshTriggerRequestSchema(),
    ModelRankingRefreshTriggerResponse: modelRankingRefreshTriggerResponseSchema(),
    AdminAiResourceMemberInput: adminAiResourceMemberInputSchema(),
    OffsetPageInfo: offsetPageInfoSchema(),
    AdminAiResourceMemberItem: adminAiResourceMemberItemSchema(),
    AdminAiResourceItem: adminAiResourceItemSchema(),
    AiResourcesPage: offsetPageSchema(
      "AdminAiResourceItem",
      "Paginated AI resources returned by resources.list.",
    ),
    AdminAiResourceCreateRequest: adminAiResourceCreateRequestSchema(),
    AdminAiResourceUpdateRequest: adminAiResourceUpdateRequestSchema(),
    AdminAiResourceGroupItem: adminAiResourceGroupItemSchema(),
    AiResourceGroupsPage: offsetPageSchema(
      "AdminAiResourceGroupItem",
      "Paginated AI resource groups returned by resourceGroups.list.",
    ),
    AdminAiResourceGroupResourceItem: adminAiResourceGroupResourceItemSchema(),
    AiResourceGroupResourcesPage: offsetPageSchema(
      "AdminAiResourceGroupResourceItem",
      "Paginated resources returned by resourceGroups.resources.list.",
    ),
    AdminAiResourceGroupMemberInput: adminAiResourceGroupMemberInputSchema(),
    AdminAiResourceGroupMemberUpdateRequest:
      adminAiResourceGroupMemberUpdateRequestSchema(),
    AdminAiResourceGroupCreateRequest: adminAiResourceGroupCreateRequestSchema(),
    AdminAiResourceGroupUpdateRequest: adminAiResourceGroupUpdateRequestSchema(),
  });
}

function operationAt(document, pathKey, method) {
  return document.paths?.[pathKey]?.[method];
}

function setRequestBody(document, pathKey, method, schemaName, description) {
  const operation = operationAt(document, pathKey, method);
  if (!operation) {
    return;
  }
  operation.requestBody = jsonRequestBody(
    `#/components/schemas/${schemaName}`,
    description,
  );
}

function standardizeDataOperation(operation, schemaName, description) {
  if (!operation?.responses) {
    return;
  }
  operation.responses = {
    "200": operationDataSuccessResponse(
      `#/components/schemas/${schemaName}`,
      description,
    ),
    ...keepProblemResponses(operation.responses),
  };
}

function standardizeRecordDataOperation(operation, description) {
  if (!operation?.responses) {
    return;
  }
  operation.responses = {
    "200": recordDataSuccessResponse(description),
    ...keepProblemResponses(operation.responses),
  };
}

function standardizeModelMappingsListOperation(document) {
  const operation = operationAt(document, "/backend/v3/api/ai/model_mappings", "get");
  if (!operation) {
    return;
  }
  operation.operationId = "modelMappings.list";
  operation.summary = operation.summary || "List model mappings";
  operation.description =
    "List model mapping rules with SQL-backed offset pagination and optional binding/vendor/channel/search filters.";
  operation.parameters = [
    {
      in: "query",
      name: "page",
      required: false,
      schema: { type: "integer", minimum: 1, default: 1 },
      description: "Offset pagination page number.",
    },
    {
      in: "query",
      name: "page_size",
      required: false,
      schema: { type: "integer", minimum: 1, maximum: 200, default: 20 },
      description: "Offset pagination page size.",
    },
    {
      in: "query",
      name: "binding_type",
      required: false,
      schema: {
        type: "string",
        enum: [
          "global",
          "vendor",
          "channel_group",
          "channel",
          "provider_account",
          "site",
          "site_service",
        ],
      },
      description: "Filter mappings by binding type.",
    },
    {
      in: "query",
      name: "vendor_code",
      required: false,
      schema: safeCodeSchema(64, false),
      description: "Filter mappings by source, target, or binding vendor code.",
    },
    {
      in: "query",
      name: "channel_id",
      required: false,
      schema: int64StringSchema(false, 1),
      description: "Filter mappings by channel binding id encoded as int64 string.",
    },
    {
      in: "query",
      name: "channel_code",
      required: false,
      schema: safeCodeSchema(64, false),
      description: "Filter mappings by channel binding code.",
    },
    {
      in: "query",
      name: "q",
      required: false,
      schema: stringSchema(128, 1),
      description: "Free-text search on vendor codes, mapping items, and binding labels.",
    },
  ];
  standardizeDataOperation(operation, "ModelMappingsPage", "OK");
}

function offsetListQueryParameters(searchDescription) {
  return [
    {
      in: "query",
      name: "page",
      required: false,
      schema: { type: "integer", minimum: 1, default: 1 },
      description: "Offset pagination page number.",
    },
    {
      in: "query",
      name: "page_size",
      required: false,
      schema: { type: "integer", minimum: 1, maximum: 200, default: 20 },
      description: "Offset pagination page size.",
    },
    {
      in: "query",
      name: "q",
      required: false,
      schema: { type: "string", maxLength: 256 },
      description: searchDescription,
    },
  ];
}

function standardizeAiResourceListOperation(
  document,
  pathKey,
  schemaName,
  description,
  searchDescription,
  additionalQueryParameters = [],
) {
  const operation = operationAt(document, pathKey, "get");
  if (!operation) {
    return;
  }
  const pathParameters = (operation.parameters ?? []).filter(
    (parameter) => parameter?.in === "path",
  );
  operation.description = description;
  operation.parameters = [
    ...pathParameters,
    ...offsetListQueryParameters(searchDescription),
    ...additionalQueryParameters,
  ];
  operation["x-sdkwork-pagination-mode"] = "offset";
  standardizeDataOperation(operation, schemaName, "OK");
}

function standardizeAiResourceListOperations(document) {
  standardizeAiResourceListOperation(
    document,
    "/backend/v3/api/ai/resources",
    "AiResourcesPage",
    "List tenant-visible AI resources with SQL-backed offset pagination. The default stable sort is tenant override, sortOrder, then id; page_size defaults to 20 and is capped at 200. This is a P1 administrative list.",
    "Free-text search on resource code, display name, vendor, modality, endpoint, catalog key, and model fields.",
    [
      {
        in: "query",
        name: "resource_type",
        required: false,
        schema: {
          type: "string",
          enum: AI_RESOURCE_TYPES,
        },
        description: "Filter by the canonical AI resource type.",
      },
    ],
  );
  standardizeAiResourceListOperation(
    document,
    "/backend/v3/api/ai/resource_groups",
    "AiResourceGroupsPage",
    "List tenant-visible AI resource groups with SQL-backed offset pagination. The default stable sort is tenant override, sortOrder, then id; page_size defaults to 20 and is capped at 200. This is a P1 administrative list.",
    "Free-text search on resource-group code, name, and description.",
  );
  standardizeAiResourceListOperation(
    document,
    "/backend/v3/api/ai/resource_groups/{groupIdOrCode}/resources",
    "AiResourceGroupResourcesPage",
    "List resources in a tenant-visible AI resource group with SQL-backed offset pagination. The default stable sort is membership or resource sortOrder, then id; page_size defaults to 20 and is capped at 200. This is a P1 administrative list.",
    "Free-text search on resource code, display name, vendor, modality, endpoint, catalog key, and model fields.",
  );
}

function standardizeBackendWriteOperations(document) {
  ensureBackendWriteSchemas(document);

  setRequestBody(
    document,
    "/backend/v3/api/ai/model_vendors",
    "post",
    "AdminModelVendorCreateRequest",
    "Model vendor create request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/models",
    "post",
    "AdminAiModelCreateRequest",
    "AI model create request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/models/{modelId}",
    "patch",
    "AdminAiModelUpdateRequest",
    "AI model update request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/models/sync",
    "post",
    "AdminModelCatalogSyncRequest",
    "Model catalog refresh request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/model_mappings",
    "post",
    "AdminModelMappingCreateRequest",
    "Model mapping create request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/model_mappings/{mappingId}",
    "patch",
    "AdminModelMappingUpdateRequest",
    "Model mapping update request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/model_mappings/resolve",
    "post",
    "AdminModelMappingResolveRequest",
    "Model mapping resolve request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/model_rankings/refresh",
    "post",
    "ModelRankingRefreshTriggerRequest",
    "Model ranking refresh request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/resources",
    "post",
    "AdminAiResourceCreateRequest",
    "AI resource create request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/resources/{resourceId}",
    "put",
    "AdminAiResourceUpdateRequest",
    "AI resource update request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/resource_groups",
    "post",
    "AdminAiResourceGroupCreateRequest",
    "AI resource group create request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/resource_groups/{groupId}",
    "patch",
    "AdminAiResourceGroupUpdateRequest",
    "AI resource group update request.",
  );
  setRequestBody(
    document,
    "/backend/v3/api/ai/resource_groups/{groupId}/resources/{resourceCode}",
    "put",
    "AdminAiResourceGroupMemberUpdateRequest",
    "AI resource-group member assignment request.",
  );

  standardizeUpdateOperation(operationAt(document, "/backend/v3/api/ai/models/{modelId}", "patch"));
  standardizeUpdateOperation(
    operationAt(document, "/backend/v3/api/ai/model_mappings/{mappingId}", "patch"),
  );
  standardizeUpdateOperation(
    operationAt(document, "/backend/v3/api/ai/resources/{resourceId}", "put"),
  );
  standardizeUpdateOperation(
    operationAt(document, "/backend/v3/api/ai/resource_groups/{groupId}", "patch"),
  );
  const memberUpdateOperation = operationAt(
    document,
    "/backend/v3/api/ai/resource_groups/{groupId}/resources/{resourceCode}",
    "put",
  );
  if (memberUpdateOperation?.responses) {
    memberUpdateOperation.responses = {
      "200": operationResourceItemSuccessResponse(
        "AdminAiResourceGroupResourceItem",
        "OK",
      ),
      ...keepProblemResponses(memberUpdateOperation.responses),
    };
  }
  standardizeDataOperation(
    operationAt(document, "/backend/v3/api/ai/models/sync", "post"),
    "ModelCatalogSyncResult",
    "OK",
  );
  standardizeDataOperation(
    operationAt(document, "/backend/v3/api/ai/model_rankings/refresh", "post"),
    "ModelRankingRefreshTriggerResponse",
    "OK",
  );
}

function standardizeModelMappingsResolveOperation(document) {
  if (!document.components) {
    document.components = {};
  }
  if (!document.components.schemas) {
    document.components.schemas = {};
  }
  document.components.schemas.AdminModelMappingResolveRequest =
    adminModelMappingResolveRequestSchema();

  const operation = document.paths?.["/backend/v3/api/ai/model_mappings/resolve"]?.post;
  if (!operation) {
    return;
  }
  operation.operationId = "modelMappings.resolve";
  operation.summary = operation.summary || "Resolve model mapping";
  operation.description =
    "Resolve a model mapping rule without creating or mutating catalog state.";
  operation.requestBody = jsonRequestBody(
    "#/components/schemas/AdminModelMappingResolveRequest",
    "Model mapping resolve request.",
  );
  standardizeRecordDataOperation(operation, "OK");
}

function standardizeBackendOperationPatterns(document) {
  delete document.paths?.["/backend/v3/api/ai/model_mappings"]?.put;

  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    if (pathItem.post?.operationId?.endsWith(".create")) {
      standardizeCreateOperation(pathItem.post);
    }
    if (pathItem.delete?.operationId?.endsWith(".delete")) {
      standardizeDeleteOperation(pathItem.delete);
    }
    if (Object.keys(pathItem).length === 0) {
      delete document.paths[pathKey];
    }
  }
  standardizeBackendWriteOperations(document);
  standardizeAiResourceListOperations(document);
  standardizeModelMappingsListOperation(document);
  standardizeModelMappingsResolveOperation(document);
  return document;
}

function pruneUnusedComponentSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas) {
    return document;
  }

  const seedSchemaNames = collectSchemaRefs(document.paths || {});
  document.components.schemas = expandSchemas(schemas, seedSchemaNames);
  return document;
}

function gatewayPermission(operationId, method) {
  const resourcePermission =
    operationId?.startsWith("resources.") ||
    operationId?.startsWith("resourceGroups.")
      ? "intelligence.resources"
      : "intelligence.models";
  const readOperation = method === "get" || operationId === "modelMappings.resolve";
  return `${resourcePermission}.${readOperation ? "read" : "manage"}`;
}

function decorateGatewayContract(document, surface) {
  const appSurface = surface === "app-api";
  for (const pathItem of Object.values(document.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!["get", "post", "put", "patch", "delete"].includes(method)) {
        continue;
      }
      operation["x-sdkwork-api-surface"] = surface;
      operation["x-sdkwork-request-context"] = "WebRequestContext";
      operation["x-sdkwork-auth-mode"] = "dual-token";
      operation.security = [{ AccessToken: [], AuthToken: [] }];
      if (!appSurface) {
        operation["x-sdkwork-permission"] = gatewayPermission(
          operation.operationId,
          method,
        );
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

const backendDocument = decorateGatewayContract(
  migrateOpenApiDocument(
    pruneUnusedComponentSchemas(
      standardizeBackendOperationPatterns(
        mergeModelCatalogSyncSchema(
          mergeVideoProfileCatalogBackendPaths(
            mergeVoiceCatalogBackendPaths(
              mergeAiResourceGroupMemberPaths(
                mergeAiResourceWritePaths(
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
        ),
      ),
    ),
  ),
  "backend-api",
);
const appDocument = decorateGatewayContract(
  migrateOpenApiDocument(
    mergeModelAccessChannelAppPath(
      mergeVideoProfileCatalogAppPaths(
        mergeVoiceCatalogAppPaths(
          extractSurface(
            appSource,
            APP_PATH_PREFIXES,
            "SDKWork Models App API",
            "/app/v3/api",
          ),
        ),
      ),
    ),
  ),
  "app-api",
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
