#!/usr/bin/env node
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");

function runNodeScript(relativePath, args = []) {
  const result = spawnSync(process.execPath, [join(root, relativePath), ...args], {
    cwd: root,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

runNodeScript("tools/models_openapi_export.mjs", checkOnly ? ["--check"] : []);

const targets = [
  [
    join(root, "apis/backend-api/intelligence/openapi.json"),
    join(root, "sdks/sdkwork-models-backend-sdk/openapi/sdkwork-models-backend-api.openapi.json"),
  ],
  [
    join(root, "apis/app-api/intelligence/openapi.json"),
    join(root, "sdks/sdkwork-models-app-sdk/openapi/sdkwork-models-app-api.openapi.json"),
  ],
];

const routeSurfaces = [
  {
    openapi: "apis/app-api/intelligence/openapi.json",
    target:
      "sdks/_route-manifests/app-api/sdkwork-routes-models-catalog-app-api.route-manifest.json",
    packageName: "sdkwork-routes-models-catalog-app-api",
    crateRoot: "crates/sdkwork-routes-catalog-app-api",
    crateImport: "sdkwork_routes_models_catalog_app_api",
    surface: "app-api",
    apiAuthority: "sdkwork-models-app-api",
    sdkFamily: "sdkwork-models-app-sdk",
    prefix: "/app/v3/api/ai",
  },
  {
    openapi: "apis/backend-api/intelligence/openapi.json",
    target:
      "sdks/_route-manifests/backend-api/sdkwork-routes-models-catalog-backend-api.route-manifest.json",
    packageName: "sdkwork-routes-models-catalog-backend-api",
    crateRoot: "crates/sdkwork-routes-catalog-backend-api",
    crateImport: "sdkwork_routes_models_catalog_backend_api",
    surface: "backend-api",
    apiAuthority: "sdkwork-models-backend-api",
    sdkFamily: "sdkwork-models-backend-sdk",
    prefix: "/backend/v3/api/ai",
  },
];

function routeManifest(profile) {
  const openapi = JSON.parse(readFileSync(join(root, profile.openapi), "utf8"));
  const routes = [];
  for (const [routePath, pathItem] of Object.entries(openapi.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!["get", "post", "put", "patch", "delete"].includes(method)) {
        continue;
      }
      const authMode = operation["x-sdkwork-auth-mode"];
      routes.push({
        method: method.toUpperCase(),
        path: routePath,
        operationId: operation.operationId,
        tags: operation.tags ?? ["intelligence"],
        auth: { mode: authMode, required: authMode !== "public" },
        handler: { module: "crate::routes", name: null },
        ownership: {
          owner: "sdkwork-models",
          apiAuthority: profile.apiAuthority,
        },
        requestContext: operation["x-sdkwork-request-context"],
        apiSurface: profile.surface,
        permission: operation["x-sdkwork-permission"] ?? null,
        idempotent: operation["x-sdkwork-idempotent"] === true,
      });
    }
  }
  routes.sort((left, right) =>
    left.path.localeCompare(right.path) || left.method.localeCompare(right.method),
  );
  return {
    schemaVersion: 1,
    kind: "sdkwork.route.manifest",
    packageName: profile.packageName,
    surface: profile.surface,
    owner: "sdkwork-models",
    domain: "intelligence",
    capability: "models-catalog",
    apiAuthority: profile.apiAuthority,
    sdkFamily: profile.sdkFamily,
    prefix: profile.prefix,
    source: {
      crateRoot: profile.crateRoot,
      crateImport: profile.crateImport,
      openApiAuthority: profile.openapi,
    },
    routes,
  };
}

function materializeJson(relativePath, value) {
  const target = join(root, relativePath);
  const expected = `${JSON.stringify(value, null, 2)}\n`;
  if (checkOnly) {
    let actual;
    try {
      actual = readFileSync(target, "utf8");
    } catch {
      console.error(`materialized contract missing: ${target}`);
      process.exit(1);
    }
    if (actual !== expected) {
      console.error(`materialized contract drift detected: ${target}`);
      process.exit(1);
    }
    console.log(`materialized contract current: ${target}`);
    return;
  }
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, expected, "utf8");
  console.log(`materialized ${target}`);
}

for (const [source, target] of targets) {
  const expected = readFileSync(source, "utf8");
  if (checkOnly) {
    let actual;
    try {
      actual = readFileSync(target, "utf8");
    } catch {
      console.error(`materialized openapi missing: ${target}`);
      process.exit(1);
    }
    if (actual !== expected) {
      console.error(`materialized openapi drift detected: ${target}`);
      process.exit(1);
    }
    console.log(`materialized openapi current: ${target}`);
    continue;
  }
  mkdirSync(dirname(target), { recursive: true });
  copyFileSync(source, target);
  console.log(`materialized ${target}`);
}

for (const profile of routeSurfaces) {
  materializeJson(profile.target, routeManifest(profile));
}
