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
                "2026.05.08.1",
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
                                "supportedProtocols", List.of("openai_compatible", "openai_responses")
                        ),
                        Map.of(
                                "vendorCode", "minimax",
                                "displayName", "MiniMax",
                                "supportedProtocols", List.of("openai_compatible"),
                                "regions", List.of(
                                        Map.of("regionCode", "cn"),
                                        Map.of("regionCode", "global")
                                )
                        )
                ),
                List.of(
                        Map.ofEntries(
                                entry("catalogKey", "openai/global/gpt-5.5"),
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
                                entry("catalogKey", "minimax/cn/MiniMax-M2.7"),
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
                        )
                ),
                List.of(Map.of(
                        "catalogKey", "openai/global/gpt-5.5",
                        "vendorCode", "openai",
                        "regionCode", "global",
                        "modelId", "gpt-5.5",
                        "prices", List.of(Map.of("meterCode", "llm_input_token", "unitPrice", "5.000000"))
                ))
        );

        assertEquals("openai", SdkworkModels.findModel(catalog, "openai/global/gpt-5.5").get("vendorCode"));
        assertEquals("global", SdkworkModels.findModel(catalog, "openai/global/gpt-5.5").get("regionCode"));
        assertEquals("openai/global/gpt-5.5", SdkworkModels.catalogKey("openai", "global", "gpt-5.5"));
        assertEquals(null, SdkworkModels.findModel(catalog, "openai/gpt-5.5"));
        assertEquals(true, ModelCatalogQuery.listVendors(catalog).stream()
                .noneMatch(vendor -> vendor.containsKey("regionCode")));
        assertEquals(2, ModelCatalogQuery.listVendorRegions(catalog).size());
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
        assertEquals(2, ModelCatalogQuery.listModels(catalog, Map.of(
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
        assertEquals(2, SdkworkModels.listModelsByProtocol(catalog, "openai_compatible").size());
        assertEquals(1, ModelCatalogQuery.listAvailableModels(catalog).size());
        assertEquals("openai/global/gpt-5.5", ModelCatalogQuery.listAvailableModels(catalog).getFirst().get("catalogKey"));
        assertEquals(1, ModelCatalogQuery.getModelPrices(catalog, "openai/global/gpt-5.5").size());
        assertEquals(
                "5.000000",
                ModelCatalogQuery.getBestReferencePrice(catalog, "openai/global/gpt-5.5", "llm_input_token")
                        .get("unitPrice")
        );
    }
}
