#!/usr/bin/env node
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

function runNodeScript(relativePath, args = []) {
  const result = spawnSync(process.execPath, [join(root, relativePath), ...args], {
    cwd: root,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

runNodeScript("tools/models_openapi_export.mjs");

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

for (const [source, target] of targets) {
  mkdirSync(dirname(target), { recursive: true });
  copyFileSync(source, target);
  console.log(`materialized ${target}`);
}
