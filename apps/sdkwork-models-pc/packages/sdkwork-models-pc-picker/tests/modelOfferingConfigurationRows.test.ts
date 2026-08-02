import assert from 'node:assert/strict';
import test from 'node:test';
import {
  appendModelOfferingConfigurationRow,
  moveModelOfferingConfigurationRow,
  removeModelOfferingConfigurationRow,
} from '../src/agent-model-access-selector/modelOfferingConfigurationRows.ts';

const offering = {
  vendorCode: 'openai',
  vendorName: 'OpenAI',
  models: [
    { modelId: 'gpt-latest', displayName: 'GPT Latest' },
    { modelId: 'gpt-mini', displayName: 'GPT Mini' },
  ],
  modelIds: ['gpt-latest', 'gpt-mini'],
};

test('model row operations keep ordered models and compatibility IDs synchronized', () => {
  const moved = moveModelOfferingConfigurationRow(offering, 1, -1);
  assert.deepEqual(moved.models.map((model) => model.modelId), ['gpt-mini', 'gpt-latest']);
  assert.deepEqual(moved.modelIds, ['gpt-mini', 'gpt-latest']);

  const removed = removeModelOfferingConfigurationRow(moved, 0);
  assert.deepEqual(removed.models, [{ modelId: 'gpt-latest', displayName: 'GPT Latest' }]);
  assert.deepEqual(removed.modelIds, ['gpt-latest']);

  const appended = appendModelOfferingConfigurationRow(removed, {
    modelId: 'gpt-custom',
    displayName: 'GPT Custom',
  });
  assert.deepEqual(appended.modelIds, ['gpt-latest', 'gpt-custom']);
});
