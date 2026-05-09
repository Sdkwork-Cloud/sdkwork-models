package com.sdkwork.models;

import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import static java.util.Map.entry;

/**
 * Dependency-free smoke test for offline SDK verification.
 */
public final class ModelCatalogSmoke {
    private ModelCatalogSmoke() {
    }

    public static void main(String[] args) {
        ModelCatalog catalog = ModelCatalogLoader.fromParts(
                "2026.05.08.1",
                "1.0.0",
                List.of(Map.of("meterCode", "llm_input_token", "defaultUnitSize", "1000000")),
                List.of(
                        Map.of("vendorCode", "openai", "displayName", "OpenAI"),
                        Map.of(
                                "vendorCode", "minimax",
                                "displayName", "MiniMax",
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

        require("openai".equals(SdkworkModels.findModel(catalog, "openai/global/gpt-5.5").get("vendorCode")));
        require("global".equals(SdkworkModels.findModel(catalog, "openai/global/gpt-5.5").get("regionCode")));
        require(SdkworkModels.findModel(catalog, "openai/gpt-5.5") == null);
        require("openai/global/gpt-5.5".equals(SdkworkModels.catalogKey("openai", "global", "gpt-5.5")));
        require(SdkworkModels.listVendors(catalog).stream().noneMatch(vendor -> vendor.containsKey("regionCode")));
        require(SdkworkModels.listVendorRegions(catalog).size() == 2);
        require(SdkworkModels.listMeters(catalog).stream()
                .anyMatch(meter -> "llm_input_token".equals(meter.get("meterCode"))));
        require("1000000".equals(SdkworkModels.findMeter(catalog, "llm_input_token").get("defaultUnitSize")));
        require(SdkworkModels.findMeter(catalog, "missing_meter") == null);
        require(!SdkworkModels.listModels(catalog, Map.of(
                "vendorCode", "openai",
                "regionCode", "global",
                "familyCode", "gpt-5"
        )).isEmpty());
        require(!SdkworkModels.listModels(catalog, Map.of(
                "releaseStage", "active",
                "shelfState", "listed",
                "routingState", "enabled",
                "apiFormat", "openai_compatible"
        )).isEmpty());
        require(SdkworkModels.listModelsByCapability(catalog, "chat").size() == 2);
        require(SdkworkModels.listModelsByModality(catalog, "text", "text").size() == 2);
        require(SdkworkModels.listAvailableModels(catalog).size() == 1);
        require("openai/global/gpt-5.5".equals(SdkworkModels.listAvailableModels(catalog).getFirst().get("catalogKey")));
        require(SdkworkModels.getModelPrices(catalog, "openai/global/gpt-5.5").size() == 1);
        require("5.000000".equals(
                SdkworkModels.getBestReferencePrice(catalog, "openai/global/gpt-5.5", "llm_input_token")
                        .get("unitPrice")
        ));

        ModelCatalog localCatalog = SdkworkModels.loadCatalog(Path.of("data", "sdkwork-models"));
        require("openai".equals(SdkworkModels.findModel(localCatalog, "openai/global/gpt-5.5").get("vendorCode")));
        require(SdkworkModels.listModels(localCatalog, Map.of(
                "vendorCode", "minimax",
                "regionCode", "cn",
                "familyCode", "minimax"
        )).size() >= 1);
        Map<String, Object> openaiGlobal = SdkworkModels.loadVendorCatalog(
                Path.of("data", "sdkwork-models"),
                "openai",
                "global"
        );
        require("openai".equals(openaiGlobal.get("vendorCode")));
        require("global".equals(openaiGlobal.get("regionCode")));
        require(!((List<?>) openaiGlobal.get("models")).isEmpty());
    }

    private static void require(boolean value) {
        if (!value) {
            throw new AssertionError("sdkwork-models Java smoke assertion failed");
        }
    }
}
