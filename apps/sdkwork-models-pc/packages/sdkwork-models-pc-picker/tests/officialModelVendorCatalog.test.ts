import assert from 'node:assert/strict';
import test from 'node:test';
import {
  applyOfficialModelVendorCatalogEntry,
  resolveOfficialModelVendorCatalog,
  resolveOfficialModelVendorPresets,
} from '../src/agent-model-access-selector/officialModelVendorCatalog.ts';
import { SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS } from '../src/agent-model-access-selector/officialModelVendorPresets.generated.ts';
import { createEmptyModelAccessChannelConfigurationDraft } from '../src/agent-model-access-selector/modelAccessChannelConfigurationValidation.ts';
import { GENERATED_MAINSTREAM_AGENT_MODEL_FALLBACK } from '../src/agent-model-access-selector/generatedAgentModelFallback.ts';
import { SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG } from '../src/unified-agent-model-selector/mainstreamAgentModelCatalog.generated.ts';

const providers = [
  { id: 'codex', label: 'Codex' },
  { id: 'claude-code', label: 'Claude Code' },
];

const models = [
  {
    id: 'openai/gpt-mini',
    label: 'GPT Mini',
    modelId: 'gpt-mini',
    sortOrder: 2,
    vendorCode: 'openai',
    vendorName: 'OpenAI',
  },
  {
    id: 'openai/gpt-latest',
    label: 'GPT Latest',
    modelId: 'gpt-latest',
    sortOrder: 1,
    vendorCode: 'openai',
    vendorName: 'OpenAI',
  },
  {
    id: 'alibaba/qwen-latest',
    label: 'Qwen Latest',
    modelId: 'qwen-latest',
    sortOrder: 0,
    vendorCode: 'alibaba',
    vendorName: 'Alibaba Cloud',
  },
];

test('generated official presets cover every catalog vendor with chat models', () => {
  assert.deepEqual(
    SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS.map((preset) => preset.providerCode),
    [
      'openai_direct',
      'anthropic_direct',
      'google_ai_direct',
      'alibaba.direct',
      'baidu.direct',
      'bytedance.direct',
      'deepseek.direct',
      'meituan.direct',
      'minimax.direct',
      'moonshot.direct',
      'stepfun.direct',
      'tencent.direct',
      'xai.direct',
      'xiaomi.direct',
      'zhipu.direct',
    ],
  );
  assert.deepEqual(
    SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS.map((preset) => preset.vendorCode),
    [
      'openai',
      'anthropic',
      'google',
      'alibaba',
      'baidu',
      'bytedance',
      'deepseek',
      'meituan',
      'minimax',
      'moonshot',
      'stepfun',
      'tencent',
      'xai',
      'xiaomi',
      'zhipu',
    ],
  );
  // Presets carry the vendor chat models so channels stay usable even when the
  // runtime fallback catalog does not include that vendor.
  const baidu = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS.find((preset) => (
    preset.vendorCode === 'baidu'
  ));
  assert.ok(baidu);
  assert.equal(baidu.baseUrl, '');
  assert.ok((baidu.models?.length ?? 0) > 0);
  // Vendors without a direct CloudRouter provider keep an empty Base URL so the
  // user supplies the official endpoint during configuration.
  const deepSeek = SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS.find((preset) => (
    preset.vendorCode === 'deepseek'
  ));
  assert.ok(deepSeek);
  assert.equal(deepSeek.baseUrl, '');
});

test('official vendor selection fills immutable provider facts and every catalog model', () => {
  const openAiPreset = {
    baseUrl: 'https://api.openai.com/v1',
    channelName: 'OpenAI',
    models: [
      { catalogKey: 'openai/gpt-latest', model: 'gpt-latest', displayName: 'GPT Latest' },
      { catalogKey: 'openai/gpt-mini', model: 'gpt-mini', displayName: 'GPT Mini' },
    ],
    protocol: 'openai_compatible',
    providerCode: 'openai_direct',
    providerDisplayName: 'OpenAI direct',
    sortOrder: 0,
    vendorCode: 'openai',
    vendorName: 'OpenAI',
  };
  const catalog = resolveOfficialModelVendorCatalog(models, [openAiPreset]);
  const openAi = catalog.find((entry) => entry.vendorCode === 'openai');
  assert.ok(openAi);
  const draft = applyOfficialModelVendorCatalogEntry(
    createEmptyModelAccessChannelConfigurationDraft(providers),
    openAi,
  );

  assert.equal(draft.channelId, 'official.openai');
  assert.equal(draft.name, 'OpenAI');
  assert.equal(draft.baseUrl, 'https://api.openai.com/v1');
  assert.equal(draft.defaultVendorCode, 'openai');
  assert.equal(draft.defaultModelId, 'gpt-latest');
  assert.deepEqual(draft.offerings[0]?.models, [
    { modelId: 'gpt-latest', displayName: 'GPT Latest' },
    { modelId: 'gpt-mini', displayName: 'GPT Mini' },
  ]);
  assert.deepEqual(draft.offerings[0]?.modelIds, ['gpt-latest', 'gpt-mini']);
  assert.deepEqual(draft.supportedAgentProviderIds, ['codex', 'claude-code']);
});

test('runtime official presets are authoritative and preserve their model order', () => {
  const runtimePresets = [{
    ...SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS[0],
    channelName: 'OpenAI Cloud',
    baseUrl: 'https://gateway.openai.example/v1',
    defaultModelId: 'gpt-mini',
    models: [
      { catalogKey: 'openai/gpt-mini', model: 'gpt-mini', displayName: 'GPT Mini (runtime)' },
      { catalogKey: 'openai/gpt-latest', model: 'gpt-latest', displayName: 'GPT Latest (runtime)' },
    ],
  }];
  const catalog = resolveOfficialModelVendorCatalog(models, runtimePresets);
  assert.deepEqual(catalog.map((entry) => entry.vendorCode), ['openai']);
  assert.equal(catalog[0]?.channelName, 'OpenAI Cloud');
  assert.equal(catalog[0]?.baseUrl, 'https://gateway.openai.example/v1');
  assert.equal(catalog[0]?.defaultModelId, 'gpt-mini');
  assert.deepEqual(catalog[0]?.models.map((model) => model.modelId), [
    'gpt-mini',
    'gpt-latest',
  ]);
  assert.deepEqual(catalog[0]?.models.map((model) => model.label), [
    'GPT Mini (runtime)',
    'GPT Latest (runtime)',
  ]);
});

test('empty runtime official presets fall back to generated catalog vendors', () => {
  assert.deepEqual(
    resolveOfficialModelVendorPresets([]).map((preset) => preset.providerCode),
    [
      'openai_direct',
      'anthropic_direct',
      'google_ai_direct',
      'alibaba.direct',
      'baidu.direct',
      'bytedance.direct',
      'deepseek.direct',
      'meituan.direct',
      'minimax.direct',
      'moonshot.direct',
      'stepfun.direct',
      'tencent.direct',
      'xai.direct',
      'xiaomi.direct',
      'zhipu.direct',
    ],
  );
});

test('generated fallback preserves canonical model capabilities without invented tool rounds', () => {
  const generated = SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG.find((model) => (
    model.catalogKey === 'openai/gpt-5.6-sol'
  ));
  assert.ok(generated);
  assert.equal(generated.contextTokens, 1_050_000);
  assert.equal(generated.maxOutputTokens, 128_000);
  assert.deepEqual(generated.inputModalities, ['text', 'image']);
  assert.deepEqual(generated.outputModalities, ['text']);
  assert.equal(generated.supportsTools, true);
  assert.equal('toolCallRounds' in generated, false);

  const fallback = GENERATED_MAINSTREAM_AGENT_MODEL_FALLBACK.find((model) => (
    model.catalogKey === generated.catalogKey
  ));
  assert.ok(fallback);
  assert.equal(fallback.contextTokens, generated.contextTokens);
  assert.equal(fallback.maxOutputTokens, generated.maxOutputTokens);
  assert.deepEqual(fallback.inputModalities, generated.inputModalities);
  assert.deepEqual(fallback.outputModalities, generated.outputModalities);
  assert.equal(fallback.supportsTools, generated.supportsTools);
  assert.equal(fallback.toolCallRounds, undefined);
});
