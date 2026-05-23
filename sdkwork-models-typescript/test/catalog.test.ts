import test from "node:test";
import assert from "node:assert/strict";
import {
  catalogKey,
  findModel,
  findModelByVendorRegion,
  findMeter,
  findProtocol,
  getBestReferencePrice,
  getModelPrices,
  listAvailableModels,
  listMeters,
  listModels,
  listModelsByCapability,
  listModelsByModality,
  listModelsByProtocol,
  listProtocols,
  listProtocolsByVendor,
  listVendorRegions,
  listVendors,
  loadBundledCatalog,
  loadCatalog,
} from "../dist/index.js";

test("loads local catalog", async () => {
  const catalog = await loadCatalog("..");
  assert.equal(catalog.catalogVersion, "2026.05.08.1");
  assert.equal(findModel(catalog, "openai/global/gpt-5.5")?.vendorCode, "openai");
  assert.equal(findModel(catalog, "openai/global/gpt-5.5")?.regionCode, "global");
  assert.equal(findModelByVendorRegion(catalog, "openai", "global", "gpt-5.5")?.vendorCode, "openai");
  assert.equal(catalogKey("openai", "global", "gpt-5.5"), "openai/global/gpt-5.5");
  assert.equal(findModel(catalog, "openai/gpt-5.5"), undefined);
  assert.equal(listVendors(catalog).every((vendor) => !("regionCode" in vendor)), true);
  assert.equal(
    listVendorRegions(catalog).some((item) => item.vendorCode === "minimax" && item.regionCode === "cn"),
    true,
  );
  assert.equal(listVendors(catalog).filter((vendor) => vendor.vendorCode === "minimax").length, 1);
  assert.ok(listMeters(catalog).some((meter) => meter.meterCode === "llm_input_token"));
  assert.equal(findMeter(catalog, "llm_input_token")?.defaultUnitSize, "1000000");
  assert.equal(findMeter(catalog, "missing_meter"), undefined);
  assert.ok(listModels(catalog, { vendorCode: "openai", regionCode: "global", familyCode: "gpt-5" }).length > 0);
  assert.ok(listModels(catalog, { releaseStage: "active", shelfState: "listed", routingState: "enabled" }).length > 0);
  assert.ok(listModels(catalog, { apiFormat: "openai_compatible" }).length > 0);
  assert.ok(listModelsByCapability(catalog, "chat").length > 0);
  assert.ok(listModelsByModality(catalog, "text", "text").length > 0);
  assert.ok(listProtocols(catalog).length >= 4);
  assert.equal(findProtocol(catalog, "openai_responses")?.displayName, "OpenAI Responses API");
  assert.equal(findProtocol(catalog, "missing_protocol"), undefined);
  assert.ok(listProtocolsByVendor(catalog, "openai").some((protocol) => protocol.protocolCode === "openai_responses"));
  assert.equal(
    listModelsByProtocol(catalog, "openai_responses").every((model) => model.apiFormat === "openai_responses"),
    true,
  );
  assert.ok(listVendors(catalog).some((vendor) => vendor.supportedProtocols.includes("openai_responses")));
  assert.ok(listAvailableModels(catalog).length > 0);
  assert.equal(
    listAvailableModels(catalog).some((model) => getModelPrices(catalog, model.catalogKey).length === 0),
    false,
  );
  assert.equal(
    listAvailableModels(catalog).some((model) => model.routingState !== "enabled" || model.shelfState !== "listed"),
    false,
  );
  assert.equal(
    listAvailableModels(catalog).some((model) => model.catalogKey === "kuaishou/cn/kling-v3-0-preview"),
    false,
  );
  assert.ok(getModelPrices(catalog, "openai/global/gpt-5.5").length > 0);
  assert.equal(
    getBestReferencePrice(catalog, "openai/global/gpt-5.5", "llm_input_token")?.unitPrice,
    "5.000000",
  );
});

test("loads bundled catalog from SDKWORK_MODELS_CATALOG_ROOT", async () => {
  const previousRoot = process.env.SDKWORK_MODELS_CATALOG_ROOT;
  process.env.SDKWORK_MODELS_CATALOG_ROOT = "..";
  try {
    const catalog = await loadBundledCatalog();
    assert.equal(catalog.catalogVersion, "2026.05.08.1");
    assert.equal(findModel(catalog, "openai/global/gpt-5.5")?.vendorCode, "openai");
  } finally {
    if (previousRoot === undefined) {
      delete process.env.SDKWORK_MODELS_CATALOG_ROOT;
    } else {
      process.env.SDKWORK_MODELS_CATALOG_ROOT = previousRoot;
    }
  }
});
