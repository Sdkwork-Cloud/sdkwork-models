package com.sdkwork.models;

import java.util.List;
import java.util.Map;

/**
 * Dependency-free Java catalog view.
 *
 * <p>{@code vendors} contains unique vendor identities. Model and pricing facts
 * are flattened and keyed by {@code vendorCode/regionCode/modelId}.</p>
 */
public record ModelCatalog(
        String catalogVersion,
        String schemaVersion,
        List<Map<String, Object>> meters,
        List<Map<String, Object>> vendors,
        List<Map<String, Object>> models,
        List<Map<String, Object>> pricing
) {
}
