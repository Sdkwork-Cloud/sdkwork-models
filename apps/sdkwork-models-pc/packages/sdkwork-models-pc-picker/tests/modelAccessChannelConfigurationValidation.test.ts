import assert from 'node:assert/strict';
import test from 'node:test';
import {
  createEmptyModelAccessChannelConfigurationDraft,
  createModelAccessChannelConfigurationDraft,
  isModelAccessChannelConfigurationDraftValid,
  normalizeModelAccessChannelConfigurationDraft,
  validateModelAccessChannelConfigurationDraft,
} from '../src/agent-model-access-selector/modelAccessChannelConfigurationValidation.ts';

const providers = [
  { id: 'codex', label: 'Codex' },
  { id: 'claude-code', label: 'Claude Code' },
  { id: 'disabled', label: 'Disabled', disabled: true },
];

function offering(vendorCode: string, vendorName: string, modelIds: string[]) {
  return {
    vendorCode,
    vendorName,
    models: modelIds.map((modelId) => ({ modelId, displayName: modelId })),
    modelIds,
  };
}

test('new channel configurations support every enabled Agent provider by default', () => {
  const draft = createEmptyModelAccessChannelConfigurationDraft(providers);
  assert.deepEqual(draft.supportedAgentProviderIds, ['codex', 'claude-code']);
  assert.equal('inputContextTokens' in draft, false);
  assert.equal('outputContextTokens' in draft, false);
  assert.equal('toolCallRounds' in draft, false);
  assert.equal('supportsMultimodal' in draft, false);
});

test('an official channel accepts one vendor with multiple models', () => {
  const draft = {
    ...createEmptyModelAccessChannelConfigurationDraft(providers),
    name: 'OpenAI Official',
    baseUrl: 'https://api.openai.com/v1/',
    apiKey: 'write-only-secret',
    offerings: [{
      vendorCode: ' openai ',
      vendorName: ' OpenAI ',
      models: [
        { modelId: ' gpt-latest ', displayName: ' GPT Latest ' },
        { modelId: 'gpt-latest', displayName: 'Duplicate' },
        { modelId: 'gpt-mini', displayName: 'GPT Mini' },
      ],
      modelIds: ['stale-compatibility-id'],
    }],
    defaultVendorCode: 'openai',
    defaultModelId: 'gpt-latest',
  };
  const normalized = normalizeModelAccessChannelConfigurationDraft(draft);
  const validation = validateModelAccessChannelConfigurationDraft(draft, 'codex');
  assert.equal(isModelAccessChannelConfigurationDraftValid(validation), true);
  assert.equal(normalized.baseUrl, 'https://api.openai.com/v1');
  assert.deepEqual(normalized.offerings[0]?.modelIds, ['gpt-latest', 'gpt-mini']);
  assert.deepEqual(normalized.offerings[0]?.models, [
    { modelId: 'gpt-latest', displayName: 'GPT Latest' },
    { modelId: 'gpt-mini', displayName: 'GPT Mini' },
  ]);
});

test('a relay channel supports multiple vendors and rejects an unknown default model', () => {
  const draft = {
    ...createEmptyModelAccessChannelConfigurationDraft(providers, 'relay'),
    name: 'Team Relay',
    baseUrl: 'https://relay.example.com/v1',
    apiKey: 'write-only-secret',
    offerings: [
      offering('openai', 'OpenAI', ['gpt-latest']),
      offering('anthropic', 'Anthropic', ['claude-latest']),
    ],
    defaultVendorCode: 'anthropic',
    defaultModelId: 'missing-model',
  };
  assert.equal(
    validateModelAccessChannelConfigurationDraft(draft, 'codex').defaultModelRequired,
    true,
  );
  draft.defaultModelId = 'claude-latest';
  assert.equal(
    isModelAccessChannelConfigurationDraftValid(
      validateModelAccessChannelConfigurationDraft(draft, 'codex'),
    ),
    true,
  );
});

test('editing a configured channel does not expose or require its existing API key', () => {
  const publicChannel = {
    id: 'channel.team-relay',
    name: 'Team Relay',
    kind: 'relay' as const,
    baseUrl: 'https://relay.example.com/v1',
    apiKeyConfigured: true,
    offerings: [{
      vendorCode: 'openai',
      vendorName: 'OpenAI',
      models: [{ model: 'gpt-latest', displayName: 'GPT Latest' }],
    }],
    supportedAgentProviderIds: ['codex'],
  };
  assert.equal('apiKey' in publicChannel, false);
  const draft = createModelAccessChannelConfigurationDraft(publicChannel);
  assert.equal(draft.apiKey, '');
  assert.equal(draft.apiKeyConfigured, true);
  assert.equal(
    validateModelAccessChannelConfigurationDraft(draft, 'codex').apiKeyRequired,
    false,
  );
});

test('editing a database channel uses its resource code as the upsert identity', () => {
  const databaseChannel = {
    id: '42',
    code: 'model-access.relay.team-gateway',
    name: 'Team Relay',
    kind: 'relay' as const,
    baseUrl: 'https://relay.example.com/v1',
    offerings: [{
      vendorCode: 'openai',
      vendorName: 'OpenAI',
      models: [{ model: 'gpt-latest', displayName: 'GPT Latest' }],
    }],
    supportedAgentProviderIds: ['codex'],
  };
  const draft = createModelAccessChannelConfigurationDraft(databaseChannel);
  assert.equal(draft.channelId, 'model-access.relay.team-gateway');
  // The API never reports whether a key exists, so an existing channel must
  // not force the user to re-enter a key that is managed outside this API.
  assert.equal(draft.apiKeyConfigured, true);
  assert.equal(
    validateModelAccessChannelConfigurationDraft(draft, 'codex').apiKeyRequired,
    false,
  );
});

test('editing restores the persisted default vendor and model instead of list order', () => {
  const publicChannel = {
    id: 'channel.team-relay',
    name: 'Team Relay',
    kind: 'relay' as const,
    defaultVendorCode: 'anthropic',
    defaultModelId: 'claude-latest',
    offerings: [
      {
        vendorCode: 'openai',
        vendorName: 'OpenAI',
        models: [{ model: 'gpt-latest', displayName: 'GPT Latest' }],
      },
      {
        vendorCode: 'anthropic',
        vendorName: 'Anthropic',
        models: [{ model: 'claude-latest', displayName: 'Claude Latest' }],
      },
    ],
    supportedAgentProviderIds: ['codex'],
  };

  const draft = createModelAccessChannelConfigurationDraft(publicChannel);
  assert.equal(draft.defaultVendorCode, 'anthropic');
  assert.equal(draft.defaultModelId, 'claude-latest');
});

test('official channels reject multiple vendors and relay channels reject duplicates', () => {
  const official = {
    ...createEmptyModelAccessChannelConfigurationDraft(providers),
    offerings: [
      offering('openai', 'OpenAI', ['gpt-latest']),
      offering('anthropic', 'Anthropic', ['claude-latest']),
    ],
  };
  assert.equal(
    validateModelAccessChannelConfigurationDraft(official, 'codex').officialVendorCountInvalid,
    true,
  );
  const relay = { ...official, kind: 'relay' as const };
  relay.offerings[1] = offering('OPENAI', 'Duplicate', ['gpt-mini']);
  assert.equal(
    validateModelAccessChannelConfigurationDraft(relay, 'codex').duplicateVendor,
    true,
  );
});

test('official validation rejects vendors without a generated direct provider preset', () => {
  const draft = {
    ...createEmptyModelAccessChannelConfigurationDraft(providers),
    name: 'Unsupported Official',
    baseUrl: 'https://example.com/v1',
    apiKey: 'write-only-secret',
    offerings: [offering('alibaba', 'Alibaba Cloud', ['qwen-latest'])],
    defaultVendorCode: 'alibaba',
    defaultModelId: 'qwen-latest',
  };
  const validation = validateModelAccessChannelConfigurationDraft(
    draft,
    'codex',
    ['openai', 'anthropic', 'google'],
  );
  assert.equal(validation.officialVendorUnsupported, true);
  assert.equal(isModelAccessChannelConfigurationDraftValid(validation), false);
});
