package com.sdkwork.models;

import java.io.IOException;
import java.math.BigDecimal;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Path;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Stream;

/**
 * Loader contract for Java integrations.
 */
public final class ModelCatalogLoader {
    private static final HttpClient HTTP_CLIENT = HttpClient.newHttpClient();

    private ModelCatalogLoader() {
    }

    public static ModelCatalog loadCatalog(Path root) {
        try {
            Path catalogRoot = root.toAbsolutePath().normalize();
            Map<String, Object> manifest = readJsonObject(catalogRoot.resolve("sdkwork-models.json"));
            Map<String, Object> meterFile = readJsonObject(catalogRoot.resolve("models").resolve("meters.json"));
            Map<String, Object> protocolFile = readJsonObject(catalogRoot.resolve("models").resolve("protocols.json"));
            Map<String, Object> index = readJsonObject(catalogRoot.resolve("models").resolve("index.json"));

            List<Map<String, Object>> vendors = new ArrayList<>();
            Set<String> vendorCodes = new LinkedHashSet<>();
            Map<String, Map<String, Object>> models = new LinkedHashMap<>();
            Map<String, Integer> modelScores = new LinkedHashMap<>();
            List<Map<String, Object>> vendorCatalogs = new ArrayList<>();
            List<Map<String, Object>> pricing = new ArrayList<>();

            for (Map<String, Object> vendorIndex : mapList(index.get("vendors"))) {
                String vendorCode = stringValue(vendorIndex.get("vendorCode"), "vendorCode");
                String regionCode = stringValue(vendorIndex.get("regionCode"), "regionCode");
                Map<String, Object> vendorCatalog = loadVendorCatalog(catalogRoot, vendorCode, regionCode);
                vendorCatalogs.add(vendorCatalog);
                if (vendorCodes.add(vendorCode)) {
                    vendors.add(mapValue(vendorCatalog.get("vendor"), "vendor"));
                }
                for (Map<String, Object> model : mapList(vendorCatalog.get("models"))) {
                    putBestModelIdentity(models, modelScores, vendorCatalog, model);
                }
                pricing.addAll(mapList(vendorCatalog.get("pricing")));
            }

            return new ModelCatalog(
                    stringValue(manifest.get("catalogVersion"), "catalogVersion"),
                    stringValue(manifest.get("schemaVersion"), "schemaVersion"),
                    mapList(meterFile.get("meters")),
                    mapList(protocolFile.get("protocols")),
                    vendors,
                    vendorCatalogs,
                    new ArrayList<>(models.values()),
                    pricing
            );
        } catch (IOException error) {
            throw new IllegalStateException("failed to load sdkwork-models catalog from " + root, error);
        }
    }

    public static ModelCatalog loadCatalog(URI uri) {
        if (uri.getScheme() == null || "file".equalsIgnoreCase(uri.getScheme())) {
            return loadCatalog(Path.of(uri));
        }
        if ("http".equalsIgnoreCase(uri.getScheme()) || "https".equalsIgnoreCase(uri.getScheme())) {
            return loadRemoteCatalog(uri);
        }
        throw new UnsupportedOperationException("Only file, http, and https catalog URIs are supported.");
    }

    public static ModelCatalog loadBundledCatalog() {
        String configuredRoot = System.getProperty("sdkwork.models.catalogRoot");
        if (configuredRoot == null || configuredRoot.isBlank()) {
            configuredRoot = System.getenv("SDKWORK_MODELS_CATALOG_ROOT");
        }
        if (configuredRoot != null && !configuredRoot.isBlank()) {
            return loadCatalog(Path.of(configuredRoot));
        }
        Path workspaceRoot = Path.of("data", "sdkwork-models");
        if (Files.isRegularFile(workspaceRoot.resolve("sdkwork-models.json"))) {
            return loadCatalog(workspaceRoot);
        }
        throw new UnsupportedOperationException(
                "Bundled Java catalog resources are produced during release publishing; set sdkwork.models.catalogRoot or SDKWORK_MODELS_CATALOG_ROOT for local loading."
        );
    }

    public static Map<String, Object> loadVendorCatalog(Path root, String vendorCode, String regionCode) {
        try {
            Path catalogRoot = root.toAbsolutePath().normalize();
            Map<String, Object> index = readJsonObject(catalogRoot.resolve("models").resolve("index.json"));
            Map<String, Object> vendorIndex = findVendorIndex(index, vendorCode, regionCode);
            Map<String, Object> vendor = readJsonObject(catalogRoot.resolve("models").resolve(stringValue(vendorIndex.get("path"), "path")));
            Map<String, Object> families = readJsonObject(catalogRoot.resolve("models").resolve(stringValue(vendorIndex.get("familiesPath"), "familiesPath")));
            Map<String, Object> result = new LinkedHashMap<>();
            result.put("vendorCode", vendorCode);
            result.put("regionCode", regionCode);
            result.put("vendor", vendor);
            result.put("families", families);
            result.put("models", readJsonObjectsByRef(catalogRoot, mapListOfString(vendorIndex.get("modelFiles"))));
            result.put("pricing", readJsonObjectsByRef(catalogRoot, mapListOfString(vendorIndex.get("pricingFiles"))));
            return result;
        } catch (IOException error) {
            throw new IllegalStateException(
                    "failed to load sdkwork-models vendor catalog " + vendorCode + "/" + regionCode + " from " + root,
                    error
            );
        }
    }

    public static Map<String, Object> loadVendorCatalog(URI root, String vendorCode, String regionCode) {
        if (root.getScheme() == null || "file".equalsIgnoreCase(root.getScheme())) {
            return loadVendorCatalog(Path.of(root), vendorCode, regionCode);
        }
        if (!"http".equalsIgnoreCase(root.getScheme()) && !"https".equalsIgnoreCase(root.getScheme())) {
            throw new UnsupportedOperationException("Only file, http, and https catalog URIs are supported.");
        }
        Map<String, Object> index = readRemoteJsonObject(root, "models/index.json");
        Map<String, Object> vendorIndex = findVendorIndex(index, vendorCode, regionCode);
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("vendorCode", vendorCode);
        result.put("regionCode", regionCode);
        result.put("vendor", readRemoteJsonObject(root, "models/" + stringValue(vendorIndex.get("path"), "path")));
        result.put("families", readRemoteJsonObject(root, "models/" + stringValue(vendorIndex.get("familiesPath"), "familiesPath")));
        result.put("models", readRemoteJsonObjects(root, mapListOfString(vendorIndex.get("modelFiles"))));
        result.put("pricing", readRemoteJsonObjects(root, mapListOfString(vendorIndex.get("pricingFiles"))));
        return result;
    }

    public static ModelCatalog fromParts(
            String catalogVersion,
            String schemaVersion,
            List<Map<String, Object>> meters,
            List<Map<String, Object>> vendors,
            List<Map<String, Object>> models,
            List<Map<String, Object>> pricing
    ) {
        return fromParts(catalogVersion, schemaVersion, meters, List.of(), vendors, models, pricing);
    }

    public static ModelCatalog fromParts(
            String catalogVersion,
            String schemaVersion,
            List<Map<String, Object>> meters,
            List<Map<String, Object>> protocols,
            List<Map<String, Object>> vendors,
            List<Map<String, Object>> models,
            List<Map<String, Object>> pricing
    ) {
        return fromParts(catalogVersion, schemaVersion, meters, protocols, vendors, List.of(), models, pricing);
    }

    public static ModelCatalog fromParts(
            String catalogVersion,
            String schemaVersion,
            List<Map<String, Object>> meters,
            List<Map<String, Object>> protocols,
            List<Map<String, Object>> vendors,
            List<Map<String, Object>> vendorCatalogs,
            List<Map<String, Object>> models,
            List<Map<String, Object>> pricing
    ) {
        List<Map<String, Object>> effectiveVendorCatalogs = vendorCatalogs;
        if (effectiveVendorCatalogs.isEmpty()) {
            effectiveVendorCatalogs = vendorCatalogsFromFlatParts(vendors, models, pricing);
        }
        Map<String, Map<String, Object>> canonicalModels = new LinkedHashMap<>();
        Map<String, Integer> canonicalModelScores = new LinkedHashMap<>();
        for (Map<String, Object> vendorCatalog : effectiveVendorCatalogs) {
            for (Map<String, Object> model : mapList(vendorCatalog.get("models"))) {
                putBestModelIdentity(canonicalModels, canonicalModelScores, vendorCatalog, model);
            }
        }
        if (canonicalModels.isEmpty()) {
            for (Map<String, Object> model : models) {
                putBestModelIdentity(canonicalModels, canonicalModelScores, Map.of("pricing", pricing), model);
            }
        }
        return new ModelCatalog(
                catalogVersion,
                schemaVersion,
                meters,
                protocols,
                vendors,
                effectiveVendorCatalogs,
                new ArrayList<>(canonicalModels.values()),
                pricing
        );
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> readJsonObject(Path path) throws IOException {
        Object value = new JsonParser(Files.readString(path)).parse();
        if (value instanceof Map<?, ?> map) {
            return (Map<String, Object>) map;
        }
        throw new IllegalArgumentException("JSON root must be an object: " + path);
    }

    private static List<Map<String, Object>> readJsonObjects(Path directory) throws IOException {
        try (Stream<Path> stream = Files.list(directory)) {
            return stream
                    .filter(path -> path.getFileName().toString().endsWith(".json"))
                    .sorted()
                    .map(path -> {
                        try {
                            return readJsonObject(path);
                        } catch (IOException error) {
                            throw new IllegalStateException("failed to load JSON file " + path, error);
                        }
                    })
                    .toList();
        }
    }

    private static List<Map<String, Object>> readJsonObjectsByRef(Path catalogRoot, List<String> refs) throws IOException {
        List<Map<String, Object>> result = new ArrayList<>();
        for (String ref : refs) {
            result.add(readJsonObject(catalogRoot.resolve("models").resolve(ref)));
        }
        return result;
    }

    private static ModelCatalog loadRemoteCatalog(URI root) {
        Map<String, Object> manifest = readRemoteJsonObject(root, "sdkwork-models.json");
        Map<String, Object> meterFile = readRemoteJsonObject(root, "models/meters.json");
        Map<String, Object> protocolFile = readRemoteJsonObject(root, "models/protocols.json");
        Map<String, Object> index = readRemoteJsonObject(root, "models/index.json");
        List<Map<String, Object>> vendors = new ArrayList<>();
        Set<String> vendorCodes = new LinkedHashSet<>();
        Map<String, Map<String, Object>> models = new LinkedHashMap<>();
        Map<String, Integer> modelScores = new LinkedHashMap<>();
        List<Map<String, Object>> vendorCatalogs = new ArrayList<>();
        List<Map<String, Object>> pricing = new ArrayList<>();
        for (Map<String, Object> vendorIndex : mapList(index.get("vendors"))) {
            String vendorCode = stringValue(vendorIndex.get("vendorCode"), "vendorCode");
            String regionCode = stringValue(vendorIndex.get("regionCode"), "regionCode");
            Map<String, Object> vendorCatalog = loadVendorCatalog(root, vendorCode, regionCode);
            vendorCatalogs.add(vendorCatalog);
            if (vendorCodes.add(vendorCode)) {
                vendors.add(mapValue(vendorCatalog.get("vendor"), "vendor"));
            }
            for (Map<String, Object> model : mapList(vendorCatalog.get("models"))) {
                putBestModelIdentity(models, modelScores, vendorCatalog, model);
            }
            pricing.addAll(mapList(vendorCatalog.get("pricing")));
        }
        return new ModelCatalog(
                stringValue(manifest.get("catalogVersion"), "catalogVersion"),
                stringValue(manifest.get("schemaVersion"), "schemaVersion"),
                mapList(meterFile.get("meters")),
                mapList(protocolFile.get("protocols")),
                vendors,
                vendorCatalogs,
                new ArrayList<>(models.values()),
                pricing
        );
    }

    private static void putBestModelIdentity(
            Map<String, Map<String, Object>> models,
            Map<String, Integer> scores,
            Map<String, Object> vendorCatalog,
            Map<String, Object> model
    ) {
        Object keyValue = model.get("catalogKey");
        if (!(keyValue instanceof String catalogKey) || catalogKey.isBlank()) {
            return;
        }
        int score = modelIdentityScore(vendorCatalog, model);
        Integer existingScore = scores.get(catalogKey);
        if (existingScore == null || score > existingScore) {
            models.put(catalogKey, model);
            scores.put(catalogKey, score);
        }
    }

    private static int modelIdentityScore(Map<String, Object> vendorCatalog, Map<String, Object> model) {
        int score = 0;
        if (hasRegionPricing(vendorCatalog, model)) {
            score += 100;
        }
        if (Objects.equals(model.get("routingState"), "enabled")) {
            score += 40;
        }
        if (Objects.equals(model.get("shelfState"), "listed")) {
            score += 20;
        }
        if (Objects.equals(model.get("releaseStage"), "active")) {
            score += 10;
        }
        if (Objects.equals(model.get("lifecycle"), "current") || Objects.equals(model.get("lifecycle"), "preview")) {
            score += 5;
        }
        if (Objects.equals(model.get("regionCode"), "global")) {
            score += 1;
        }
        return score;
    }

    private static boolean hasRegionPricing(Map<String, Object> vendorCatalog, Map<String, Object> model) {
        Object modelId = model.get("modelId");
        if (!(modelId instanceof String)) {
            return false;
        }
        return mapList(vendorCatalog.get("pricing")).stream()
                .anyMatch(pricing -> Objects.equals(pricing.get("modelId"), modelId) && !mapList(pricing.get("prices")).isEmpty());
    }

    private static List<Map<String, Object>> vendorCatalogsFromFlatParts(
            List<Map<String, Object>> vendors,
            List<Map<String, Object>> models,
            List<Map<String, Object>> pricing
    ) {
        Map<String, Map<String, Object>> byRegion = new LinkedHashMap<>();
        for (Map<String, Object> model : models) {
            String vendorCode = stringOrNull(model.get("vendorCode"));
            String regionCode = stringOrNull(model.get("regionCode"));
            if (vendorCode == null || regionCode == null) {
                continue;
            }
            Map<String, Object> vendorCatalog = byRegion.computeIfAbsent(
                    vendorCode + "/" + regionCode,
                    ignored -> newVendorCatalog(vendors, vendorCode, regionCode)
            );
            mapListMutable(vendorCatalog, "models").add(model);
        }
        for (Map<String, Object> price : pricing) {
            String vendorCode = stringOrNull(price.get("vendorCode"));
            String regionCode = stringOrNull(price.get("regionCode"));
            if (vendorCode == null || regionCode == null) {
                continue;
            }
            Map<String, Object> vendorCatalog = byRegion.computeIfAbsent(
                    vendorCode + "/" + regionCode,
                    ignored -> newVendorCatalog(vendors, vendorCode, regionCode)
            );
            mapListMutable(vendorCatalog, "pricing").add(price);
        }
        return new ArrayList<>(byRegion.values());
    }

    private static Map<String, Object> newVendorCatalog(
            List<Map<String, Object>> vendors,
            String vendorCode,
            String regionCode
    ) {
        Map<String, Object> vendorCatalog = new LinkedHashMap<>();
        vendorCatalog.put("vendorCode", vendorCode);
        vendorCatalog.put("regionCode", regionCode);
        vendorCatalog.put(
                "vendor",
                vendors.stream()
                        .filter(vendor -> Objects.equals(vendor.get("vendorCode"), vendorCode))
                        .findFirst()
                        .orElse(Map.of("vendorCode", vendorCode))
        );
        vendorCatalog.put("families", Map.of());
        vendorCatalog.put("models", new ArrayList<Map<String, Object>>());
        vendorCatalog.put("pricing", new ArrayList<Map<String, Object>>());
        return vendorCatalog;
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> mapListMutable(Map<String, Object> map, String field) {
        Object value = map.get(field);
        if (value instanceof List<?> list) {
            return (List<Map<String, Object>>) list;
        }
        List<Map<String, Object>> result = new ArrayList<>();
        map.put(field, result);
        return result;
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> readRemoteJsonObject(URI root, String relPath) {
        try {
            URI uri = root.resolve(root.toString().endsWith("/") ? relPath : "/" + relPath);
            HttpRequest request = HttpRequest.newBuilder(uri).GET().build();
            HttpResponse<String> response = HTTP_CLIENT.send(request, HttpResponse.BodyHandlers.ofString());
            if (response.statusCode() < 200 || response.statusCode() >= 300) {
                throw new IllegalStateException("failed to fetch sdkwork-models catalog file " + relPath + ": " + response.statusCode());
            }
            Object value = new JsonParser(response.body()).parse();
            if (value instanceof Map<?, ?> map) {
                return (Map<String, Object>) map;
            }
            throw new IllegalArgumentException("JSON root must be an object: " + uri);
        } catch (IOException error) {
            throw new IllegalStateException("failed to fetch sdkwork-models catalog file " + relPath, error);
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException("interrupted while fetching sdkwork-models catalog file " + relPath, error);
        }
    }

    private static List<Map<String, Object>> readRemoteJsonObjects(URI root, List<String> refs) {
        List<Map<String, Object>> result = new ArrayList<>();
        for (String ref : refs) {
            result.add(readRemoteJsonObject(root, "models/" + ref));
        }
        return result;
    }

    private static Map<String, Object> findVendorIndex(Map<String, Object> index, String vendorCode, String regionCode) {
        return mapList(index.get("vendors")).stream()
                .filter(item -> vendorCode.equals(item.get("vendorCode")) && regionCode.equals(item.get("regionCode")))
                .findFirst()
                .orElseThrow(() -> new IllegalArgumentException("vendor region " + vendorCode + "/" + regionCode + " is not indexed"));
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> mapList(Object value) {
        if (!(value instanceof List<?> list)) {
            return List.of();
        }
        List<Map<String, Object>> result = new ArrayList<>();
        for (Object item : list) {
            if (item instanceof Map<?, ?> map) {
                result.add((Map<String, Object>) map);
            }
        }
        return result;
    }

    private static List<String> mapListOfString(Object value) {
        if (!(value instanceof List<?> list)) {
            return List.of();
        }
        List<String> result = new ArrayList<>();
        for (Object item : list) {
            if (item instanceof String text) {
                result.add(text);
            }
        }
        return result;
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> mapValue(Object value, String field) {
        if (value instanceof Map<?, ?> map) {
            return (Map<String, Object>) map;
        }
        throw new IllegalArgumentException(field + " must be an object");
    }

    private static String stringValue(Object value, String field) {
        if (value instanceof String text && !text.isBlank()) {
            return text;
        }
        throw new IllegalArgumentException(field + " must be a non-empty string");
    }

    private static String stringOrNull(Object value) {
        return value instanceof String text && !text.isBlank() ? text : null;
    }

    private static final class JsonParser {
        private final String source;
        private int index;

        private JsonParser(String source) {
            this.source = source;
        }

        private Object parse() {
            Object value = parseValue();
            skipWhitespace();
            if (index != source.length()) {
                throw error("unexpected trailing content");
            }
            return value;
        }

        private Object parseValue() {
            skipWhitespace();
            if (index >= source.length()) {
                throw error("unexpected end of JSON");
            }
            char ch = source.charAt(index);
            return switch (ch) {
                case '{' -> parseObject();
                case '[' -> parseArray();
                case '"' -> parseString();
                case 't' -> parseLiteral("true", Boolean.TRUE);
                case 'f' -> parseLiteral("false", Boolean.FALSE);
                case 'n' -> parseLiteral("null", null);
                default -> parseNumber();
            };
        }

        private Map<String, Object> parseObject() {
            expect('{');
            Map<String, Object> object = new LinkedHashMap<>();
            skipWhitespace();
            if (tryConsume('}')) {
                return object;
            }
            do {
                String key = parseString();
                skipWhitespace();
                expect(':');
                object.put(key, parseValue());
                skipWhitespace();
            } while (tryConsume(','));
            expect('}');
            return object;
        }

        private List<Object> parseArray() {
            expect('[');
            List<Object> array = new ArrayList<>();
            skipWhitespace();
            if (tryConsume(']')) {
                return array;
            }
            do {
                array.add(parseValue());
                skipWhitespace();
            } while (tryConsume(','));
            expect(']');
            return array;
        }

        private String parseString() {
            expect('"');
            StringBuilder builder = new StringBuilder();
            while (index < source.length()) {
                char ch = source.charAt(index++);
                if (ch == '"') {
                    return builder.toString();
                }
                if (ch != '\\') {
                    builder.append(ch);
                    continue;
                }
                if (index >= source.length()) {
                    throw error("unterminated escape sequence");
                }
                char escape = source.charAt(index++);
                switch (escape) {
                    case '"' -> builder.append('"');
                    case '\\' -> builder.append('\\');
                    case '/' -> builder.append('/');
                    case 'b' -> builder.append('\b');
                    case 'f' -> builder.append('\f');
                    case 'n' -> builder.append('\n');
                    case 'r' -> builder.append('\r');
                    case 't' -> builder.append('\t');
                    case 'u' -> builder.append(parseUnicodeEscape());
                    default -> throw error("unsupported escape sequence");
                }
            }
            throw error("unterminated string");
        }

        private char parseUnicodeEscape() {
            if (index + 4 > source.length()) {
                throw error("incomplete unicode escape");
            }
            String hex = source.substring(index, index + 4);
            index += 4;
            return (char) Integer.parseInt(hex, 16);
        }

        private Object parseNumber() {
            int start = index;
            if (source.charAt(index) == '-') {
                index++;
            }
            while (index < source.length() && Character.isDigit(source.charAt(index))) {
                index++;
            }
            if (index < source.length() && source.charAt(index) == '.') {
                index++;
                while (index < source.length() && Character.isDigit(source.charAt(index))) {
                    index++;
                }
            }
            if (index < source.length() && (source.charAt(index) == 'e' || source.charAt(index) == 'E')) {
                index++;
                if (index < source.length() && (source.charAt(index) == '+' || source.charAt(index) == '-')) {
                    index++;
                }
                while (index < source.length() && Character.isDigit(source.charAt(index))) {
                    index++;
                }
            }
            if (start == index) {
                throw error("expected JSON value");
            }
            String number = source.substring(start, index);
            if (number.contains(".") || number.contains("e") || number.contains("E")) {
                return new BigDecimal(number);
            }
            return Long.parseLong(number);
        }

        private Object parseLiteral(String literal, Object value) {
            if (!source.startsWith(literal, index)) {
                throw error("invalid literal");
            }
            index += literal.length();
            return value;
        }

        private void skipWhitespace() {
            while (index < source.length() && Character.isWhitespace(source.charAt(index))) {
                index++;
            }
        }

        private void expect(char expected) {
            skipWhitespace();
            if (index >= source.length() || source.charAt(index) != expected) {
                throw error("expected '" + expected + "'");
            }
            index++;
        }

        private boolean tryConsume(char expected) {
            skipWhitespace();
            if (index < source.length() && source.charAt(index) == expected) {
                index++;
                return true;
            }
            return false;
        }

        private IllegalArgumentException error(String message) {
            return new IllegalArgumentException(message + " at offset " + index);
        }
    }
}
