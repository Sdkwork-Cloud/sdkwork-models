import assert from 'node:assert/strict';
import test from 'node:test';
import {
  agentModelMatchesQuery,
  compareAgentModelCatalogOptions,
  createAgentModelAccessSelection,
  createFallbackOfficialAccessChannels,
  createModelAccessChannelConfigurationTarget,
  filterModelAccessChannels,
  modelAccessChannelNeedsConfiguration,
  resolveAuthoritativeAgentModelCatalog,
  sortAgentModelCatalogOptions,
} from '../src/agent-model-access-selector/agentModelAccessCatalog.ts';

const openAiModel = {
  id: 'model.openai.gpt-latest',
  catalogKey: 'openai/gpt-latest',
  label: 'GPT Latest',
  modelId: 'gpt-latest',
  rankScore: 100,
  sourceObservedAt: '2026-08-01T00:00:00Z',
  supportedAgentProviderIds: ['codex'],
  vendorCode: 'openai',
  vendorName: 'OpenAI',
};

const qwenModel = {
  id: 'model.alibaba.qwen-latest',
  catalogKey: 'alibaba/qwen-latest',
  label: 'Qwen Latest',
  modelId: 'qwen-latest',
  rankScore: 90,
  sourceObservedAt: '2026-07-30T00:00:00Z',
  supportedAgentProviderIds: ['codex', 'claude-code'],
  vendorCode: 'alibaba',
  vendorName: 'Alibaba Cloud',
};

const anthropicModel = {
  id: 'model.anthropic.claude-latest',
  catalogKey: 'anthropic/claude-latest',
  label: 'Claude Latest',
  modelId: 'claude-latest',
  rankScore: 95,
  sourceObservedAt: '2026-08-01T00:00:00Z',
  supportedAgentProviderIds: ['claude-code'],
  vendorCode: 'anthropic',
  vendorName: 'Anthropic',
};

const relayChannel = {
  id: 'channel.team-relay',
  code: 'team-relay',
  name: 'Team Gateway',
  kind: 'relay' as const,
  baseUrl: 'https://relay.example.com/v1',
  offerings: [{
    vendorCode: 'alibaba',
    vendorName: 'Alibaba Cloud',
    models: [{
      catalogKey: 'alibaba/qwen-latest',
      model: 'qwen-latest',
      displayName: 'Qwen Latest',
    }],
  }],
  supportedAgentProviderIds: ['codex', 'claude-code'],
};

const openAiRelayChannel = {
  ...relayChannel,
  id: 'channel.openai-relay',
  offerings: [{
    vendorCode: 'openai',
    vendorName: 'OpenAI',
    models: [{
      catalogKey: 'openai/gpt-latest',
      model: 'gpt-latest',
      displayName: 'GPT Latest',
    }],
  }],
};

test('database models replace fallback models without merging', () => {
  assert.deepEqual(
    resolveAuthoritativeAgentModelCatalog([qwenModel], [openAiModel]).map((model) => model.id),
    [qwenModel.id],
  );
  assert.deepEqual(
    resolveAuthoritativeAgentModelCatalog([], [openAiModel]).map((model) => model.id),
    [openAiModel.id],
  );
});

test('model order is deterministic: curated rank first, explicit sort order for ties', () => {
  const sorted = sortAgentModelCatalogOptions([
    { ...qwenModel, id: 'second', sortOrder: 2 },
    { ...openAiModel, id: 'first', sortOrder: 1 },
    { ...openAiModel, id: 'third', label: 'Zeta', sortOrder: 3 },
  ]);
  assert.deepEqual(sorted.map((model) => model.id), ['first', 'third', 'second']);
});

test('comparator places higher-ranked (newer) models before explicit sort order', () => {
  const older = { ...openAiModel, id: 'rank-low', rankScore: 10, sortOrder: 1 };
  const newer = { ...openAiModel, id: 'rank-high', rankScore: 90, sortOrder: 2 };
  assert.deepEqual(
    [older, newer].sort(compareAgentModelCatalogOptions).map((model) => model.id),
    ['rank-high', 'rank-low'],
  );
});

test('database models without ranks inherit the curated newest-first order', () => {
  const dbGptLatest = {
    ...openAiModel,
    id: 'db.gpt-latest',
    rankScore: undefined,
    sortOrder: undefined,
    sourceObservedAt: undefined,
  };
  const dbClaudeLatest = {
    ...anthropicModel,
    id: 'db.claude-latest',
    rankScore: undefined,
    sortOrder: undefined,
    sourceObservedAt: undefined,
  };
  const resolved = resolveAuthoritativeAgentModelCatalog(
    [dbClaudeLatest, dbGptLatest],
    [openAiModel, anthropicModel],
  );
  assert.deepEqual(resolved.map((model) => model.id), [
    dbGptLatest.id,
    dbClaudeLatest.id,
  ]);
  assert.equal(resolved[0]?.rankScore, openAiModel.rankScore);
});

test('database models keep their own rank and mainstream-less rows keep their order', () => {
  const ownRank = { ...openAiModel, id: 'db.own-rank', rankScore: 7 };
  const foreign = {
    ...qwenModel,
    id: 'db.foreign',
    catalogKey: 'acme/private-model',
    modelId: 'private-model',
    rankScore: undefined,
  };
  const resolved = resolveAuthoritativeAgentModelCatalog(
    [foreign, ownRank],
    [openAiModel, anthropicModel],
  );
  assert.equal(resolved.find((model) => model.id === 'db.own-rank')?.rankScore, 7);
  assert.equal(resolved.find((model) => model.id === 'db.foreign')?.rankScore, undefined);
  assert.deepEqual(resolved.map((model) => model.id), ['db.own-rank', 'db.foreign']);
});

test('top search combines vendor, model, and access channel terms', () => {
  assert.equal(agentModelMatchesQuery(qwenModel, 'qwen gateway', [relayChannel]), true);
  assert.equal(agentModelMatchesQuery(openAiModel, 'qwen gateway', [relayChannel]), false);
  assert.equal(filterModelAccessChannels([relayChannel], 'alibaba qwen').length, 1);
  assert.equal(filterModelAccessChannels([relayChannel], 'openai').length, 0);
});

test('fallback official channels cover every catalog vendor with chat models', () => {
  const channels = createFallbackOfficialAccessChannels([
    qwenModel,
    openAiModel,
    anthropicModel,
  ]);
  // The generated presets include every catalog vendor that publishes chat
  // models; vendors without a direct ClawRouter provider keep an empty base
  // URL for manual configuration.
  assert.equal(channels.length, 15);
  const alibabaChannel = channels.find((channel) => channel.id === 'official.alibaba');
  assert.ok(alibabaChannel);
  assert.equal(alibabaChannel.baseUrl, '');
  const openAiChannel = channels.find((channel) => channel.id === 'official.openai');
  assert.ok(openAiChannel);
  assert.ok((openAiChannel.offerings[0]?.models.length ?? 0) > 0);
  assert.equal(openAiChannel?.baseUrl, 'https://api.birdcoder.com/v1');
  assert.equal(openAiChannel?.isCustom, false);
  // Generated fallback channels are marked so the picker can hide them from
  // the channel list while keeping them for model-selection resolution.
  assert.equal(openAiChannel?.source, 'fallback');
});

test('runtime official preset models build fallback channels without local vendor rows', () => {
  const channels = createFallbackOfficialAccessChannels([], [{
    baseUrl: 'https://api.example.test/v1',
    channelName: 'Example Official',
    defaultModelId: 'example-pro',
    models: [
      { catalogKey: 'example/example-mini', model: 'example-mini', displayName: 'Example Mini' },
      { catalogKey: 'example/example-pro', model: 'example-pro', displayName: 'Example Pro' },
    ],
    protocol: 'openai_compatible',
    providerCode: 'example_direct',
    providerDisplayName: 'Example direct',
    sortOrder: 0,
    vendorCode: 'example',
    vendorName: 'Example',
  }]);

  assert.equal(channels.length, 1);
  assert.equal(channels[0]?.id, 'official.example');
  assert.equal(channels[0]?.defaultModelId, 'example-pro');
  assert.deepEqual(channels[0]?.offerings[0]?.models.map((model) => model.model), [
    'example-mini',
    'example-pro',
  ]);
});

test('selection returns the preferred channel, vendor offering, and offered model', () => {
  const openAiPreset = {
    baseUrl: 'https://api.openai.com/v1',
    channelName: 'OpenAI',
    models: [{ catalogKey: 'openai/gpt-latest', model: 'gpt-latest', displayName: 'GPT Latest' }],
    protocol: 'openai_compatible',
    providerCode: 'openai_direct',
    providerDisplayName: 'OpenAI direct',
    sortOrder: 0,
    vendorCode: 'openai',
    vendorName: 'OpenAI',
  };
  const fallbackChannel = createFallbackOfficialAccessChannels([openAiModel], [openAiPreset])[0];
  assert.ok(fallbackChannel);
  const selection = createAgentModelAccessSelection(
    openAiModel,
    [fallbackChannel, openAiRelayChannel],
    openAiRelayChannel.id,
  );
  assert.equal(selection?.channel.id, openAiRelayChannel.id);
  assert.equal(selection?.offering.vendorCode, 'openai');
  assert.equal(selection?.offeredModel.model, 'gpt-latest');
  assert.equal(selection?.model.id, openAiModel.id);
});

test('unconfigured channels open configuration for the model the user selected', () => {
  const openAiPreset = {
    baseUrl: 'https://api.openai.com/v1',
    channelName: 'OpenAI',
    models: [{ catalogKey: 'openai/gpt-latest', model: 'gpt-latest', displayName: 'GPT Latest' }],
    protocol: 'openai_compatible',
    providerCode: 'openai_direct',
    providerDisplayName: 'OpenAI direct',
    sortOrder: 0,
    vendorCode: 'openai',
    vendorName: 'OpenAI',
  };
  const fallbackChannel = createFallbackOfficialAccessChannels([openAiModel], [openAiPreset])[0];
  assert.ok(fallbackChannel);
  const selection = createAgentModelAccessSelection(openAiModel, [fallbackChannel]);
  assert.ok(selection);
  assert.equal(modelAccessChannelNeedsConfiguration(fallbackChannel), true);
  assert.equal(
    modelAccessChannelNeedsConfiguration({ ...fallbackChannel, apiKeyConfigured: true }),
    false,
  );
  assert.deepEqual(createModelAccessChannelConfigurationTarget(selection), {
    ...fallbackChannel,
    defaultVendorCode: 'openai',
    defaultModelId: 'gpt-latest',
  });
});
