#!/usr/bin/env node
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
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
