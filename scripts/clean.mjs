import { rmSync } from "node:fs";
import { join } from "node:path";
import { projectRootFromTool } from "../tools/catalog-lib.mjs";

const root = projectRootFromTool(import.meta.url);

for (const relativePath of [
  "sdks/sdkwork-models-sdk/sdkwork-models-sdk-typescript/dist",
  "sdks/sdkwork-models-sdk/sdkwork-models-sdk-typescript/node_modules/.cache",
  "sdks/sdkwork-models-sdk/sdkwork-models-sdk-rust/target",
  "sdks/sdkwork-models-sdk/sdkwork-models-sdk-java/target",
  "sdks/sdkwork-models-sdk/sdkwork-models-sdk-flutter/.dart_tool",
  "sdks/sdkwork-models-sdk/sdkwork-models-sdk-flutter/build",
  "apps/sdkwork-models-pc/dist",
  "apps/sdkwork-models-pc/node_modules/.vite",
]) {
  rmSync(join(root, relativePath), { force: true, recursive: true });
}

console.log("sdkwork-models clean complete");
