typedef JsonObject = Map<String, Object?>;

class ProtocolStandard {
  const ProtocolStandard({
    required this.protocolCode,
    required this.vendorOrigin,
    required this.displayName,
    required this.family,
    required this.docsUrl,
    required this.maturity,
  });

  final String protocolCode;
  final String vendorOrigin;
  final String displayName;
  final String family;
  final String docsUrl;
  final String maturity;
}

class ModelCatalog {
  const ModelCatalog({
    required this.catalogVersion,
    required this.schemaVersion,
    required this.meters,
    required this.protocols,
    required this.vendors,
    required this.vendorCatalogs,
    required this.models,
    required this.pricing,
  });

  final String catalogVersion;
  final String schemaVersion;
  final List<JsonObject> meters;
  final List<JsonObject> protocols;
  final List<JsonObject> vendors;
  final List<JsonObject> vendorCatalogs;
  final List<JsonObject> models;
  final List<JsonObject> pricing;
}
