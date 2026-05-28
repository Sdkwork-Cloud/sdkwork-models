import 'dart:convert';
import 'dart:io';

import 'types.dart';

Future<ModelCatalog> loadCatalog(String pathOrUrl) async {
  final manifest = await _readJsonObject(pathOrUrl, 'sdkwork-models.json');
  final meterFile = await _readJsonObject(pathOrUrl, 'models/meters.json');
  final protocolFile =
      await _readJsonObject(pathOrUrl, 'models/protocols.json');
  final index = await _readJsonObject(pathOrUrl, 'models/index.json');
  final vendors = <JsonObject>[];
  final seenVendorCodes = <String>{};
  final modelScores = <String, int>{};
  final vendorCatalogs = <JsonObject>[];
  final models = <String, JsonObject>{};
  final pricing = <JsonObject>[];
  for (final vendorIndex in _objectList(index['vendors'])) {
    final vendorCode = vendorIndex['vendorCode'];
    final regionCode = vendorIndex['regionCode'];
    if (vendorCode is! String) {
      continue;
    }
    if (regionCode is! String) {
      continue;
    }
    final vendorCatalog =
        await loadVendorCatalog(pathOrUrl, vendorCode, regionCode);
    vendorCatalogs.add(vendorCatalog);
    final vendor = vendorCatalog['vendor'];
    if (seenVendorCodes.add(vendorCode) && vendor is JsonObject) {
      vendors.add(vendor);
    }
    for (final model in _objectList(vendorCatalog['models'])) {
      _putBestModelIdentity(models, modelScores, vendorCatalog, model);
    }
    pricing.addAll(_objectList(vendorCatalog['pricing']));
  }
  return ModelCatalog(
    catalogVersion:
        _requiredString(manifest['catalogVersion'], 'catalogVersion'),
    schemaVersion: _requiredString(manifest['schemaVersion'], 'schemaVersion'),
    meters: _objectList(meterFile['meters']),
    protocols: _objectList(protocolFile['protocols']),
    vendors: vendors,
    vendorCatalogs: vendorCatalogs,
    models: models.values.toList(),
    pricing: pricing,
  );
}

Future<ModelCatalog> loadBundledCatalog() async {
  const envRoot = String.fromEnvironment('SDKWORK_MODELS_CATALOG_ROOT');
  if (envRoot.isNotEmpty) {
    return loadCatalog(envRoot);
  }
  final processRoot = Platform.environment['SDKWORK_MODELS_CATALOG_ROOT'];
  if (processRoot != null && processRoot.trim().isNotEmpty) {
    return loadCatalog(processRoot);
  }
  return loadCatalog('data/sdkwork-models');
}

Future<Map<String, Object?>> loadVendorCatalog(
    String pathOrUrl, String vendorCode, String regionCode) async {
  final index = await _readJsonObject(pathOrUrl, 'models/index.json');
  final vendorIndex =
      _objectList(index['vendors']).cast<JsonObject?>().firstWhere(
            (item) =>
                item?['vendorCode'] == vendorCode &&
                item?['regionCode'] == regionCode,
            orElse: () => null,
          );
  if (vendorIndex == null) {
    throw StateError('vendor region $vendorCode/$regionCode is not indexed');
  }
  return {
    'vendorCode': vendorCode,
    'regionCode': regionCode,
    'vendor': await _readJsonObject(pathOrUrl, 'models/${vendorIndex['path']}'),
    'families': await _readJsonObject(
        pathOrUrl, 'models/${vendorIndex['familiesPath']}'),
    'models': await _readJsonObjectsByRef(
        pathOrUrl, _stringList(vendorIndex['modelFiles'])),
    'pricing': await _readJsonObjectsByRef(
        pathOrUrl, _stringList(vendorIndex['pricingFiles'])),
  };
}

bool _isRemoteUrl(String value) {
  final uri = Uri.tryParse(value);
  return uri != null && (uri.scheme == 'http' || uri.scheme == 'https');
}

Future<JsonObject> _readJsonObject(String root, String relPath) async {
  final decoded = jsonDecode(await _readText(root, relPath));
  if (decoded is Map<String, Object?>) {
    return decoded;
  }
  if (decoded is Map) {
    return decoded.map((key, value) => MapEntry(key.toString(), value));
  }
  throw FormatException('JSON root must be an object: $relPath');
}

Future<String> _readText(String root, String relPath) async {
  if (_isRemoteUrl(root)) {
    final response = await HttpClient()
        .getUrl(Uri.parse('${root.replaceFirst(RegExp(r'/+$'), '')}/$relPath'))
        .then((request) => request.close());
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw HttpException(
          'failed to fetch sdkwork-models catalog file $relPath: ${response.statusCode}');
    }
    return utf8.decode(await response.expand((chunk) => chunk).toList());
  }
  return File('${Directory(root).path}/$relPath').readAsString();
}

Future<List<JsonObject>> _readJsonObjectsByRef(
    String root, List<String> refs) async {
  final result = <JsonObject>[];
  for (final ref in refs) {
    result.add(await _readJsonObject(root, 'models/$ref'));
  }
  return result;
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

List<String> _stringList(Object? value) {
  if (value is! List) {
    return const [];
  }
  return value.whereType<String>().toList();
}

String _requiredString(Object? value, String field) {
  if (value is String && value.isNotEmpty) {
    return value;
  }
  throw FormatException('$field must be a non-empty string');
}

void _putBestModelIdentity(
  Map<String, JsonObject> models,
  Map<String, int> scores,
  JsonObject vendorCatalog,
  JsonObject model,
) {
  final catalogKey = model['catalogKey'];
  if (catalogKey is! String || catalogKey.isEmpty) {
    return;
  }
  final score = _modelIdentityScore(vendorCatalog, model);
  final existingScore = scores[catalogKey];
  if (existingScore == null || score > existingScore) {
    models[catalogKey] = model;
    scores[catalogKey] = score;
  }
}

int _modelIdentityScore(JsonObject vendorCatalog, JsonObject model) {
  var score = 0;
  if (_hasRegionPricing(vendorCatalog, model)) {
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

bool _hasRegionPricing(JsonObject vendorCatalog, JsonObject model) {
  final modelId = model['modelId'];
  if (modelId is! String) {
    return false;
  }
  return _objectList(vendorCatalog['pricing']).any((item) {
    return item['modelId'] == modelId && _objectList(item['prices']).isNotEmpty;
  });
}
