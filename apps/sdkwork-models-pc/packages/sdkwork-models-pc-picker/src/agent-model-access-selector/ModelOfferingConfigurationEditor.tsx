import {
  ArrowDown,
  ArrowUp,
  Plus,
  Trash2,
} from 'lucide-react';
import { useId } from 'react';
import type {
  AgentModelAccessSelectorMessages,
  AgentModelCatalogOption,
  ModelOfferingConfigurationDraft,
  ModelOfferingConfigurationModelDraft,
  ModelVendor,
} from './agentModelAccessSelectorTypes';
import {
  appendModelOfferingConfigurationRow,
  moveModelOfferingConfigurationRow,
  removeModelOfferingConfigurationRow,
  updateModelOfferingConfigurationRow,
} from './modelOfferingConfigurationRows';
import { compareAgentModelCatalogOptions } from './agentModelAccessCatalog';
import { VendorCodeCombobox } from './VendorCodeCombobox';

/** Read-only token metadata summary for a model row (hover hint). */
function modelTokenSummary(model: ModelOfferingConfigurationModelDraft): string | undefined {
  const parts: string[] = [];
  if (model.contextTokens != null) {
    parts.push(`ctx ${model.contextTokens.toLocaleString()}`);
  }
  if (model.maxOutputTokens != null) {
    parts.push(`out ${model.maxOutputTokens.toLocaleString()}`);
  }
  if (model.toolCallRounds != null) {
    parts.push(`${model.toolCallRounds} rounds`);
  }
  return parts.length > 0 ? parts.join(' · ') : undefined;
}

export interface ModelOfferingConfigurationEditorProps {
  catalogModels: readonly AgentModelCatalogOption[];
  disabled: boolean;
  messages: AgentModelAccessSelectorMessages;
  offering: ModelOfferingConfigurationDraft;
  onChange: (offering: ModelOfferingConfigurationDraft) => void;
  onRemove?: () => void;
  onVendorCodeChange?: (vendorCode: string) => void;
  showVendorFields: boolean;
  vendorOptions: readonly ModelVendor[];
}

export function ModelOfferingConfigurationEditor({
  catalogModels,
  disabled,
  messages,
  offering,
  onChange,
  onRemove,
  onVendorCodeChange,
  showVendorFields,
  vendorOptions,
}: ModelOfferingConfigurationEditorProps) {
  const vendorCodeInputId = useId();
  const configuredModelIds = new Set(
    offering.models.map((model) => model.modelId.trim().toLowerCase()).filter(Boolean),
  );
  // The catalog picker lists the newest mainstream models first so the most
  // recent releases are reachable without scrolling past older entries.
  const knownModelsToAdd = [...catalogModels]
    .filter((model) => (
      !configuredModelIds.has(model.modelId.trim().toLowerCase())
    ))
    .sort(compareAgentModelCatalogOptions);

  const updateModel = (
    index: number,
    update: Partial<ModelOfferingConfigurationDraft['models'][number]>,
  ) => {
    onChange(updateModelOfferingConfigurationRow(offering, index, update));
  };

  const moveModel = (index: number, offset: -1 | 1) => {
    onChange(moveModelOfferingConfigurationRow(offering, index, offset));
  };

  return (
    <section className="sdkwork-model-access-offering-editor">
      <div className="sdkwork-model-access-offering-editor-heading">
        <span>{offering.vendorName || offering.vendorCode || messages.vendorCodeLabel}</span>
        {onRemove ? (
          <button
            aria-label={messages.removeVendor}
            disabled={disabled}
            onClick={onRemove}
            title={messages.removeVendor}
            type="button"
          >
            <Trash2 aria-hidden="true" size={16} />
          </button>
        ) : null}
      </div>

      {showVendorFields ? (
        <div className="sdkwork-model-access-field-rows">
          <label className="sdkwork-model-access-field-inline" htmlFor={vendorCodeInputId}>
            <span className="sdkwork-model-access-field-inline-label">
              <span><strong aria-hidden="true">*</strong>{messages.vendorCodeLabel}</span>
            </span>
            <VendorCodeCombobox
              disabled={disabled}
              inputId={vendorCodeInputId}
              listLabel={messages.vendorCodeLabel}
              onChange={(vendorCode) => onVendorCodeChange?.(vendorCode)}
              options={vendorOptions}
              placeholder={messages.vendorCodePlaceholder}
              value={offering.vendorCode}
            />
          </label>
          <label className="sdkwork-model-access-field-inline">
            <span className="sdkwork-model-access-field-inline-label">
              <span>{messages.vendorNameLabel}</span>
            </span>
            <input
              autoComplete="off"
              disabled={disabled}
              maxLength={128}
              onChange={(event) => onChange({
                ...offering,
                vendorName: event.target.value,
              })}
              placeholder={messages.vendorNamePlaceholder}
              value={offering.vendorName}
            />
          </label>
        </div>
      ) : null}

      <div className="sdkwork-model-access-model-editor-heading">
        <span>{messages.modelsForVendorLabel}</span>
        <div className="sdkwork-model-access-model-add-controls">
          <select
            aria-label={messages.addKnownModel}
            disabled={disabled || knownModelsToAdd.length === 0}
            onChange={(event) => {
              const model = catalogModels.find((item) => item.id === event.target.value);
              if (!model) {
                return;
              }
              onChange(appendModelOfferingConfigurationRow(offering, {
                modelId: model.modelId,
                displayName: model.label,
              }));
              event.target.value = '';
            }}
            title={messages.addKnownModel}
            value=""
          >
            <option value="">{messages.addKnownModel}</option>
            {knownModelsToAdd.map((model) => (
              <option key={model.id} value={model.id}>
                {model.label} ({model.modelId})
              </option>
            ))}
          </select>
          <button
            disabled={disabled}
            onClick={() => onChange(appendModelOfferingConfigurationRow(
              offering,
              { modelId: '', displayName: '' },
            ))}
            type="button"
          >
            <Plus aria-hidden="true" size={15} />
            <span>{messages.addModel}</span>
          </button>
        </div>
      </div>

      <div className="sdkwork-model-access-model-column-labels" aria-hidden="true">
        <span>{messages.modelDisplayNameLabel}</span>
        <span>{messages.modelIdLabel}</span>
        <span />
      </div>
      <div className="sdkwork-model-access-model-row-list">
        {offering.models.length === 0 ? (
          <p className="sdkwork-model-access-model-row-empty">{messages.noKnownModels}</p>
        ) : offering.models.map((model, index) => {
          const modelLabel = model.displayName || model.modelId || messages.addModel;
          return (
            <div className="sdkwork-model-access-model-row" key={index}>
              <input
                aria-label={`${messages.modelDisplayNameLabel}: ${modelLabel}`}
                disabled={disabled}
                maxLength={256}
                onChange={(event) => updateModel(index, { displayName: event.target.value })}
                placeholder={messages.modelDisplayNameLabel}
                value={model.displayName}
              />
              <input
                aria-label={`${messages.modelIdLabel}: ${modelLabel}`}
                autoComplete="off"
                disabled={disabled}
                maxLength={256}
                onChange={(event) => updateModel(index, { modelId: event.target.value })}
                placeholder={messages.modelsForVendorPlaceholder}
                spellCheck={false}
                title={modelTokenSummary(model)}
                value={model.modelId}
              />
              <div className="sdkwork-model-access-model-row-actions">
                <button
                  aria-label={`${messages.moveModelUp}: ${modelLabel}`}
                  disabled={disabled || index === 0}
                  onClick={() => moveModel(index, -1)}
                  title={messages.moveModelUp}
                  type="button"
                >
                  <ArrowUp aria-hidden="true" size={15} />
                </button>
                <button
                  aria-label={`${messages.moveModelDown}: ${modelLabel}`}
                  disabled={disabled || index === offering.models.length - 1}
                  onClick={() => moveModel(index, 1)}
                  title={messages.moveModelDown}
                  type="button"
                >
                  <ArrowDown aria-hidden="true" size={15} />
                </button>
                <button
                  aria-label={`${messages.removeModel}: ${modelLabel}`}
                  disabled={disabled}
                  onClick={() => onChange(removeModelOfferingConfigurationRow(offering, index))}
                  title={messages.removeModel}
                  type="button"
                >
                  <Trash2 aria-hidden="true" size={15} />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
