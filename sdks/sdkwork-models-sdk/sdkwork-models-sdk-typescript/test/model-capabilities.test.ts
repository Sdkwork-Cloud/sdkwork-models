import test from "node:test";
import assert from "node:assert/strict";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  findModel,
  getModelCapabilityProfile,
  listModelsWithFeature,
  loadCatalog,
  modelSupportsAudioInput,
  modelSupportsFeature,
  modelSupportsImageInput,
  modelSupportsSpeechInput,
  modelSupportsToolCall,
  modelSupportsVision,
} from "../dist/index.js";

const REPOSITORY_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "..");

test("model capability predicates reflect catalog modality and feature flags", async () => {
  const catalog = await loadCatalog(REPOSITORY_ROOT);

  const claude = findModel(catalog, "anthropic/claude-opus-4-8");
  assert.ok(claude);
  assert.equal(modelSupportsVision(claude!), true);
  assert.equal(modelSupportsImageInput(claude!), true);
  assert.equal(modelSupportsToolCall(claude!), true);
  assert.equal(modelSupportsFeature(claude!, "tool_call"), true);
  assert.equal(modelSupportsAudioInput(claude!), false);

  const live = findModel(catalog, "google/gemini-3.1-flash-live-preview");
  assert.ok(live);
  assert.equal(modelSupportsSpeechInput(live!), true);
  assert.equal(modelSupportsAudioInput(live!), true);
  assert.equal(modelSupportsToolCall(live!), true);

  const tts = findModel(catalog, "google/gemini-3.1-flash-tts-preview");
  assert.ok(tts);
  assert.equal(modelSupportsToolCall(tts!), false);
  assert.equal(modelSupportsFeature(tts!, "structured_output"), false);

  const profile = getModelCapabilityProfile(claude!);
  assert.equal(profile.catalogKey, "anthropic/claude-opus-4-8");
  assert.ok(profile.features.includes("tool_call"));
  assert.ok(profile.features.includes("streaming"));

  assert.ok(listModelsWithFeature(catalog, "tool_call").length > 0);
  assert.equal(
    listModelsWithFeature(catalog, "tool_call").every((model) => model.supportsTools === true),
    true,
  );
});
