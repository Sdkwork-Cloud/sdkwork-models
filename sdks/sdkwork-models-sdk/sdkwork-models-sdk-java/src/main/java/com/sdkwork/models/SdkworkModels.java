package com.sdkwork.models;

import java.net.URI;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

/**
 * Standard Java entrypoint for the sdkwork-models catalog.
 */
public final class SdkworkModels {
    private SdkworkModels() {
    }

    public static ModelCatalog loadCatalog(Path root) {
        return ModelCatalogLoader.loadCatalog(root);
    }

    public static ModelCatalog loadCatalog(URI uri) {
        return ModelCatalogLoader.loadCatalog(uri);
    }

    public static ModelCatalog loadBundledCatalog() {
        return ModelCatalogLoader.loadBundledCatalog();
    }

    public static Map<String, Object> loadVendorCatalog(Path root, String vendorCode, String regionCode) {
        return ModelCatalogLoader.loadVendorCatalog(root, vendorCode, regionCode);
    }

    public static Map<String, Object> findModel(ModelCatalog catalog, String catalogKey) {
        return ModelCatalogQuery.findModel(catalog, catalogKey);
    }

    public static Map<String, Object> findModelByVendorRegion(
            ModelCatalog catalog,
            String vendorCode,
            String regionCode,
            String modelId
    ) {
        return ModelCatalogQuery.findModelByVendorRegion(catalog, vendorCode, regionCode, modelId);
    }

    public static String catalogKey(String vendorCode, String modelId) {
        return ModelCatalogQuery.catalogKey(vendorCode, modelId);
    }

    public static List<Map<String, Object>> listVendors(ModelCatalog catalog) {
        return ModelCatalogQuery.listVendors(catalog);
    }

    public static List<Map<String, Object>> listVendorRegions(ModelCatalog catalog) {
        return ModelCatalogQuery.listVendorRegions(catalog);
    }

    public static List<Map<String, Object>> listModels(ModelCatalog catalog) {
        return ModelCatalogQuery.listModels(catalog);
    }

    public static List<Map<String, Object>> listModels(ModelCatalog catalog, Map<String, String> filter) {
        return ModelCatalogQuery.listModels(catalog, filter);
    }

    public static List<Map<String, Object>> listAvailableModels(ModelCatalog catalog) {
        return ModelCatalogQuery.listAvailableModels(catalog);
    }

    public static List<Map<String, Object>> listAvailableModels(ModelCatalog catalog, Map<String, String> filter) {
        return ModelCatalogQuery.listAvailableModels(catalog, filter);
    }

    public static List<Map<String, Object>> listMeters(ModelCatalog catalog) {
        return ModelCatalogQuery.listMeters(catalog);
    }

    public static Map<String, Object> findMeter(ModelCatalog catalog, String meterCode) {
        return ModelCatalogQuery.findMeter(catalog, meterCode);
    }

    public static List<Map<String, Object>> getModelPrices(ModelCatalog catalog, String catalogKey) {
        return ModelCatalogQuery.getModelPrices(catalog, catalogKey);
    }

    public static List<Map<String, Object>> getModelRegionPrices(
            ModelCatalog catalog,
            String catalogKey,
            String regionCode
    ) {
        return ModelCatalogQuery.getModelRegionPrices(catalog, catalogKey, regionCode);
    }

    public static Map<String, Object> getBestReferencePrice(ModelCatalog catalog, String catalogKey, String meterCode) {
        return ModelCatalogQuery.getBestReferencePrice(catalog, catalogKey, meterCode);
    }

    public static List<Map<String, Object>> listModelsByCapability(ModelCatalog catalog, String capability) {
        return ModelCatalogQuery.listModelsByCapability(catalog, capability);
    }

    public static List<Map<String, Object>> listModelsByModality(
            ModelCatalog catalog,
            String inputModality,
            String outputModality
    ) {
        return ModelCatalogQuery.listModelsByModality(catalog, inputModality, outputModality);
    }

    public static List<Map<String, Object>> listModelsWithFeature(ModelCatalog catalog, String feature) {
        return ModelCatalogQuery.listModelsWithFeature(catalog, feature);
    }

    public static Map<String, Object> getModelCapabilityProfile(Map<String, Object> model) {
        return ModelCapabilities.getModelCapabilityProfile(model);
    }

    public static boolean modelSupportsFeature(Map<String, Object> model, String feature) {
        return ModelCapabilities.modelSupportsFeature(model, feature);
    }

    public static boolean modelSupportsToolCall(Map<String, Object> model) {
        return ModelCapabilities.modelSupportsToolCall(model);
    }

    public static boolean modelSupportsVision(Map<String, Object> model) {
        return ModelCapabilities.modelSupportsVision(model);
    }

    public static List<Map<String, Object>> listProtocols(ModelCatalog catalog) {
        return ModelCatalogQuery.listProtocols(catalog);
    }

    public static Map<String, Object> findProtocol(ModelCatalog catalog, String protocolCode) {
        return ModelCatalogQuery.findProtocol(catalog, protocolCode);
    }

    public static List<Map<String, Object>> listProtocolsByVendor(ModelCatalog catalog, String vendorCode) {
        return ModelCatalogQuery.listProtocolsByVendor(catalog, vendorCode);
    }

    public static List<Map<String, Object>> listClientApiCompatibilityByVendor(ModelCatalog catalog, String vendorCode) {
        return ModelCatalogQuery.listClientApiCompatibilityByVendor(catalog, vendorCode);
    }

    public static List<Map<String, Object>> listModelsByProtocol(ModelCatalog catalog, String protocolCode) {
        return ModelCatalogQuery.listModelsByProtocol(catalog, protocolCode);
    }
}
