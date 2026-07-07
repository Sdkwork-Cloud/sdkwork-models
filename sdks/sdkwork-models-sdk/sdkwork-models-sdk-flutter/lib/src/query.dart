import 'capabilities.dart';
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
      'capabilities':
          ((vendor['capabilities'] as List?) ?? const []).cast<Object?>(),
      'supportedProtocols':
          ((vendor['supportedProtocols'] as List?) ?? const []).cast<Object?>(),
      'clientApiCompatibility':
          vendor['clientApiCompatibility'] is JsonObject
              ? vendor['clientApiCompatibility']
              : <String, Object?>{},
      'openSource': vendor['openSource'] ?? false,
    };
  }
  return vendors.values.toList();
}

List<JsonObject> listVendorRegions(ModelCatalog catalog) {
  if (catalog.vendorCatalogs.isNotEmpty) {
    return [
      for (final vendorCatalog in catalog.vendorCatalogs)
        if (vendorCatalog['vendorCode'] is String &&
            vendorCatalog['regionCode'] is String)
          {
            'vendorCode': vendorCatalog['vendorCode'],
            'regionCode': vendorCatalog['regionCode']
          }
    ];
  }
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
          regions.add(
              {'vendorCode': vendorCode, 'regionCode': region['regionCode']});
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
  final matches = _regionalModels(catalog)
      .map((model) => {
            'model': model,
            'hasRegionPricing': _hasRegionPricing(catalog, model),
          })
      .where((item) {
    final model = item['model'] as JsonObject;
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
  if (regionCode != null) {
    return matches.map((item) => item['model'] as JsonObject).toList();
  }
  final deduped = <String, JsonObject>{};
  for (final item in matches) {
    final model = item['model'] as JsonObject;
    final catalogKey = model['catalogKey'];
    if (catalogKey is! String) {
      continue;
    }
    final existing = deduped[catalogKey];
    if (existing == null ||
        _modelIdentityScore(item) > _modelIdentityScore(existing)) {
      deduped[catalogKey] = item;
    }
  }
  return deduped.values.map((item) => item['model'] as JsonObject).toList();
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
    final regionCode = model['regionCode'];
    return catalogKeyValue is String &&
        regionCode is String &&
        getModelRegionPrices(catalog, catalogKeyValue, regionCode).isNotEmpty;
  }).toList();
}

String catalogKey(String vendorCode, String modelId) =>
    '$vendorCode/$modelId';

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
  final parts = _splitCatalogKey(catalogKeyValue);
  if (parts == null) {
    return null;
  }
  for (final model in listModels(catalog)) {
    if (model['vendorCode'] == parts.vendorCode &&
        model['modelId'] == parts.modelId) {
      return model;
    }
  }
  return null;
}

JsonObject? findModelByVendorRegion(ModelCatalog catalog, String vendorCode,
    String regionCode, String modelId) {
  for (final model
      in listModels(catalog, vendorCode: vendorCode, regionCode: regionCode)) {
    if (model['vendorCode'] == vendorCode &&
        model['regionCode'] == regionCode &&
        model['modelId'] == modelId) {
      return model;
    }
  }
  return null;
}

List<JsonObject> getModelPrices(ModelCatalog catalog, String catalogKeyValue) {
  final parts = _splitCatalogKey(catalogKeyValue);
  if (parts == null) {
    return const [];
  }
  final vendorCode = parts.vendorCode;
  final modelId = parts.modelId;
  for (final item in catalog.pricing) {
    if (item['vendorCode'] == vendorCode && item['modelId'] == modelId) {
      return ((item['prices'] as List?) ?? const []).cast<JsonObject>();
    }
  }
  return const [];
}

List<JsonObject> getModelRegionPrices(
    ModelCatalog catalog, String catalogKeyValue, String regionCode) {
  final parts = _splitCatalogKey(catalogKeyValue);
  if (parts == null) {
    return const [];
  }
  final vendorCode = parts.vendorCode;
  final modelId = parts.modelId;
  for (final vendorCatalog in catalog.vendorCatalogs) {
    if (vendorCatalog['vendorCode'] != vendorCode ||
        vendorCatalog['regionCode'] != regionCode) {
      continue;
    }
    for (final item in _objectList(vendorCatalog['pricing'])) {
      if (item['modelId'] == modelId) {
        return _objectList(item['prices']);
      }
    }
    return const [];
  }
  for (final item in catalog.pricing) {
    if (item['vendorCode'] == vendorCode &&
        item['regionCode'] == regionCode &&
        item['modelId'] == modelId) {
      return _objectList(item['prices']);
    }
  }
  return const [];
}

JsonObject? getBestReferencePrice(
    ModelCatalog catalog, String catalogKeyValue, String meterCode) {
  for (final price in getModelPrices(catalog, catalogKeyValue)) {
    if (price['meterCode'] == meterCode) {
      return price;
    }
  }
  return null;
}

List<JsonObject> listModelsByCapability(
    ModelCatalog catalog, String capability) {
  return listModelsWhere(catalog, capability: capability);
}

List<JsonObject> listModelsByModality(
    ModelCatalog catalog, String inputModality, String outputModality) {
  return listModelsWhere(catalog,
      inputModality: inputModality, outputModality: outputModality);
}

List<JsonObject> listModelsWithFeature(ModelCatalog catalog, String feature) {
  return listModels(catalog)
      .where((model) => modelSupportsFeature(model, feature))
      .toList();
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

List<JsonObject> listProtocolsByVendor(
    ModelCatalog catalog, String vendorCode) {
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
  return catalog.protocols
      .where((p) => supported.contains(p['protocolCode']))
      .toList();
}

List<JsonObject> listClientApiCompatibilityByVendor(
    ModelCatalog catalog, String vendorCode) {
  final vendor = catalog.vendors.cast<JsonObject?>().firstWhere(
        (v) => v?['vendorCode'] == vendorCode,
        orElse: () => null,
      );
  if (vendor == null || vendor['clientApiCompatibility'] is! JsonObject) {
    return const [];
  }
  return (vendor['clientApiCompatibility'] as JsonObject)
      .values
      .whereType<Map>()
      .map((item) => item.map((key, value) => MapEntry(key.toString(), value)))
      .toList();
}

List<JsonObject> listModelsByProtocol(
    ModelCatalog catalog, String protocolCode) {
  return listModelsWhere(catalog, apiFormat: protocolCode);
}

_CatalogKeyParts? _splitCatalogKey(String catalogKeyValue) {
  final separatorIndex = catalogKeyValue.indexOf('/');
  if (separatorIndex <= 0 || separatorIndex == catalogKeyValue.length - 1) {
    return null;
  }
  return _CatalogKeyParts(
    catalogKeyValue.substring(0, separatorIndex),
    catalogKeyValue.substring(separatorIndex + 1),
  );
}

String? _filterString(JsonObject? filter, String key) {
  final value = filter?[key];
  return value is String && value.isNotEmpty ? value : null;
}

List<JsonObject> _regionalModels(ModelCatalog catalog) {
  if (catalog.vendorCatalogs.isNotEmpty) {
    return [
      for (final vendorCatalog in catalog.vendorCatalogs)
        ..._objectList(vendorCatalog['models'])
    ];
  }
  return catalog.models;
}

bool _hasRegionPricing(ModelCatalog catalog, JsonObject model) {
  final catalogKey = model['catalogKey'];
  final regionCode = model['regionCode'];
  return catalogKey is String &&
      regionCode is String &&
      getModelRegionPrices(catalog, catalogKey, regionCode).isNotEmpty;
}

int _modelIdentityScore(JsonObject item) {
  final model = item['model'] as JsonObject;
  var score = 0;
  if (item['hasRegionPricing'] == true) {
    score += 100;
  }
  if (model['routingState'] == 'enabled') {
    score += 40;
  }
  if (model['shelfState'] == 'listed') {
    score += 20;
  }
  if (model['releaseStage'] == 'active') {
    score += 10;
  }
  if (model['lifecycle'] == 'current' || model['lifecycle'] == 'preview') {
    score += 5;
  }
  if (model['regionCode'] == 'global') {
    score += 1;
  }
  return score;
}

String voiceCatalogKey(String vendorCode, String voiceId) => '$vendorCode/$voiceId';

List<JsonObject> listVoices(
  ModelCatalog catalog, {
  String? vendorCode,
  String? regionCode,
  String? locale,
  String? modelCatalogKey,
  String? q,
}) {
  return _regionalVoices(catalog)
      .where((voice) => vendorCode == null || voice['vendorCode'] == vendorCode)
      .where((voice) => regionCode == null || voice['regionCode'] == regionCode)
      .where((voice) {
        if (locale == null) {
          return true;
        }
        return voice['primaryLocale'] == locale ||
            _objectList(voice['supportedLocales']).any((entry) => entry == locale);
      })
      .where((voice) {
        if (q == null) {
          return true;
        }
        final query = q.toLowerCase();
        final displayName = voice['displayName'];
        final voiceId = voice['voiceId'];
        return (displayName is String && displayName.toLowerCase().contains(query)) ||
            (voiceId is String && voiceId.toLowerCase().contains(query));
      })
      .where((voice) {
        if (modelCatalogKey == null) {
          return true;
        }
        final voiceKey = voice['catalogKey'];
        if (voiceKey is! String) {
          return false;
        }
        for (final vendorCatalog in catalog.vendorCatalogs) {
          for (final binding in _objectList(vendorCatalog['modelVoiceBindings'])) {
            if (binding['catalogKey'] != modelCatalogKey) {
              continue;
            }
            for (final entry in _objectList(binding['bindings'])) {
              if (entry['voiceKey'] == voiceKey) {
                return true;
              }
            }
          }
        }
        return false;
      })
      .toList();
}

List<JsonObject> listVoicesForModel(ModelCatalog catalog, String modelCatalogKey) {
  return listVoices(catalog, modelCatalogKey: modelCatalogKey);
}

List<JsonObject> listModelsForVoice(ModelCatalog catalog, String voiceCatalogKey) {
  final modelKeys = <String>{};
  for (final vendorCatalog in catalog.vendorCatalogs) {
    for (final binding in _objectList(vendorCatalog['modelVoiceBindings'])) {
      final catalogKey = binding['catalogKey'];
      if (catalogKey is! String) {
        continue;
      }
      for (final entry in _objectList(binding['bindings'])) {
        if (entry['voiceKey'] == voiceCatalogKey) {
          modelKeys.add(catalogKey);
        }
      }
    }
  }
  return listModels(catalog).where((model) {
    final catalogKey = model['catalogKey'];
    return catalogKey is String && modelKeys.contains(catalogKey);
  }).toList();
}

String videoProfileCatalogKey(
    String vendorCode, String modelId, String profileCode) {
  return '$vendorCode/$modelId/$profileCode';
}

List<JsonObject> listVideoProfiles(
  ModelCatalog catalog, {
  String? vendorCode,
  String? regionCode,
  String? modelCatalogKey,
  String? generationMode,
  String? durationTierCode,
  String? resolution,
}) {
  final result = <JsonObject>[];
  for (final vendorCatalog in catalog.vendorCatalogs) {
    if (vendorCode != null && vendorCatalog['vendorCode'] != vendorCode) {
      continue;
    }
    if (regionCode != null && vendorCatalog['regionCode'] != regionCode) {
      continue;
    }
    for (final profileFile in _objectList(vendorCatalog['modelVideoProfiles'])) {
      if (modelCatalogKey != null && profileFile['catalogKey'] != modelCatalogKey) {
        continue;
      }
      for (final profile in _objectList(profileFile['profiles'])) {
        if (generationMode != null && profile['generationMode'] != generationMode) {
          continue;
        }
        if (durationTierCode != null &&
            profile['durationTierCode'] != durationTierCode &&
            !_objectList(profile['durationTierCodes'])
                .any((entry) => entry == durationTierCode)) {
          continue;
        }
        if (resolution != null && profile['resolution'] != resolution) {
          continue;
        }
        result.add(profile);
      }
    }
  }
  return result;
}

List<JsonObject> listVideoProfilesForModel(
    ModelCatalog catalog, String modelCatalogKey) {
  return listVideoProfiles(catalog, modelCatalogKey: modelCatalogKey);
}

JsonObject? findVideoProfile(ModelCatalog catalog, String profileCatalogKey) {
  for (final profile in listVideoProfiles(catalog)) {
    if (profile['catalogKey'] == profileCatalogKey) {
      return profile;
    }
  }
  return null;
}

List<JsonObject> _regionalVoices(ModelCatalog catalog) {
  if (catalog.vendorCatalogs.isEmpty) {
    return const [];
  }
  return [
    for (final vendorCatalog in catalog.vendorCatalogs)
      ..._objectList(vendorCatalog['voices'])
  ];
}

List<JsonObject> _objectList(Object? value) {
  if (value is! List) {
    return const [];
  }
  return value
      .whereType<Map>()
      .map((item) => item.map((key, value) => MapEntry(key.toString(), value)))
      .toList();
}

class _CatalogKeyParts {
  const _CatalogKeyParts(this.vendorCode, this.modelId);

  final String vendorCode;
  final String modelId;
}
