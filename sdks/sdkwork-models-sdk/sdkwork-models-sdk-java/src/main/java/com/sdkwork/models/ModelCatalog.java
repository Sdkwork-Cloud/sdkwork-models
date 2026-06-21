package com.sdkwork.models;

import java.util.List;
import java.util.Map;

/**
 * Dependency-free Java catalog view.
 *
 * <p>{@code vendors} contains unique vendor identities. {@code vendorCatalogs}
 * preserves regional supply catalogs, while {@code models} exposes canonical
 * {@code vendorCode/modelId} identities.</p>
 */
public record ModelCatalog(
        String catalogVersion,
        String schemaVersion,
        List<Map<String, Object>> meters,
        List<Map<String, Object>> protocols,
        List<Map<String, Object>> vendors,
        List<Map<String, Object>> vendorCatalogs,
        List<Map<String, Object>> models,
        List<Map<String, Object>> pricing
) {
}
