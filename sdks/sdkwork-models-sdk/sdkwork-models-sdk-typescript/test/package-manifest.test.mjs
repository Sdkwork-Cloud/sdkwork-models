import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const packageJson = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
);

test("package manifest is configured for public npm publishing", () => {
  assert.equal(packageJson.name, "@sdkwork/models-sdk");
  assert.notEqual(packageJson.private, true);
  assert.equal(packageJson.publishConfig?.access, "public");
  assert.equal(packageJson.publishConfig?.registry, "https://registry.npmjs.org/");
  assert.deepEqual(packageJson.files, ["dist/", "README.md", "LICENSE"]);
  assert.equal(packageJson.scripts?.prepack, "npm run build");
  assert.equal(packageJson.scripts?.prepublishOnly, "npm test && npm run pack:dry-run");
  assert.equal(packageJson.scripts?.["pack:dry-run"], "npm pack --dry-run");
});
