import 'package:sdkwork_models/sdkwork_models.dart';

Future<void> main() async {
  final catalog = ModelCatalog(
    catalogVersion: '2026.05.08.1',
    schemaVersion: '1.1.0',
    meters: const [
      {'meterCode': 'llm_input_token', 'defaultUnitSize': '1000000'}
    ],
    vendors: const [
      {'vendorCode': 'openai', 'displayName': 'OpenAI'},
      {
        'vendorCode': 'minimax',
        'displayName': 'MiniMax',
        'regions': [
          {'regionCode': 'cn'},
          {'regionCode': 'global'}
        ]
      }
    ],
    models: const [
      {
        'catalogKey': 'openai/global/gpt-5.5',
        'modelId': 'gpt-5.5',
        'vendorCode': 'openai',
        'regionCode': 'global',
        'familyCode': 'gpt-5',
        'releaseStage': 'active',
        'shelfState': 'listed',
        'routingState': 'enabled',
        'apiFormat': 'openai_compatible',
        'capabilities': ['chat'],
        'inputModalities': ['text'],
        'outputModalities': ['text']
      },
      {
        'catalogKey': 'minimax/cn/MiniMax-M2.7',
        'modelId': 'MiniMax-M2.7',
        'vendorCode': 'minimax',
        'regionCode': 'cn',
        'familyCode': 'MiniMax-M2',
        'releaseStage': 'active',
        'shelfState': 'listed',
        'routingState': 'enabled',
        'apiFormat': 'openai_compatible',
        'capabilities': ['chat'],
        'inputModalities': ['text'],
        'outputModalities': ['text']
      }
    ],
    pricing: const [
      {
        'catalogKey': 'openai/global/gpt-5.5',
        'vendorCode': 'openai',
        'regionCode': 'global',
        'modelId': 'gpt-5.5',
        'prices': [
          {'meterCode': 'llm_input_token', 'unitPrice': '5.000000'}
        ]
      }
    ],
  );
  assert(findModel(catalog, 'openai/global/gpt-5.5')?['vendorCode'] == 'openai');
  assert(findModel(catalog, 'openai/global/gpt-5.5')?['regionCode'] == 'global');
  assert(findModel(catalog, 'openai/gpt-5.5') == null);
  assert(findModelByVendorRegion(catalog, 'openai', 'global', 'gpt-5.5')?['vendorCode'] == 'openai');
  assert(catalogKey('openai', 'global', 'gpt-5.5') == 'openai/global/gpt-5.5');
  assert(listVendors(catalog).every((vendor) => !vendor.containsKey('regionCode')));
  assert(listVendorRegions(catalog).any((item) => item['vendorCode'] == 'minimax' && item['regionCode'] == 'cn'));
  assert(listVendors(catalog).where((vendor) => vendor['vendorCode'] == 'minimax').length == 1);
  assert(listMeters(catalog).any((meter) => meter['meterCode'] == 'llm_input_token'));
  assert(findMeter(catalog, 'llm_input_token')?['defaultUnitSize'] == '1000000');
  assert(findMeter(catalog, 'missing_meter') == null);
  assert(listModels(catalog, filter: {'vendorCode': 'openai', 'regionCode': 'global', 'familyCode': 'gpt-5'}).isNotEmpty);
  assert(listModels(catalog, vendorCode: 'openai', regionCode: 'global', familyCode: 'gpt-5').isNotEmpty);
  assert(listModels(catalog, filter: {'releaseStage': 'active', 'shelfState': 'listed', 'routingState': 'enabled'}).isNotEmpty);
  assert(listModels(catalog, releaseStage: 'active', shelfState: 'listed', routingState: 'enabled').isNotEmpty);
  assert(listModels(catalog, filter: {'apiFormat': 'openai_compatible'}).isNotEmpty);
  assert(listModels(catalog, apiFormat: 'openai_compatible').isNotEmpty);
  assert(listModelsByCapability(catalog, 'chat').isNotEmpty);
  assert(listModelsByModality(catalog, 'text', 'text').isNotEmpty);
  final availableModels = listAvailableModels(catalog);
  assert(availableModels.isNotEmpty);
  assert(availableModels.every((model) => getModelPrices(catalog, model['catalogKey'] as String).isNotEmpty));
  assert(availableModels.every((model) => model['routingState'] == 'enabled' && model['shelfState'] == 'listed'));
  assert(availableModels.every((model) => model['catalogKey'] != 'kuaishou/cn/kling-v3-0-preview'));
  assert(getModelPrices(catalog, 'openai/global/gpt-5.5').isNotEmpty);
  assert(getBestReferencePrice(catalog, 'openai/global/gpt-5.5', 'llm_input_token')?['unitPrice'] == '5.000000');

  final localCatalog = await loadCatalog('..');
  assert(findModel(localCatalog, 'openai/global/gpt-5.5')?['vendorCode'] == 'openai');
  assert(listModels(localCatalog, filter: {'vendorCode': 'minimax', 'regionCode': 'cn', 'familyCode': 'minimax'}).isNotEmpty);

  final openaiGlobal = await loadVendorCatalog('..', 'openai', 'global');
  assert(openaiGlobal['vendorCode'] == 'openai');
  assert(openaiGlobal['regionCode'] == 'global');
  assert((openaiGlobal['models'] as List).isNotEmpty);

  final bundledCatalog = await loadBundledCatalog();
  assert(findModel(bundledCatalog, 'openai/global/gpt-5.5')?['vendorCode'] == 'openai');
}
