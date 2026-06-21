import test from "node:test";
import assert from "node:assert/strict";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  catalogKey,
  findModel,
  findModelByVendorRegion,
  findMeter,
  findProtocol,
  getBestReferencePrice,
  getModelPrices,
  getModelRegionPrices,
  listClientApiCompatibilityByVendor,
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

const REPOSITORY_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "..");

test("loads local catalog", async () => {
  const catalog = await loadCatalog(REPOSITORY_ROOT);
  assert.equal(catalog.catalogVersion, "2026.05.08.1");
  assert.equal(findModel(catalog, "openai/gpt-5.5")?.vendorCode, "openai");
  assert.equal(findModel(catalog, "openai/gpt-5.5")?.regionCode, "global");
  assert.equal(findModelByVendorRegion(catalog, "openai", "global", "gpt-5.5")?.vendorCode, "openai");
  assert.equal(catalogKey("openai", "gpt-5.5"), "openai/gpt-5.5");
  assert.equal(findModel(catalog, "openai/global/gpt-5.5"), undefined);
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
  assert.ok(
    listClientApiCompatibilityByVendor(catalog, "openai").some(
      (item) => item.clientApiCode === "codex" && item.supportStatus === "supported",
    ),
  );
  assert.ok(listAvailableModels(catalog).length > 0);
  const modelKeys = listModels(catalog).map((model) => model.catalogKey);
  assert.equal(new Set(modelKeys).size, modelKeys.length);
  assert.equal(
    listAvailableModels(catalog).some((model) => getModelPrices(catalog, model.catalogKey).length === 0),
    false,
  );
  assert.equal(
    listAvailableModels(catalog).some((model) => model.routingState !== "enabled" || model.shelfState !== "listed"),
    false,
  );
  assert.equal(
    listAvailableModels(catalog).some(
      (model) => model.catalogKey === "kuaishou/kling-v3-0-preview" && model.regionCode === "global",
    ),
    true,
  );
  assert.equal(findModel(catalog, "kuaishou/kling-v3-0-preview")?.regionCode, "global");
  assert.equal(findModelByVendorRegion(catalog, "kuaishou", "cn", "kling-v3-0-preview")?.regionCode, "cn");
  assert.equal(
    listAvailableModels(catalog, { regionCode: "cn" }).some(
      (model) => model.catalogKey === "kuaishou/kling-v3-0-preview",
    ),
    false,
  );
  assert.equal(
    listAvailableModels(catalog, { regionCode: "global" }).some(
      (model) => model.catalogKey === "kuaishou/kling-v3-0-preview",
    ),
    true,
  );
  assert.ok(getModelPrices(catalog, "openai/gpt-5.5").length > 0);
  assert.ok(getModelRegionPrices(catalog, "openai/gpt-5.5", "global").length > 0);
  assert.equal(getModelRegionPrices(catalog, "openai/gpt-5.5", "cn").length, 0);
  assert.equal(getModelPrices(catalog, "openai/global/gpt-5.5").length, 0);
  assert.equal(
    getBestReferencePrice(catalog, "openai/gpt-5.5", "llm_input_token")?.unitPrice,
    "5.000000",
  );
});

test("loads bundled catalog from SDKWORK_MODELS_CATALOG_ROOT", async () => {
  const previousRoot = process.env.SDKWORK_MODELS_CATALOG_ROOT;
  process.env.SDKWORK_MODELS_CATALOG_ROOT = REPOSITORY_ROOT;
  try {
    const catalog = await loadBundledCatalog();
    assert.equal(catalog.catalogVersion, "2026.05.08.1");
    assert.equal(findModel(catalog, "openai/gpt-5.5")?.vendorCode, "openai");
  } finally {
    if (previousRoot === undefined) {
      delete process.env.SDKWORK_MODELS_CATALOG_ROOT;
    } else {
      process.env.SDKWORK_MODELS_CATALOG_ROOT = previousRoot;
    }
  }
});

test("catalog key parser keeps slash-delimited provider model ids intact", () => {
  const catalog = {
    catalogVersion: "2026.05.08.1",
    schemaVersion: "1.1.0",
    meters: [],
    protocols: [],
    vendors: [
      {
        vendorCode: "openrouter",
        regionCode: "global",
        vendor: {
          vendorCode: "openrouter",
          displayName: "OpenRouter",
          vendorType: "commercial",
            capabilities: ["chat"],
            supportedProtocols: ["openai_compatible"],
            clientApiCompatibility: {},
            openSource: false,
          },
        models: [
          {
            catalogKey: "openrouter/anthropic/claude-3-opus",
            modelId: "anthropic/claude-3-opus",
            displayName: "Claude 3 Opus through OpenRouter",
            vendorCode: "openrouter",
            regionCode: "global",
            familyCode: "anthropic",
            primaryCapability: "chat",
            capabilities: ["chat"],
            inputModalities: ["text"],
            outputModalities: ["text"],
            apiFormat: "openai_compatible",
            lifecycle: "current",
            releaseStage: "active",
            shelfState: "listed",
            routingState: "enabled",
            source: { sourceUrl: "https://openrouter.ai", observedAt: "2026-06-02" },
          },
        ],
        pricing: [
          {
            catalogKey: "openrouter/anthropic/claude-3-opus",
            vendorCode: "openrouter",
            regionCode: "global",
            modelId: "anthropic/claude-3-opus",
            currency: "USD",
            prices: [
              {
                priceId: "openrouter-claude-opus-input",
                priceSide: "input",
                pricingScope: "reference",
                meterCode: "llm_input_token",
                unitSize: "1000000",
                unitPrice: "15.000000",
                minimumQuantity: "0",
                currency: "USD",
                effectiveFrom: "2026-06-02",
                source: { sourceUrl: "https://openrouter.ai", observedAt: "2026-06-02" },
              },
            ],
          },
        ],
      },
    ],
  } as any;

  assert.equal(catalogKey("openrouter", "anthropic/claude-3-opus"), "openrouter/anthropic/claude-3-opus");
  assert.equal(findModel(catalog, "openrouter/anthropic/claude-3-opus")?.modelId, "anthropic/claude-3-opus");
  assert.equal(getModelPrices(catalog, "openrouter/anthropic/claude-3-opus").length, 1);
  assert.equal(getModelRegionPrices(catalog, "openrouter/anthropic/claude-3-opus", "global").length, 1);
  assert.equal(findModel(catalog, "openrouter/global/anthropic/claude-3-opus"), undefined);
  assert.equal(getModelPrices(catalog, "openrouter/global/anthropic/claude-3-opus").length, 0);
});
