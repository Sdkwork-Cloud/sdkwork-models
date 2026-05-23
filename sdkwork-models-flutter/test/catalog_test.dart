import 'package:sdkwork_models/sdkwork_models.dart';

Future<void> main() async {
  final catalog = ModelCatalog(
    catalogVersion: '2026.05.08.1',
    schemaVersion: '1.1.0',
    meters: const [
      {'meterCode': 'llm_input_token', 'defaultUnitSize': '1000000'}
    ],
    protocols: const [
      {'protocolCode': 'openai_compatible', 'vendorOrigin': 'openai', 'displayName': 'OpenAI Chat Completions Compatible', 'family': 'openai', 'docsUrl': 'https://platform.openai.com/docs/api-reference/chat/create', 'maturity': 'stable'},
      {'protocolCode': 'openai_responses', 'vendorOrigin': 'openai', 'displayName': 'OpenAI Responses API', 'family': 'openai', 'docsUrl': 'https://platform.openai.com/docs/api-reference/responses', 'maturity': 'stable'},
      {'protocolCode': 'anthropic_messages', 'vendorOrigin': 'anthropic', 'displayName': 'Anthropic Messages API', 'family': 'anthropic', 'docsUrl': 'https://docs.anthropic.com/en/api/messages', 'maturity': 'stable'},
      {'protocolCode': 'google_gemini', 'vendorOrigin': 'google', 'displayName': 'Google Gemini API', 'family': 'google', 'docsUrl': 'https://ai.google.dev/gemini-api/docs', 'maturity': 'stable'},
    ],
    vendors: const [
      {'vendorCode': 'openai', 'displayName': 'OpenAI', 'supportedProtocols': ['openai_responses', 'openai_compatible']},
      {
        'vendorCode': 'minimax',
        'displayName': 'MiniMax',
        'supportedProtocols': ['openai_compatible'],
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
        'apiFormat': 'openai_responses',
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
  assert(listModels(catalog, filter: {'apiFormat': 'openai_compatible'}).isNotEmpty);
  assert(listModelsByProtocol(catalog, 'openai_compatible').isNotEmpty);
  assert(listModelsByProtocol(catalog, 'openai_responses').isNotEmpty);
  assert(getModelPrices(catalog, 'openai/global/gpt-5.5').isNotEmpty);
  assert(getBestReferencePrice(catalog, 'openai/global/gpt-5.5', 'llm_input_token')?['unitPrice'] == '5.000000');

  final protocols = listProtocols(catalog);
  assert(protocols.length >= 4);
  assert(findProtocol(catalog, 'openai_responses')?['displayName'] == 'OpenAI Responses API');
  assert(findProtocol(catalog, 'nonexistent') == null);
  final openaiProtocols = listProtocolsByVendor(catalog, 'openai');
  assert(openaiProtocols.length >= 2);
  assert(openaiProtocols.any((p) => p['protocolCode'] == 'openai_responses'));
  assert(openaiProtocols.any((p) => p['protocolCode'] == 'openai_compatible'));
  final vendor = listVendors(catalog).firstWhere((v) => v['vendorCode'] == 'openai');
  assert((vendor['supportedProtocols'] as List).contains('openai_responses'));

  final localCatalog = await loadCatalog('..');
  assert(findModel(localCatalog, 'openai/global/gpt-5.5')?['vendorCode'] == 'openai');
  assert(listProtocols(localCatalog).any((p) => p['protocolCode'] == 'openai_responses'));
  assert(listModelsByProtocol(localCatalog, 'openai_responses').isNotEmpty);

  final openaiGlobal = await loadVendorCatalog('..', 'openai', 'global');
  assert(openaiGlobal['vendorCode'] == 'openai');
  assert(openaiGlobal['regionCode'] == 'global');
  assert((openaiGlobal['models'] as List).isNotEmpty);
}
