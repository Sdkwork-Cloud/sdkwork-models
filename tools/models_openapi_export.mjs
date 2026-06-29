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
];

const APP_PATH_PREFIXES = [
  "/app/v3/api/ai/model_vendors",
  "/app/v3/api/ai/models",
  "/app/v3/api/ai/model_rankings",
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
  injectModelsListQueryParams(
    extractSurface(
      backendSource,
      BACKEND_PATH_PREFIXES,
      "SDKWork Models Backend API",
      "/backend/v3/api",
    ),
  ),
);
const appDocument = migrateOpenApiDocument(
  injectModelsListQueryParams(
    extractSurface(
      appSource,
      APP_PATH_PREFIXES,
      "SDKWork Models App API",
      "/app/v3/api",
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
