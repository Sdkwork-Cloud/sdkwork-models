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
            identity.put("clientApiCompatibility", vendor.getOrDefault("clientApiCompatibility", Map.of()));
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

    public static String catalogKey(String vendorCode, String modelId) {
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
        CatalogKeyParts parts = splitCatalogKey(catalogKey);
        if (parts == null) return null;
        return listModels(catalog).stream()
                .filter(model -> Objects.equals(model.get("vendorCode"), parts.vendorCode()))
                .filter(model -> Objects.equals(model.get("modelId"), parts.modelId()))
                .findFirst().orElse(null);
    }

    public static Map<String, Object> findModelByVendorRegion(ModelCatalog catalog, String vendorCode, String regionCode, String modelId) {
        return listModels(catalog, Map.of("vendorCode", vendorCode, "regionCode", regionCode)).stream()
                .filter(model -> Objects.equals(model.get("modelId"), modelId))
                .findFirst().orElse(null);
    }

    public static List<Map<String, Object>> getModelPrices(ModelCatalog catalog, String catalogKey) {
        CatalogKeyParts parts = splitCatalogKey(catalogKey);
        if (parts == null) return List.of();
        return catalog.pricing().stream()
                .filter(item -> Objects.equals(item.get("vendorCode"), parts.vendorCode()))
                .filter(item -> Objects.equals(item.get("modelId"), parts.modelId()))
                .findFirst()
                .map(item -> mapList(item.get("prices")))
                .orElse(List.of());
    }

    public static List<Map<String, Object>> getModelRegionPrices(ModelCatalog catalog, String catalogKey, String regionCode) {
        CatalogKeyParts parts = splitCatalogKey(catalogKey);
        if (parts == null) return List.of();
        return regionalPricing(catalog).stream()
                .filter(item -> Objects.equals(item.get("vendorCode"), parts.vendorCode()))
                .filter(item -> Objects.equals(item.get("regionCode"), regionCode))
                .filter(item -> Objects.equals(item.get("modelId"), parts.modelId()))
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

    public static List<Map<String, Object>> listModelsWithFeature(ModelCatalog catalog, String feature) {
        return listModels(catalog).stream()
                .filter(model -> ModelCapabilities.modelSupportsFeature(model, feature))
                .toList();
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

    public static List<Map<String, Object>> listClientApiCompatibilityByVendor(ModelCatalog catalog, String vendorCode) {
        Map<String, Object> vendor = catalog.vendors().stream()
                .filter(v -> Objects.equals(v.get("vendorCode"), vendorCode))
                .findFirst().orElse(null);
        if (vendor == null) return List.of();
        Object compatibility = vendor.get("clientApiCompatibility");
        if (!(compatibility instanceof Map<?, ?> map)) return List.of();
        return map.values().stream()
                .filter(item -> item instanceof Map<?, ?>)
                .map(item -> normalizeMap((Map<?, ?>) item))
                .toList();
    }

    public static List<Map<String, Object>> listModelsByProtocol(ModelCatalog catalog, String protocolCode) {
        return listModels(catalog, Map.of("apiFormat", protocolCode));
    }

    public static String voiceCatalogKey(String vendorCode, String voiceId) {
        return vendorCode + "/" + voiceId;
    }

    public static List<Map<String, Object>> listVoices(ModelCatalog catalog) {
        return listVoices(catalog, Map.of());
    }

    @SuppressWarnings("unchecked")
    public static List<Map<String, Object>> listVoices(ModelCatalog catalog, Map<String, String> filter) {
        return regionalVoices(catalog).stream()
                .filter(voice -> matchesScalar(voice, "vendorCode", filter.get("vendorCode")))
                .filter(voice -> matchesScalar(voice, "regionCode", filter.get("regionCode")))
                .filter(voice -> matchesVoiceLocale(voice, filter.get("locale")))
                .filter(voice -> matchesVoiceSearch(voice, filter.get("q")))
                .filter(voice -> matchesVoiceModel(catalog, voice, filter.get("modelCatalogKey")))
                .toList();
    }

    public static List<Map<String, Object>> listVoicesForModel(ModelCatalog catalog, String modelCatalogKey) {
        return listVoices(catalog, Map.of("modelCatalogKey", modelCatalogKey));
    }

    public static List<Map<String, Object>> listModelsForVoice(ModelCatalog catalog, String voiceCatalogKey) {
        Set<String> modelKeys = new LinkedHashSet<>();
        for (Map<String, Object> vendorCatalog : catalog.vendorCatalogs()) {
            for (Map<String, Object> binding : mapList(vendorCatalog.get("modelVoiceBindings"))) {
                if (!(binding.get("catalogKey") instanceof String catalogKey)) {
                    continue;
                }
                for (Map<String, Object> entry : mapList(binding.get("bindings"))) {
                    if (Objects.equals(entry.get("voiceKey"), voiceCatalogKey)) {
                        modelKeys.add(catalogKey);
                    }
                }
            }
        }
        return listModels(catalog).stream()
                .filter(model -> model.get("catalogKey") instanceof String key && modelKeys.contains(key))
                .toList();
    }

    public static String videoProfileCatalogKey(String vendorCode, String modelId, String profileCode) {
        return vendorCode + "/" + modelId + "/" + profileCode;
    }

    public static List<Map<String, Object>> listVideoProfiles(ModelCatalog catalog) {
        return listVideoProfiles(catalog, Map.of());
    }

    public static List<Map<String, Object>> listVideoProfiles(ModelCatalog catalog, Map<String, String> filter) {
        List<Map<String, Object>> result = new ArrayList<>();
        String vendorCode = filter.get("vendorCode");
        if (vendorCode == null) {
            vendorCode = filter.get("vendor_code");
        }
        String regionCode = filter.get("regionCode");
        if (regionCode == null) {
            regionCode = filter.get("region_code");
        }
        String modelCatalogKey = filter.get("modelCatalogKey");
        if (modelCatalogKey == null) {
            modelCatalogKey = filter.get("model_catalog_key");
        }
        String generationMode = filter.get("generationMode");
        if (generationMode == null) {
            generationMode = filter.get("generation_mode");
        }
        String durationTierCode = filter.get("durationTierCode");
        if (durationTierCode == null) {
            durationTierCode = filter.get("duration_tier_code");
        }
        String resolution = filter.get("resolution");
        for (Map<String, Object> vendorCatalog : catalog.vendorCatalogs()) {
            if (vendorCode != null && !Objects.equals(vendorCatalog.get("vendorCode"), vendorCode)) {
                continue;
            }
            if (regionCode != null && !Objects.equals(vendorCatalog.get("regionCode"), regionCode)) {
                continue;
            }
            for (Map<String, Object> profileFile : mapList(vendorCatalog.get("modelVideoProfiles"))) {
                if (modelCatalogKey != null && !Objects.equals(profileFile.get("catalogKey"), modelCatalogKey)) {
                    continue;
                }
                for (Map<String, Object> profile : mapList(profileFile.get("profiles"))) {
                    if (generationMode != null && !Objects.equals(profile.get("generationMode"), generationMode)) {
                        continue;
                    }
                    if (durationTierCode != null && !matchesDurationTier(profile, durationTierCode)) {
                        continue;
                    }
                    if (resolution != null && !Objects.equals(profile.get("resolution"), resolution)) {
                        continue;
                    }
                    result.add(profile);
                }
            }
        }
        return result;
    }

    public static List<Map<String, Object>> listVideoProfilesForModel(ModelCatalog catalog, String modelCatalogKey) {
        return listVideoProfiles(catalog, Map.of("modelCatalogKey", modelCatalogKey));
    }

    public static Map<String, Object> findVideoProfile(ModelCatalog catalog, String profileCatalogKey) {
        for (Map<String, Object> profile : listVideoProfiles(catalog)) {
            if (Objects.equals(profile.get("catalogKey"), profileCatalogKey)) {
                return profile;
            }
        }
        return null;
    }

    private static boolean matchesDurationTier(Map<String, Object> profile, String durationTierCode) {
        if (Objects.equals(profile.get("durationTierCode"), durationTierCode)) {
            return true;
        }
        return mapList(profile.get("durationTierCodes")).stream().anyMatch(entry -> Objects.equals(entry, durationTierCode));
    }

    private static List<Map<String, Object>> regionalVoices(ModelCatalog catalog) {
        if (catalog.vendorCatalogs().isEmpty()) {
            return List.of();
        }
        return catalog.vendorCatalogs().stream()
                .flatMap(vendorCatalog -> mapList(vendorCatalog.get("voices")).stream())
                .toList();
    }

    @SuppressWarnings("unchecked")
    private static boolean matchesVoiceLocale(Map<String, Object> voice, String locale) {
        if (locale == null) {
            return true;
        }
        if (Objects.equals(voice.get("primaryLocale"), locale)) {
            return true;
        }
        return mapList(voice.get("supportedLocales")).stream().anyMatch(entry -> Objects.equals(entry, locale));
    }

    private static boolean matchesVoiceSearch(Map<String, Object> voice, String query) {
        if (query == null) {
            return true;
        }
        String normalized = query.toLowerCase();
        Object displayName = voice.get("displayName");
        Object voiceId = voice.get("voiceId");
        return (displayName instanceof String name && name.toLowerCase().contains(normalized))
                || (voiceId instanceof String id && id.toLowerCase().contains(normalized));
    }

    @SuppressWarnings("unchecked")
    private static boolean matchesVoiceModel(ModelCatalog catalog, Map<String, Object> voice, String modelCatalogKey) {
        if (modelCatalogKey == null) {
            return true;
        }
        if (!(voice.get("catalogKey") instanceof String voiceKey)) {
            return false;
        }
        for (Map<String, Object> vendorCatalog : catalog.vendorCatalogs()) {
            for (Map<String, Object> binding : mapList(vendorCatalog.get("modelVoiceBindings"))) {
                if (!Objects.equals(binding.get("catalogKey"), modelCatalogKey)) {
                    continue;
                }
                for (Map<String, Object> entry : mapList(binding.get("bindings"))) {
                    if (Objects.equals(entry.get("voiceKey"), voiceKey)) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    private static CatalogKeyParts splitCatalogKey(String catalogKey) {
        int separatorIndex = catalogKey.indexOf('/');
        if (separatorIndex <= 0 || separatorIndex == catalogKey.length() - 1) return null;
        return new CatalogKeyParts(
                catalogKey.substring(0, separatorIndex),
                catalogKey.substring(separatorIndex + 1)
        );
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

    private record CatalogKeyParts(String vendorCode, String modelId) {
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
            items.add(normalizeMap(source));
        }
        return items;
    }

    private static Map<String, Object> normalizeMap(Map<?, ?> source) {
        Map<String, Object> normalized = new LinkedHashMap<>();
        for (Map.Entry<?, ?> entry : source.entrySet()) {
            if (entry.getKey() instanceof String key) normalized.put(key, entry.getValue());
        }
        return normalized;
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
