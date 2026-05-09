package com.sdkwork.models;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

public final class ModelCatalogQuery {
    private ModelCatalogQuery() {
    }

    public static List<Map<String, Object>> listVendors(ModelCatalog catalog) {
        Map<String, Map<String, Object>> vendors = new LinkedHashMap<>();
        for (Map<String, Object> vendor : catalog.vendors()) {
            Object vendorCode = vendor.get("vendorCode");
            if (!(vendorCode instanceof String code) || vendors.containsKey(code)) {
                continue;
            }
            Map<String, Object> identity = new LinkedHashMap<>();
            identity.put("vendorCode", code);
            identity.put("displayName", vendor.get("displayName"));
            identity.put("legalName", vendor.get("legalName"));
            identity.put("vendorType", vendor.get("vendorType"));
            identity.put("capabilities", vendor.getOrDefault("capabilities", List.of()));
            identity.put("openSource", vendor.getOrDefault("openSource", false));
            vendors.put(code, identity);
        }
        return new ArrayList<>(vendors.values());
    }

    public static List<Map<String, Object>> listVendorRegions(ModelCatalog catalog) {
        return catalog.vendors().stream()
                .filter(vendor -> vendor.get("vendorCode") instanceof String)
                .filter(vendor -> vendor.get("regions") instanceof List<?> || vendor.get("regionCode") instanceof String)
                .flatMap(vendor -> regionRefs(vendor).stream())
                .toList();
    }

    public static List<Map<String, Object>> listModels(ModelCatalog catalog) {
        return catalog.models();
    }

    public static List<Map<String, Object>> listModels(ModelCatalog catalog, Map<String, String> filter) {
        return catalog.models().stream()
                .filter(model -> matchesScalar(model, "vendorCode", filter.get("vendorCode")))
                .filter(model -> matchesScalar(model, "regionCode", filter.get("regionCode")))
                .filter(model -> matchesScalar(model, "familyCode", filter.get("familyCode")))
                .filter(model -> containsIfPresent(model, "capabilities", filter.get("capability")))
                .filter(model -> containsIfPresent(model, "inputModalities", filter.get("inputModality")))
                .filter(model -> containsIfPresent(model, "outputModalities", filter.get("outputModality")))
                .filter(model -> matchesScalar(model, "releaseStage", filter.get("releaseStage")))
                .filter(model -> matchesScalar(model, "shelfState", filter.get("shelfState")))
                .filter(model -> matchesScalar(model, "routingState", filter.get("routingState")))
                .filter(model -> matchesScalar(model, "apiFormat", filter.get("apiFormat")))
                .toList();
    }

    public static List<Map<String, Object>> listAvailableModels(ModelCatalog catalog) {
        return listAvailableModels(catalog, Map.of());
    }

    public static List<Map<String, Object>> listAvailableModels(ModelCatalog catalog, Map<String, String> filter) {
        Map<String, String> normalized = new LinkedHashMap<>(filter);
        normalized.put("routingState", "enabled");
        normalized.put("shelfState", "listed");
        return listModels(catalog, normalized).stream()
                .filter(model -> model.get("catalogKey") instanceof String key
                        && !getModelPrices(catalog, key).isEmpty())
                .toList();
    }

    public static String catalogKey(String vendorCode, String regionCode, String modelId) {
        return vendorCode + "/" + regionCode + "/" + modelId;
    }

    public static List<Map<String, Object>> listMeters(ModelCatalog catalog) {
        return catalog.meters();
    }

    public static Map<String, Object> findMeter(ModelCatalog catalog, String meterCode) {
        return catalog.meters().stream()
                .filter(meter -> Objects.equals(meter.get("meterCode"), meterCode))
                .findFirst()
                .orElse(null);
    }

    public static Map<String, Object> findModel(ModelCatalog catalog, String catalogKey) {
        String[] parts = catalogKey.split("/", -1);
        if (parts.length != 3 || parts[0].isBlank() || parts[1].isBlank() || parts[2].isBlank()) {
            return null;
        }
        return findModelByVendorRegion(catalog, parts[0], parts[1], parts[2]);
    }

    public static Map<String, Object> findModelByVendorRegion(
            ModelCatalog catalog,
            String vendorCode,
            String regionCode,
            String modelId
    ) {
        return catalog.models().stream()
                .filter(model -> Objects.equals(model.get("vendorCode"), vendorCode))
                .filter(model -> Objects.equals(model.get("regionCode"), regionCode))
                .filter(model -> Objects.equals(model.get("modelId"), modelId))
                .findFirst()
                .orElse(null);
    }

    public static List<Map<String, Object>> getModelPrices(ModelCatalog catalog, String catalogKey) {
        String[] parts = catalogKey.split("/", -1);
        if (parts.length != 3 || parts[0].isBlank() || parts[1].isBlank() || parts[2].isBlank()) {
            return List.of();
        }
        return catalog.pricing().stream()
                .filter(item -> Objects.equals(item.get("vendorCode"), parts[0]))
                .filter(item -> Objects.equals(item.get("regionCode"), parts[1]))
                .filter(item -> Objects.equals(item.get("modelId"), parts[2]))
                .findFirst()
                .map(item -> mapList(item.get("prices")))
                .orElse(List.of());
    }

    public static Map<String, Object> getBestReferencePrice(ModelCatalog catalog, String catalogKey, String meterCode) {
        return getModelPrices(catalog, catalogKey).stream()
                .filter(price -> Objects.equals(price.get("meterCode"), meterCode))
                .findFirst()
                .orElse(null);
    }

    public static List<Map<String, Object>> listModelsByCapability(ModelCatalog catalog, String capability) {
        return catalog.models().stream()
                .filter(model -> containsString(model.get("capabilities"), capability))
                .toList();
    }

    public static List<Map<String, Object>> listModelsByModality(
            ModelCatalog catalog,
            String inputModality,
            String outputModality
    ) {
        return catalog.models().stream()
                .filter(model -> containsString(model.get("inputModalities"), inputModality))
                .filter(model -> containsString(model.get("outputModalities"), outputModality))
                .toList();
    }

    private static boolean containsString(Object value, String expected) {
        if (!(value instanceof List<?> list)) {
            return false;
        }
        return list.stream().anyMatch(item -> Objects.equals(item, expected));
    }

    private static boolean containsIfPresent(Map<String, Object> model, String field, String expected) {
        return expected == null || containsString(model.get(field), expected);
    }

    private static boolean matchesScalar(Map<String, Object> model, String field, String expected) {
        return expected == null || Objects.equals(model.get(field), expected);
    }

    private static List<Map<String, Object>> mapList(Object value) {
        if (!(value instanceof List<?> list)) {
            return List.of();
        }
        List<Map<String, Object>> items = new ArrayList<>();
        for (Object item : list) {
            if (!(item instanceof Map<?, ?> source)) {
                continue;
            }
            Map<String, Object> normalized = new LinkedHashMap<>();
            for (Map.Entry<?, ?> entry : source.entrySet()) {
                if (entry.getKey() instanceof String key) {
                    normalized.put(key, entry.getValue());
                }
            }
            items.add(normalized);
        }
        return items;
    }

    private static List<Map<String, Object>> regionRefs(Map<String, Object> vendor) {
        Object regions = vendor.get("regions");
        if (regions instanceof List<?> list) {
            return list.stream()
                    .filter(region -> region instanceof Map<?, ?>)
                    .map(region -> (Map<?, ?>) region)
                    .filter(region -> region.get("regionCode") instanceof String)
                    .map(region -> Map.of(
                            "vendorCode", vendor.get("vendorCode"),
                            "regionCode", region.get("regionCode")
                    ))
                    .toList();
        }
        Object regionCode = vendor.get("regionCode");
        if (regionCode instanceof String) {
            return List.of(Map.of("vendorCode", vendor.get("vendorCode"), "regionCode", regionCode));
        }
        return List.of();
    }
}
