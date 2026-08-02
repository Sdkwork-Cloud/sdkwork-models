import type {
  ModelOfferingConfigurationDraft,
  ModelOfferingConfigurationModelDraft,
} from './agentModelAccessSelectorTypes';

export function setModelOfferingConfigurationRows(
  offering: ModelOfferingConfigurationDraft,
  models: readonly ModelOfferingConfigurationModelDraft[],
): ModelOfferingConfigurationDraft {
  return {
    ...offering,
    models: [...models],
    modelIds: models.map((model) => model.modelId.trim()).filter(Boolean),
  };
}

export function updateModelOfferingConfigurationRow(
  offering: ModelOfferingConfigurationDraft,
  index: number,
  update: Partial<ModelOfferingConfigurationModelDraft>,
): ModelOfferingConfigurationDraft {
  return setModelOfferingConfigurationRows(
    offering,
    offering.models.map((model, modelIndex) => (
      modelIndex === index ? { ...model, ...update } : model
    )),
  );
}

export function moveModelOfferingConfigurationRow(
  offering: ModelOfferingConfigurationDraft,
  index: number,
  offset: -1 | 1,
): ModelOfferingConfigurationDraft {
  const targetIndex = index + offset;
  if (targetIndex < 0 || targetIndex >= offering.models.length) {
    return offering;
  }
  const models = [...offering.models];
  const current = models[index];
  const target = models[targetIndex];
  if (!current || !target) {
    return offering;
  }
  models[index] = target;
  models[targetIndex] = current;
  return setModelOfferingConfigurationRows(offering, models);
}

export function removeModelOfferingConfigurationRow(
  offering: ModelOfferingConfigurationDraft,
  index: number,
): ModelOfferingConfigurationDraft {
  return setModelOfferingConfigurationRows(
    offering,
    offering.models.filter((_, modelIndex) => modelIndex !== index),
  );
}

export function appendModelOfferingConfigurationRow(
  offering: ModelOfferingConfigurationDraft,
  model: ModelOfferingConfigurationModelDraft,
): ModelOfferingConfigurationDraft {
  return setModelOfferingConfigurationRows(offering, [...offering.models, model]);
}
