typedef JsonObject = Map<String, Object?>;

class ModelCatalog {
  const ModelCatalog({
    required this.catalogVersion,
    required this.schemaVersion,
    required this.meters,
    required this.vendors,
    required this.models,
    required this.pricing,
  });

  final String catalogVersion;
  final String schemaVersion;
  final List<JsonObject> meters;
  // Unique vendor identities. A vendor can appear in multiple region catalogs.
  final List<JsonObject> vendors;
  // Flattened model and pricing facts keyed by vendorCode/regionCode/modelId.
  final List<JsonObject> models;
  final List<JsonObject> pricing;
}
