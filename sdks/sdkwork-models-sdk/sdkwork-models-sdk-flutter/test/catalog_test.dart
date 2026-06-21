import 'package:sdkwork_models/sdkwork_models.dart';

Future<void> main() async {
  final catalog = ModelCatalog(
    catalogVersion: '2026.05.08.1',
    schemaVersion: '1.1.0',
    meters: const [
      {'meterCode': 'llm_input_token', 'defaultUnitSize': '1000000'}
    ],
    protocols: const [
      {
        'protocolCode': 'openai_compatible',
        'vendorOrigin': 'openai',
        'displayName': 'OpenAI Chat Completions Compatible',
        'family': 'openai',
        'docsUrl': 'https://platform.openai.com/docs/api-reference/chat/create',
        'maturity': 'stable'
      },
      {
        'protocolCode': 'openai_responses',
        'vendorOrigin': 'openai',
        'displayName': 'OpenAI Responses API',
        'family': 'openai',
        'docsUrl': 'https://platform.openai.com/docs/api-reference/responses',
        'maturity': 'stable'
      },
      {
        'protocolCode': 'anthropic_messages',
        'vendorOrigin': 'anthropic',
        'displayName': 'Anthropic Messages API',
        'family': 'anthropic',
        'docsUrl': 'https://docs.anthropic.com/en/api/messages',
        'maturity': 'stable'
      },
      {
        'protocolCode': 'google_gemini',
        'vendorOrigin': 'google',
        'displayName': 'Google Gemini API',
        'family': 'google',
        'docsUrl': 'https://ai.google.dev/gemini-api/docs',
        'maturity': 'stable'
      },
    ],
    vendors: const [
      {
        'vendorCode': 'openai',
        'displayName': 'OpenAI',
        'supportedProtocols': ['openai_responses', 'openai_compatible'],
        'clientApiCompatibility': {
          'codex': {'clientApiCode': 'codex', 'supportStatus': 'supported'}
        }
      },
      {
        'vendorCode': 'minimax',
        'displayName': 'MiniMax',
        'supportedProtocols': ['openai_compatible'],
        'regions': [
          {'regionCode': 'cn'},
          {'regionCode': 'global'}
        ]
      },
      {
        'vendorCode': 'openrouter',
        'displayName': 'OpenRouter',
        'supportedProtocols': ['openai_compatible'],
        'regions': [
          {'regionCode': 'global'}
        ]
      }
    ],
    vendorCatalogs: const [],
    models: const [
      {
        'catalogKey': 'openai/gpt-5.5',
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
        'catalogKey': 'minimax/MiniMax-M2.7',
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
      },
      {
        'catalogKey': 'openrouter/anthropic/claude-3-opus',
        'modelId': 'anthropic/claude-3-opus',
        'displayName': 'Claude 3 Opus through OpenRouter',
        'vendorCode': 'openrouter',
        'regionCode': 'global',
        'familyCode': 'anthropic',
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
        'catalogKey': 'openai/gpt-5.5',
        'vendorCode': 'openai',
        'regionCode': 'global',
        'modelId': 'gpt-5.5',
        'prices': [
          {'meterCode': 'llm_input_token', 'unitPrice': '5.000000'}
        ]
      },
      {
        'catalogKey': 'openrouter/anthropic/claude-3-opus',
        'vendorCode': 'openrouter',
        'regionCode': 'global',
        'modelId': 'anthropic/claude-3-opus',
        'prices': [
          {'meterCode': 'llm_input_token', 'unitPrice': '15.000000'}
        ]
      }
    ],
  );
  assert(findModel(catalog, 'openai/gpt-5.5')?['vendorCode'] == 'openai');
  assert(findModel(catalog, 'openai/gpt-5.5')?['regionCode'] == 'global');
  assert(findModel(catalog, 'openai/global/gpt-5.5') == null);
  assert(findModelByVendorRegion(
          catalog, 'openai', 'global', 'gpt-5.5')?['vendorCode'] ==
      'openai');
  assert(catalogKey('openai', 'gpt-5.5') == 'openai/gpt-5.5');
  assert(listVendors(catalog)
      .every((vendor) => !vendor.containsKey('regionCode')));
  assert(listVendorRegions(catalog).any(
      (item) => item['vendorCode'] == 'minimax' && item['regionCode'] == 'cn'));
  assert(listVendors(catalog)
          .where((vendor) => vendor['vendorCode'] == 'minimax')
          .length ==
      1);
  assert(listMeters(catalog)
      .any((meter) => meter['meterCode'] == 'llm_input_token'));
  assert(
      findMeter(catalog, 'llm_input_token')?['defaultUnitSize'] == '1000000');
  assert(findMeter(catalog, 'missing_meter') == null);
  assert(listModels(catalog, filter: {
    'vendorCode': 'openai',
    'regionCode': 'global',
    'familyCode': 'gpt-5'
  }).isNotEmpty);
  assert(listModels(catalog,
          vendorCode: 'openai', regionCode: 'global', familyCode: 'gpt-5')
      .isNotEmpty);
  assert(listModels(catalog, filter: {
    'releaseStage': 'active',
    'shelfState': 'listed',
    'routingState': 'enabled'
  }).isNotEmpty);
  assert(listModels(catalog, filter: {'apiFormat': 'openai_compatible'})
      .isNotEmpty);
  assert(listModelsByProtocol(catalog, 'openai_compatible').isNotEmpty);
  assert(listModelsByProtocol(catalog, 'openai_responses').isNotEmpty);
  assert(getModelPrices(catalog, 'openai/gpt-5.5').isNotEmpty);
  assert(getModelPrices(catalog, 'openai/global/gpt-5.5').isEmpty);
  assert(getBestReferencePrice(
          catalog, 'openai/gpt-5.5', 'llm_input_token')?['unitPrice'] ==
      '5.000000');
  assert(catalogKey('openrouter', 'anthropic/claude-3-opus') ==
      'openrouter/anthropic/claude-3-opus');
  assert(findModel(catalog, 'openrouter/anthropic/claude-3-opus')?['modelId'] ==
      'anthropic/claude-3-opus');
  assert(getModelPrices(catalog, 'openrouter/anthropic/claude-3-opus')
      .isNotEmpty);
  assert(getModelRegionPrices(
          catalog, 'openrouter/anthropic/claude-3-opus', 'global')
      .isNotEmpty);
  assert(findModel(catalog, 'openrouter/global/anthropic/claude-3-opus') ==
      null);
  assert(getModelPrices(catalog, 'openrouter/global/anthropic/claude-3-opus')
      .isEmpty);

  final protocols = listProtocols(catalog);
  assert(protocols.length >= 4);
  assert(findProtocol(catalog, 'openai_responses')?['displayName'] ==
      'OpenAI Responses API');
  assert(findProtocol(catalog, 'nonexistent') == null);
  final openaiProtocols = listProtocolsByVendor(catalog, 'openai');
  assert(openaiProtocols.length >= 2);
  assert(openaiProtocols.any((p) => p['protocolCode'] == 'openai_responses'));
  assert(openaiProtocols.any((p) => p['protocolCode'] == 'openai_compatible'));
  final vendor =
      listVendors(catalog).firstWhere((v) => v['vendorCode'] == 'openai');
  assert((vendor['supportedProtocols'] as List).contains('openai_responses'));
  assert(listClientApiCompatibilityByVendor(catalog, 'openai').any((item) =>
      item['clientApiCode'] == 'codex' &&
      item['supportStatus'] == 'supported'));

  final localCatalog = await loadCatalog('..');
  assert(findModel(localCatalog, 'openai/gpt-5.5')?['vendorCode'] == 'openai');
  assert(
      findModel(localCatalog, 'kuaishou/kling-v3-0-preview')?['regionCode'] ==
          'global');
  assert(findModelByVendorRegion(localCatalog, 'kuaishou', 'cn',
          'kling-v3-0-preview')?['regionCode'] ==
      'cn');
  final modelKeys =
      listModels(localCatalog).map((model) => model['catalogKey']).toList();
  assert(modelKeys.toSet().length == modelKeys.length);
  assert(listAvailableModels(localCatalog).any((model) =>
      model['catalogKey'] == 'kuaishou/kling-v3-0-preview' &&
      model['regionCode'] == 'global'));
  assert(!listAvailableModels(localCatalog, regionCode: 'cn')
      .any((model) => model['catalogKey'] == 'kuaishou/kling-v3-0-preview'));
  assert(listAvailableModels(localCatalog, regionCode: 'global')
      .any((model) => model['catalogKey'] == 'kuaishou/kling-v3-0-preview'));
  assert(getModelRegionPrices(localCatalog, 'openai/gpt-5.5', 'global')
      .isNotEmpty);
  assert(getModelRegionPrices(localCatalog, 'openai/gpt-5.5', 'cn').isEmpty);
  assert(listProtocols(localCatalog)
      .any((p) => p['protocolCode'] == 'openai_responses'));
  assert(listModelsByProtocol(localCatalog, 'openai_responses').isNotEmpty);

  final openaiGlobal = await loadVendorCatalog('..', 'openai', 'global');
  assert(openaiGlobal['vendorCode'] == 'openai');
  assert(openaiGlobal['regionCode'] == 'global');
  assert((openaiGlobal['models'] as List).isNotEmpty);
}
