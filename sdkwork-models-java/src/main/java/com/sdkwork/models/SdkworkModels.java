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

    public static String catalogKey(String vendorCode, String regionCode, String modelId) {
        return ModelCatalogQuery.catalogKey(vendorCode, regionCode, modelId);
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
}
