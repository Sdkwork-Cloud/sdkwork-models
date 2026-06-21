#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

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

runNodeScript("tools/materialize-models-openapi.mjs");
runNodeScript("sdks/sdkwork-models-backend-sdk/bin/generate-sdk.mjs", [
  "--input",
  "sdks/sdkwork-models-backend-sdk/openapi/sdkwork-models-backend-api.openapi.json",
  "--language",
  "typescript",
]);
runNodeScript("sdks/sdkwork-models-app-sdk/bin/generate-sdk.mjs", [
  "--input",
  "sdks/sdkwork-models-app-sdk/openapi/sdkwork-models-app-api.openapi.json",
  "--language",
  "typescript",
]);

console.log("[models_sdk_generate] generation completed");
