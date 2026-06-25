package com.sdkwork.models;

import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Map;

import static java.util.Map.entry;
import static org.junit.jupiter.api.Assertions.assertEquals;

class ModelCatalogTest {
    @Test
    void testFindModel() {
        ModelCatalog catalog = ModelCatalogLoader.fromParts(
                "fixture-1.0.0",
                "1.0.0",
                List.of(Map.of("meterCode", "llm_input_token", "defaultUnitSize", "1000000")),
                List.of(
                        Map.of(
                                "protocolCode", "openai_compatible",
                                "vendorOrigin", "openai",
                                "displayName", "OpenAI Chat Completions Compatible"
                        ),
                        Map.of(
                                "protocolCode", "openai_responses",
                                "vendorOrigin", "openai",
                                "displayName", "OpenAI Responses API"
                        )
                ),
                List.of(
                        Map.of(
                                "vendorCode", "openai",
                                "displayName", "OpenAI",
                                "supportedProtocols", List.of("openai_compatible", "openai_responses"),
                                "clientApiCompatibility", Map.of(
                                        "codex", Map.of("clientApiCode", "codex", "supportStatus", "supported")
                                )
                        ),
                        Map.of(
                                "vendorCode", "minimax",
                                "displayName", "MiniMax",
                                "supportedProtocols", List.of("openai_compatible"),
                                "regions", List.of(
                                        Map.of("regionCode", "cn"),
                                        Map.of("regionCode", "global")
                                )
                        ),
                        Map.of(
                                "vendorCode", "kuaishou",
                                "displayName", "Kuaishou",
                                "supportedProtocols", List.of("openai_compatible"),
                                "regions", List.of(
                                        Map.of("regionCode", "cn"),
                                        Map.of("regionCode", "global")
                                )
                        )
                ),
                List.of(
                        Map.ofEntries(
                                entry("catalogKey", "openai/gpt-5.5"),
                                entry("modelId", "gpt-5.5"),
                                entry("vendorCode", "openai"),
                                entry("regionCode", "global"),
                                entry("familyCode", "gpt-5"),
                                entry("releaseStage", "active"),
                                entry("shelfState", "listed"),
                                entry("routingState", "enabled"),
                                entry("apiFormat", "openai_compatible"),
                                entry("capabilities", List.of("chat")),
                                entry("inputModalities", List.of("text")),
                                entry("outputModalities", List.of("text"))
                        ),
                        Map.ofEntries(
                                entry("catalogKey", "minimax/MiniMax-M2.7"),
                                entry("modelId", "MiniMax-M2.7"),
                                entry("vendorCode", "minimax"),
                                entry("regionCode", "cn"),
                                entry("familyCode", "MiniMax-M2"),
                                entry("releaseStage", "active"),
                                entry("shelfState", "listed"),
                                entry("routingState", "enabled"),
                                entry("apiFormat", "openai_compatible"),
                                entry("capabilities", List.of("chat")),
                                entry("inputModalities", List.of("text")),
                                entry("outputModalities", List.of("text"))
                        ),
                        Map.ofEntries(
                                entry("catalogKey", "kuaishou/kling-v3-0-preview"),
                                entry("modelId", "kling-v3-0-preview"),
                                entry("vendorCode", "kuaishou"),
                                entry("regionCode", "cn"),
                                entry("familyCode", "kling"),
                                entry("lifecycle", "catalog_only"),
                                entry("releaseStage", "preview"),
                                entry("shelfState", "hidden"),
                                entry("routingState", "catalog_only"),
                                entry("apiFormat", "openai_compatible"),
                                entry("capabilities", List.of("video")),
                                entry("inputModalities", List.of("text")),
                                entry("outputModalities", List.of("video"))
                        ),
                        Map.ofEntries(
                                entry("catalogKey", "kuaishou/kling-v3-0-preview"),
                                entry("modelId", "kling-v3-0-preview"),
                                entry("vendorCode", "kuaishou"),
                                entry("regionCode", "global"),
                                entry("familyCode", "kling"),
                                entry("lifecycle", "preview"),
                                entry("releaseStage", "active"),
                                entry("shelfState", "listed"),
                                entry("routingState", "enabled"),
                                entry("apiFormat", "openai_compatible"),
                                entry("capabilities", List.of("video")),
                                entry("inputModalities", List.of("text")),
                                entry("outputModalities", List.of("video"))
                        )
                ),
                List.of(
                        Map.of(
                                "catalogKey", "openai/gpt-5.5",
                                "vendorCode", "openai",
                                "regionCode", "global",
                                "modelId", "gpt-5.5",
                                "prices", List.of(Map.of("meterCode", "llm_input_token", "unitPrice", "5.000000"))
                        ),
                        Map.of(
                                "catalogKey", "kuaishou/kling-v3-0-preview",
                                "vendorCode", "kuaishou",
                                "regionCode", "global",
                                "modelId", "kling-v3-0-preview",
                                "prices", List.of(Map.of("meterCode", "video_second", "unitPrice", "0.200000"))
                        )
                )
        );

        assertEquals("openai", SdkworkModels.findModel(catalog, "openai/gpt-5.5").get("vendorCode"));
        assertEquals("global", SdkworkModels.findModel(catalog, "openai/gpt-5.5").get("regionCode"));
        assertEquals("openai/gpt-5.5", SdkworkModels.catalogKey("openai", "gpt-5.5"));
        assertEquals(null, SdkworkModels.findModel(catalog, "openai/global/gpt-5.5"));
        assertEquals(true, ModelCatalogQuery.listVendors(catalog).stream()
                .noneMatch(vendor -> vendor.containsKey("regionCode")));
        assertEquals(4, ModelCatalogQuery.listVendorRegions(catalog).size());
        assertEquals(1, ModelCatalogQuery.listVendors(catalog).stream()
                .filter(vendor -> "minimax".equals(vendor.get("vendorCode")))
                .count());
        assertEquals(1, ModelCatalogQuery.listMeters(catalog).size());
        assertEquals("1000000", ModelCatalogQuery.findMeter(catalog, "llm_input_token").get("defaultUnitSize"));
        assertEquals(null, ModelCatalogQuery.findMeter(catalog, "missing_meter"));
        assertEquals(1, ModelCatalogQuery.listModels(catalog, Map.of(
                "vendorCode", "openai",
                "regionCode", "global",
                "familyCode", "gpt-5"
        )).size());
        assertEquals(2, ModelCatalogQuery.listModels(catalog, Map.of("regionCode", "global")).size());
        assertEquals(3, ModelCatalogQuery.listModels(catalog).size());
        assertEquals(
                3,
                ModelCatalogQuery.listModels(catalog).stream()
                        .map(model -> model.get("catalogKey"))
                        .distinct()
                        .count()
        );
        assertEquals(
                "global",
                ModelCatalogQuery.findModel(catalog, "kuaishou/kling-v3-0-preview").get("regionCode")
        );
        assertEquals(
                "cn",
                ModelCatalogQuery.findModelByVendorRegion(catalog, "kuaishou", "cn", "kling-v3-0-preview")
                        .get("regionCode")
        );
        assertEquals(3, ModelCatalogQuery.listModels(catalog, Map.of(
                "releaseStage", "active",
                "shelfState", "listed",
                "routingState", "enabled",
                "apiFormat", "openai_compatible"
        )).size());
        assertEquals(2, ModelCatalogQuery.listModelsByCapability(catalog, "chat").size());
        assertEquals(2, ModelCatalogQuery.listModelsByModality(catalog, "text", "text").size());
        assertEquals(2, ModelCatalogQuery.listProtocols(catalog).size());
        assertEquals("OpenAI Responses API", ModelCatalogQuery.findProtocol(catalog, "openai_responses").get("displayName"));
        assertEquals(null, ModelCatalogQuery.findProtocol(catalog, "missing_protocol"));
        assertEquals(2, SdkworkModels.listProtocolsByVendor(catalog, "openai").size());
        assertEquals(
                true,
                SdkworkModels.listClientApiCompatibilityByVendor(catalog, "openai").stream()
                        .anyMatch(item -> "codex".equals(item.get("clientApiCode"))
                                && "supported".equals(item.get("supportStatus")))
        );
        assertEquals(3, SdkworkModels.listModelsByProtocol(catalog, "openai_compatible").size());
        assertEquals(2, ModelCatalogQuery.listAvailableModels(catalog).size());
        assertEquals("openai/gpt-5.5", ModelCatalogQuery.listAvailableModels(catalog).getFirst().get("catalogKey"));
        assertEquals(1, ModelCatalogQuery.getModelPrices(catalog, "openai/gpt-5.5").size());
        assertEquals(1, ModelCatalogQuery.getModelRegionPrices(catalog, "openai/gpt-5.5", "global").size());
        assertEquals(0, ModelCatalogQuery.getModelRegionPrices(catalog, "openai/gpt-5.5", "cn").size());
        assertEquals(0, ModelCatalogQuery.listAvailableModels(catalog, Map.of("regionCode", "cn")).stream()
                .filter(model -> "kuaishou/kling-v3-0-preview".equals(model.get("catalogKey")))
                .count());
        assertEquals(0, ModelCatalogQuery.getModelPrices(catalog, "openai/global/gpt-5.5").size());
        assertEquals(
                "5.000000",
                ModelCatalogQuery.getBestReferencePrice(catalog, "openai/gpt-5.5", "llm_input_token")
                        .get("unitPrice")
        );
    }

    @Test
    void catalogKeyParserKeepsSlashDelimitedProviderModelIdsIntact() {
        ModelCatalog catalog = new ModelCatalog(
                "fixture-1.0.0",
                "1.1.0",
                List.of(),
                List.of(),
                List.of(),
                List.of(Map.of(
                        "vendorCode", "openrouter",
                        "regionCode", "global",
                        "models", List.of(Map.ofEntries(
                                entry("catalogKey", "openrouter/anthropic/claude-3-opus"),
                                entry("modelId", "anthropic/claude-3-opus"),
                                entry("displayName", "Claude 3 Opus through OpenRouter"),
                                entry("vendorCode", "openrouter"),
                                entry("regionCode", "global"),
                                entry("familyCode", "anthropic"),
                                entry("releaseStage", "active"),
                                entry("shelfState", "listed"),
                                entry("routingState", "enabled"),
                                entry("apiFormat", "openai_compatible"),
                                entry("capabilities", List.of("chat")),
                                entry("inputModalities", List.of("text")),
                                entry("outputModalities", List.of("text"))
                        )),
                        "pricing", List.of(Map.of(
                                "catalogKey", "openrouter/anthropic/claude-3-opus",
                                "vendorCode", "openrouter",
                                "regionCode", "global",
                                "modelId", "anthropic/claude-3-opus",
                                "currency", "USD",
                                "prices", List.of(Map.of("meterCode", "llm_input_token", "unitPrice", "15.000000"))
                        ))
                )),
                List.of(),
                List.of(Map.of(
                        "catalogKey", "openrouter/anthropic/claude-3-opus",
                        "vendorCode", "openrouter",
                        "regionCode", "global",
                        "modelId", "anthropic/claude-3-opus",
                        "currency", "USD",
                        "prices", List.of(Map.of("meterCode", "llm_input_token", "unitPrice", "15.000000"))
                ))
        );

        assertEquals("openrouter/anthropic/claude-3-opus", SdkworkModels.catalogKey("openrouter", "anthropic/claude-3-opus"));
        assertEquals(
                "anthropic/claude-3-opus",
                SdkworkModels.findModel(catalog, "openrouter/anthropic/claude-3-opus").get("modelId")
        );
        assertEquals(1, ModelCatalogQuery.getModelPrices(catalog, "openrouter/anthropic/claude-3-opus").size());
        assertEquals(1, ModelCatalogQuery.getModelRegionPrices(catalog, "openrouter/anthropic/claude-3-opus", "global").size());
        assertEquals(null, SdkworkModels.findModel(catalog, "openrouter/global/anthropic/claude-3-opus"));
        assertEquals(0, ModelCatalogQuery.getModelPrices(catalog, "openrouter/global/anthropic/claude-3-opus").size());
    }
}
