package com.sdkwork.models;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;

public final class ModelCatalogQuery {
    private ModelCatalogQuery() {}

    @SuppressWarnings("unchecked")
    public static List<Map<String, Object>> listVendors(ModelCatalog catalog) {
        Map<String, Map<String, Object>> vendors = new LinkedHashMap<>();
        for (Map<String, Object> vendor : catalog.vendors()) {
            Object vendorCode = vendor.get("vendorCode");
            if (!(vendorCode instanceof String code) || vendors.containsKey(code)) continue;
            Map<String, Object> identity = new LinkedHashMap<>();
            identity.put("vendorCode", code);
            identity.put("displayName", vendor.get("displayName"));
            identity.put("legalName", vendor.get("legalName"));
            identity.put("vendorType", vendor.get("vendorType"));
            identity.put("capabilities", vendor.getOrDefault("capabilities", List.of()));
            identity.put("supportedProtocols", vendor.getOrDefault("supportedProtocols", List.of()));
            identity.put("openSource", vendor.getOrDefault("openSource", false));
            vendors.put(code, identity);
        }
        return new ArrayList<>(vendors.values());
    }

    public static List<Map<String, Object>> listVendorRegions(ModelCatalog catalog) {
        List<Map<String, Object>> regions = catalog.vendorCatalogs().stream()
                .filter(vendor -> vendor.get("vendorCode") instanceof String)
                .filter(vendor -> vendor.get("regionCode") instanceof String)
                .map(vendor -> Map.of("vendorCode", vendor.get("vendorCode"), "regionCode", vendor.get("regionCode")))
                .toList();
        if (!regions.isEmpty()) {
            return regions;
        }
        return catalog.vendors().stream()
                .filter(vendor -> vendor.get("vendorCode") instanceof String)
                .filter(vendor -> vendor.get("regions") instanceof List<?> || vendor.get("regionCode") instanceof String)
                .flatMap(vendor -> regionRefs(vendor).stream())
                .toList();
    }

    public static List<Map<String, Object>> listModels(ModelCatalog catalog) {
        return listModels(catalog, Map.of());
    }

    public static List<Map<String, Object>> listModels(ModelCatalog catalog, Map<String, String> filter) {
        List<ModelObservation> matches = regionalModels(catalog).stream()
                .filter(item -> matchesScalar(item.model(), "vendorCode", filter.get("vendorCode")))
                .filter(item -> matchesScalar(item.model(), "regionCode", filter.get("regionCode")))
                .filter(item -> matchesScalar(item.model(), "familyCode", filter.get("familyCode")))
                .filter(item -> containsIfPresent(item.model(), "capabilities", filter.get("capability")))
                .filter(item -> containsIfPresent(item.model(), "inputModalities", filter.get("inputModality")))
                .filter(item -> containsIfPresent(item.model(), "outputModalities", filter.get("outputModality")))
                .filter(item -> matchesScalar(item.model(), "releaseStage", filter.get("releaseStage")))
                .filter(item -> matchesScalar(item.model(), "shelfState", filter.get("shelfState")))
                .filter(item -> matchesScalar(item.model(), "routingState", filter.get("routingState")))
                .filter(item -> matchesScalar(item.model(), "apiFormat", filter.get("apiFormat")))
                .toList();
        if (filter.get("regionCode") != null) {
            return matches.stream().map(ModelObservation::model).toList();
        }
        Map<String, ModelObservation> deduped = new LinkedHashMap<>();
        for (ModelObservation item : matches) {
            Object keyValue = item.model().get("catalogKey");
            if (!(keyValue instanceof String key)) {
                continue;
            }
            ModelObservation existing = deduped.get(key);
            if (existing == null || modelIdentityScore(item) > modelIdentityScore(existing)) {
                deduped.put(key, item);
            }
        }
        return deduped.values().stream().map(ModelObservation::model).toList();
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
                        && model.get("regionCode") instanceof String regionCode
                        && !getModelRegionPrices(catalog, key, regionCode).isEmpty())
                .toList();
    }

    public static String catalogKey(String vendorCode, String regionCode, String modelId) {
        return vendorCode + "/" + modelId;
    }

    public static List<Map<String, Object>> listMeters(ModelCatalog catalog) {
        return catalog.meters();
    }

    public static Map<String, Object> findMeter(ModelCatalog catalog, String meterCode) {
        return catalog.meters().stream()
                .filter(meter -> Objects.equals(meter.get("meterCode"), meterCode))
                .findFirst().orElse(null);
    }

    public static Map<String, Object> findModel(ModelCatalog catalog, String catalogKey) {
        String[] parts = catalogKey.split("/", -1);
        if (parts.length != 2 || parts[0].isBlank() || parts[1].isBlank()) return null;
        return listModels(catalog).stream()
                .filter(model -> Objects.equals(model.get("vendorCode"), parts[0]))
                .filter(model -> Objects.equals(model.get("modelId"), parts[1]))
                .findFirst().orElse(null);
    }

    public static Map<String, Object> findModelByVendorRegion(ModelCatalog catalog, String vendorCode, String regionCode, String modelId) {
        return listModels(catalog, Map.of("vendorCode", vendorCode, "regionCode", regionCode)).stream()
                .filter(model -> Objects.equals(model.get("modelId"), modelId))
                .findFirst().orElse(null);
    }

    public static List<Map<String, Object>> getModelPrices(ModelCatalog catalog, String catalogKey) {
        String[] parts = catalogKey.split("/", -1);
        if (parts.length != 2 || parts[0].isBlank() || parts[1].isBlank()) return List.of();
        return catalog.pricing().stream()
                .filter(item -> Objects.equals(item.get("vendorCode"), parts[0]))
                .filter(item -> Objects.equals(item.get("modelId"), parts[1]))
                .findFirst()
                .map(item -> mapList(item.get("prices")))
                .orElse(List.of());
    }

    public static List<Map<String, Object>> getModelRegionPrices(ModelCatalog catalog, String catalogKey, String regionCode) {
        String[] parts = catalogKey.split("/", -1);
        if (parts.length != 2 || parts[0].isBlank() || parts[1].isBlank()) return List.of();
        return regionalPricing(catalog).stream()
                .filter(item -> Objects.equals(item.get("vendorCode"), parts[0]))
                .filter(item -> Objects.equals(item.get("regionCode"), regionCode))
                .filter(item -> Objects.equals(item.get("modelId"), parts[1]))
                .findFirst()
                .map(item -> mapList(item.get("prices")))
                .orElse(List.of());
    }

    public static Map<String, Object> getBestReferencePrice(ModelCatalog catalog, String catalogKey, String meterCode) {
        return getModelPrices(catalog, catalogKey).stream()
                .filter(price -> Objects.equals(price.get("meterCode"), meterCode))
                .findFirst().orElse(null);
    }

    public static List<Map<String, Object>> listModelsByCapability(ModelCatalog catalog, String capability) {
        return listModels(catalog, Map.of("capability", capability));
    }

    public static List<Map<String, Object>> listModelsByModality(ModelCatalog catalog, String inputModality, String outputModality) {
        return listModels(catalog, Map.of("inputModality", inputModality, "outputModality", outputModality));
    }

    public static List<Map<String, Object>> listProtocols(ModelCatalog catalog) {
        return catalog.protocols();
    }

    @SuppressWarnings("unchecked")
    public static Map<String, Object> findProtocol(ModelCatalog catalog, String protocolCode) {
        return catalog.protocols().stream()
                .filter(p -> Objects.equals(p.get("protocolCode"), protocolCode))
                .findFirst().orElse(null);
    }

    @SuppressWarnings("unchecked")
    public static List<Map<String, Object>> listProtocolsByVendor(ModelCatalog catalog, String vendorCode) {
        Map<String, Object> vendor = catalog.vendors().stream()
                .filter(v -> Objects.equals(v.get("vendorCode"), vendorCode))
                .findFirst().orElse(null);
        if (vendor == null) return List.of();
        Object sp = vendor.get("supportedProtocols");
        if (!(sp instanceof List<?> list)) return List.of();
        Set<String> supported = new LinkedHashSet<>();
        for (Object item : list) {
            if (item instanceof String s) supported.add(s);
        }
        return catalog.protocols().stream()
                .filter(p -> supported.contains(p.get("protocolCode")))
                .toList();
    }

    public static List<Map<String, Object>> listModelsByProtocol(ModelCatalog catalog, String protocolCode) {
        return listModels(catalog, Map.of("apiFormat", protocolCode));
    }

    private static List<ModelObservation> regionalModels(ModelCatalog catalog) {
        if (catalog.vendorCatalogs().isEmpty()) {
            return catalog.models().stream()
                    .map(model -> new ModelObservation(model, hasFlatPricing(catalog, model)))
                    .toList();
        }
        List<ModelObservation> models = new ArrayList<>();
        for (Map<String, Object> vendorCatalog : catalog.vendorCatalogs()) {
            for (Map<String, Object> model : mapList(vendorCatalog.get("models"))) {
                models.add(new ModelObservation(model, hasRegionPricing(vendorCatalog, model)));
            }
        }
        return models;
    }

    private static List<Map<String, Object>> regionalPricing(ModelCatalog catalog) {
        if (catalog.vendorCatalogs().isEmpty()) {
            return catalog.pricing();
        }
        return catalog.vendorCatalogs().stream()
                .flatMap(vendorCatalog -> mapList(vendorCatalog.get("pricing")).stream())
                .toList();
    }

    private static boolean hasRegionPricing(Map<String, Object> vendorCatalog, Map<String, Object> model) {
        Object modelId = model.get("modelId");
        return modelId instanceof String && mapList(vendorCatalog.get("pricing")).stream()
                .anyMatch(pricing -> Objects.equals(pricing.get("modelId"), modelId) && !mapList(pricing.get("prices")).isEmpty());
    }

    private static boolean hasFlatPricing(ModelCatalog catalog, Map<String, Object> model) {
        Object catalogKey = model.get("catalogKey");
        return catalogKey instanceof String key && !getModelPrices(catalog, key).isEmpty();
    }

    private static int modelIdentityScore(ModelObservation item) {
        Map<String, Object> model = item.model();
        int score = 0;
        if (item.hasRegionPricing()) score += 100;
        if (Objects.equals(model.get("routingState"), "enabled")) score += 40;
        if (Objects.equals(model.get("shelfState"), "listed")) score += 20;
        if (Objects.equals(model.get("releaseStage"), "active")) score += 10;
        if (Objects.equals(model.get("lifecycle"), "current") || Objects.equals(model.get("lifecycle"), "preview")) {
            score += 5;
        }
        if (Objects.equals(model.get("regionCode"), "global")) score += 1;
        return score;
    }

    private record ModelObservation(Map<String, Object> model, boolean hasRegionPricing) {
    }

    private static boolean containsString(Object value, String expected) {
        if (!(value instanceof List<?> list)) return false;
        return list.stream().anyMatch(item -> Objects.equals(item, expected));
    }

    private static boolean containsIfPresent(Map<String, Object> model, String field, String expected) {
        return expected == null || containsString(model.get(field), expected);
    }

    private static boolean matchesScalar(Map<String, Object> model, String field, String expected) {
        return expected == null || Objects.equals(model.get(field), expected);
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> mapList(Object value) {
        if (!(value instanceof List<?> list)) return List.of();
        List<Map<String, Object>> items = new ArrayList<>();
        for (Object item : list) {
            if (!(item instanceof Map<?, ?> source)) continue;
            Map<String, Object> normalized = new LinkedHashMap<>();
            for (Map.Entry<?, ?> entry : source.entrySet()) {
                if (entry.getKey() instanceof String key) normalized.put(key, entry.getValue());
            }
            items.add(normalized);
        }
        return items;
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> regionRefs(Map<String, Object> vendor) {
        Object regions = vendor.get("regions");
        if (regions instanceof List<?> list) {
            return list.stream()
                    .filter(region -> region instanceof Map<?, ?>)
                    .map(region -> (Map<?, ?>) region)
                    .filter(region -> region.get("regionCode") instanceof String)
                    .map(region -> Map.of("vendorCode", vendor.get("vendorCode"), "regionCode", region.get("regionCode")))
                    .toList();
        }
        Object regionCode = vendor.get("regionCode");
        if (regionCode instanceof String) return List.of(Map.of("vendorCode", vendor.get("vendorCode"), "regionCode", regionCode));
        return List.of();
    }
}
