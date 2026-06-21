import 'types.dart';

final _decimal = RegExp(r'^(0|[1-9][0-9]*)(\.[0-9]+)?$');

List<JsonObject> validateCatalog(ModelCatalog catalog) {
  final issues = <JsonObject>[];
  final modelKeys = catalog.models.map((model) => model['catalogKey']).toSet();
  for (final pricing in catalog.pricing) {
    if (!modelKeys.contains(pricing['catalogKey'])) {
      issues.add({'code': 'pricing.model.missing', 'catalogKey': pricing['catalogKey']});
    }
    for (final price in ((pricing['prices'] as List?) ?? const [])) {
      final item = price as JsonObject;
      for (final field in const ['unitSize', 'unitPrice', 'minimumQuantity']) {
        final value = item[field];
        if (value is! String || !_decimal.hasMatch(value)) {
          issues.add({'code': 'pricing.decimal.invalid', 'field': field});
        }
      }
    }
  }
  return issues;
}
