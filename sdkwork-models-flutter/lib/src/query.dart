import 'types.dart';

List<JsonObject> listVendors(ModelCatalog catalog) {
  final vendors = <String, JsonObject>{};
  for (final vendor in catalog.vendors) {
    final vendorCode = vendor['vendorCode'];
    if (vendorCode is! String || vendors.containsKey(vendorCode)) {
      continue;
    }
    vendors[vendorCode] = {
      'vendorCode': vendorCode,
      'displayName': vendor['displayName'],
      'legalName': vendor['legalName'],
      'vendorType': vendor['vendorType'],
      'capabilities': ((vendor['capabilities'] as List?) ?? const []).cast<Object?>(),
      'supportedProtocols': ((vendor['supportedProtocols'] as List?) ?? const []).cast<Object?>(),
      'openSource': vendor['openSource'] ?? false,
    };
  }
  return vendors.values.toList();
}

List<JsonObject> listVendorRegions(ModelCatalog catalog) {
  final regions = <JsonObject>[];
  for (final vendor in catalog.vendors) {
    final vendorCode = vendor['vendorCode'];
    if (vendorCode is! String) {
      continue;
    }
    final vendorRegions = vendor['regions'];
    if (vendorRegions is List) {
      for (final region in vendorRegions) {
        if (region is JsonObject && region['regionCode'] is String) {
          regions.add({'vendorCode': vendorCode, 'regionCode': region['regionCode']});
        }
      }
      continue;
    }
    final regionCode = vendor['regionCode'];
    if (regionCode is String) {
      regions.add({'vendorCode': vendorCode, 'regionCode': regionCode});
    }
  }
  return regions;
}

List<JsonObject> listModels(
  ModelCatalog catalog, {
  JsonObject? filter,
  String? vendorCode,
  String? regionCode,
  String? familyCode,
  String? capability,
  String? inputModality,
  String? outputModality,
  String? releaseStage,
  String? shelfState,
  String? routingState,
  String? apiFormat,
}) {
  return listModelsWhere(
    catalog,
    vendorCode: vendorCode ?? _filterString(filter, 'vendorCode'),
    regionCode: regionCode ?? _filterString(filter, 'regionCode'),
    familyCode: familyCode ?? _filterString(filter, 'familyCode'),
    capability: capability ?? _filterString(filter, 'capability'),
    inputModality: inputModality ?? _filterString(filter, 'inputModality'),
    outputModality: outputModality ?? _filterString(filter, 'outputModality'),
    releaseStage: releaseStage ?? _filterString(filter, 'releaseStage'),
    shelfState: shelfState ?? _filterString(filter, 'shelfState'),
    routingState: routingState ?? _filterString(filter, 'routingState'),
    apiFormat: apiFormat ?? _filterString(filter, 'apiFormat'),
  );
}

List<JsonObject> listModelsWhere(
  ModelCatalog catalog, {
  String? vendorCode,
  String? regionCode,
  String? familyCode,
  String? capability,
  String? inputModality,
  String? outputModality,
  String? releaseStage,
  String? shelfState,
  String? routingState,
  String? apiFormat,
}) {
  return catalog.models.where((model) {
    if (vendorCode != null && model['vendorCode'] != vendorCode) {
      return false;
    }
    if (regionCode != null && model['regionCode'] != regionCode) {
      return false;
    }
    if (familyCode != null && model['familyCode'] != familyCode) {
      return false;
    }
    if (capability != null) {
      final capabilities = (model['capabilities'] as List?) ?? const [];
      if (!capabilities.contains(capability)) {
        return false;
      }
    }
    if (inputModality != null) {
      final inputModalities = (model['inputModalities'] as List?) ?? const [];
      if (!inputModalities.contains(inputModality)) {
        return false;
      }
    }
    if (outputModality != null) {
      final outputModalities = (model['outputModalities'] as List?) ?? const [];
      if (!outputModalities.contains(outputModality)) {
        return false;
      }
    }
    if (releaseStage != null && model['releaseStage'] != releaseStage) {
      return false;
    }
    if (shelfState != null && model['shelfState'] != shelfState) {
      return false;
    }
    if (routingState != null && model['routingState'] != routingState) {
      return false;
    }
    if (apiFormat != null && model['apiFormat'] != apiFormat) {
      return false;
    }
    return true;
  }).toList();
}

List<JsonObject> listAvailableModels(
  ModelCatalog catalog, {
  JsonObject? filter,
  String? vendorCode,
  String? regionCode,
  String? familyCode,
  String? capability,
  String? inputModality,
  String? outputModality,
  String? releaseStage,
  String? apiFormat,
}) {
  return listModels(
    catalog,
    filter: filter,
    vendorCode: vendorCode,
    regionCode: regionCode,
    familyCode: familyCode,
    capability: capability,
    inputModality: inputModality,
    outputModality: outputModality,
    releaseStage: releaseStage,
    shelfState: 'listed',
    routingState: 'enabled',
    apiFormat: apiFormat,
  ).where((model) {
    final catalogKeyValue = model['catalogKey'];
    return catalogKeyValue is String && getModelPrices(catalog, catalogKeyValue).isNotEmpty;
  }).toList();
}

String catalogKey(String vendorCode, String regionCode, String modelId) => '$vendorCode/$regionCode/$modelId';

List<JsonObject> listMeters(ModelCatalog catalog) => catalog.meters;

JsonObject? findMeter(ModelCatalog catalog, String meterCode) {
  for (final meter in catalog.meters) {
    if (meter['meterCode'] == meterCode) {
      return meter;
    }
  }
  return null;
}

JsonObject? findModel(ModelCatalog catalog, String catalogKeyValue) {
  final parts = catalogKeyValue.split('/');
  if (parts.length != 3 || parts[0].isEmpty || parts[1].isEmpty || parts[2].isEmpty) {
    return null;
  }
  return findModelByVendorRegion(catalog, parts[0], parts[1], parts[2]);
}

JsonObject? findModelByVendorRegion(ModelCatalog catalog, String vendorCode, String regionCode, String modelId) {
  for (final model in catalog.models) {
    if (model['vendorCode'] == vendorCode && model['regionCode'] == regionCode && model['modelId'] == modelId) {
      return model;
    }
  }
  return null;
}

List<JsonObject> getModelPrices(ModelCatalog catalog, String catalogKeyValue) {
  final parts = catalogKeyValue.split('/');
  if (parts.length != 3 || parts[0].isEmpty || parts[1].isEmpty || parts[2].isEmpty) {
    return const [];
  }
  final vendorCode = parts[0];
  final regionCode = parts[1];
  final modelId = parts[2];
  for (final item in catalog.pricing) {
    if (item['vendorCode'] == vendorCode && item['regionCode'] == regionCode && item['modelId'] == modelId) {
      return ((item['prices'] as List?) ?? const []).cast<JsonObject>();
    }
  }
  return const [];
}

JsonObject? getBestReferencePrice(ModelCatalog catalog, String catalogKeyValue, String meterCode) {
  for (final price in getModelPrices(catalog, catalogKeyValue)) {
    if (price['meterCode'] == meterCode) {
      return price;
    }
  }
  return null;
}

List<JsonObject> listModelsByCapability(ModelCatalog catalog, String capability) {
  return listModelsWhere(catalog, capability: capability);
}

List<JsonObject> listModelsByModality(ModelCatalog catalog, String inputModality, String outputModality) {
  return listModelsWhere(catalog, inputModality: inputModality, outputModality: outputModality);
}

List<JsonObject> listProtocols(ModelCatalog catalog) {
  return catalog.protocols;
}

JsonObject? findProtocol(ModelCatalog catalog, String protocolCode) {
  for (final p in catalog.protocols) {
    if (p['protocolCode'] == protocolCode) {
      return p;
    }
  }
  return null;
}

List<JsonObject> listProtocolsByVendor(ModelCatalog catalog, String vendorCode) {
  final vendor = catalog.vendors.cast<JsonObject?>().firstWhere(
        (v) => v?['vendorCode'] == vendorCode,
        orElse: () => null,
      );
  if (vendor == null) {
    return const [];
  }
  final supported = ((vendor['supportedProtocols'] as List?) ?? const [])
      .whereType<String>()
      .toSet();
  return catalog.protocols.where((p) => supported.contains(p['protocolCode'])).toList();
}

List<JsonObject> listModelsByProtocol(ModelCatalog catalog, String protocolCode) {
  return listModelsWhere(catalog, apiFormat: protocolCode);
}

String? _filterString(JsonObject? filter, String key) {
  final value = filter?[key];
  return value is String && value.isNotEmpty ? value : null;
}
