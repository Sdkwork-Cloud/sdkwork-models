import { useEffect, useMemo, useRef, useState } from 'react';
import {
  Eye,
  EyeOff,
  Loader2,
  Plus,
  Route,
  Settings2,
  Trash2,
  X,
} from 'lucide-react';
import {
  ModelOfferingConfigurationEditor,
  createEmptyModelAccessChannelConfigurationDraft,
  createModelAccessChannelConfigurationDraft,
  createModelOfferingConfigurationDraft,
  isModelAccessChannelConfigurationDraftValid,
  normalizeModelAccessChannelConfigurationDraft,
  validateModelAccessChannelConfigurationDraft,
  type AgentModelAccessSelectorMessages,
  type AgentModelCatalogOption,
  type AgentProviderOption,
  type ModelAccessChannel,
  type ModelAccessChannelConfigurationDraft,
  type ModelOfferingConfigurationDraft,
  type ModelVendor,
} from '@sdkwork/models-pc-picker';
import type {
  ModelManagementChannelKind,
  ModelManagementSettingsMessages,
} from './modelManagementSettingsTypes';

function sameCode(left: string, right: string): boolean {
  return left.trim().toLowerCase() === right.trim().toLowerCase();
}

function deriveVendorOptions(models: readonly AgentModelCatalogOption[]): ModelVendor[] {
  const byCode = new Map<string, ModelVendor>();
  for (const model of models) {
    const code = model.vendorCode.trim();
    if (!code || code === 'unknown' || byCode.has(code.toLowerCase())) {
      continue;
    }
    byCode.set(code.toLowerCase(), {
      code,
      name: model.vendorName.trim() || code,
      sortOrder: model.sortOrder,
    });
  }
  return [...byCode.values()];
}

interface ModelManagementChannelFormProps {
  initialChannel?: ModelAccessChannel;
  kind: ModelManagementChannelKind;
  models: readonly AgentModelCatalogOption[];
  providerOptions: readonly AgentProviderOption[];
  messages: ModelManagementSettingsMessages;
  formMessages: AgentModelAccessSelectorMessages;
  onCancel: () => void;
  onDelete?: () => Promise<void>;
  /** Resolves with the saved channel code; the center selects it afterwards. */
  onSaved?: (channelCode: string) => void;
  onSave: (draft: ModelAccessChannelConfigurationDraft) => Promise<string | void>;
}

function createInitialDraft(
  initialChannel: ModelAccessChannel | undefined,
  kind: ModelManagementChannelKind,
  providerOptions: readonly AgentProviderOption[],
): ModelAccessChannelConfigurationDraft {
  const draft = initialChannel
    ? createModelAccessChannelConfigurationDraft(initialChannel)
    : createEmptyModelAccessChannelConfigurationDraft(providerOptions, kind);
  // Settings-owned channels are usable by every Agent engine; per-engine
  // bindings are managed by the chat surface. Editing keeps the channel's own
  // kind (relay/custom); only the create mode adopts the entry-point kind.
  const allProviderIds = providerOptions
    .filter((provider) => !provider.disabled)
    .map((provider) => provider.id);
  return {
    ...draft,
    kind: initialChannel ? draft.kind : kind,
    supportedAgentProviderIds: draft.supportedAgentProviderIds.length > 0
      ? draft.supportedAgentProviderIds
      : allProviderIds,
  };
}

export function ModelManagementChannelForm({
  initialChannel,
  kind,
  models,
  providerOptions,
  messages,
  formMessages,
  onCancel,
  onDelete,
  onSaved,
  onSave,
}: ModelManagementChannelFormProps) {
  const vendorOptions = useMemo(() => deriveVendorOptions(models), [models]);
  const formRef = useRef<HTMLFormElement>(null);
  const [draft, setDraft] = useState<ModelAccessChannelConfigurationDraft>(() => (
    createInitialDraft(initialChannel, kind, providerOptions)
  ));
  const [activeOfferingIndex, setActiveOfferingIndex] = useState(0);
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteConfirmed, setDeleteConfirmed] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const editing = Boolean(initialChannel);

  useEffect(() => {
    // Focus the first editable field when the form mounts.
    formRef.current?.querySelector<HTMLElement>('[data-initial-focus="true"]')?.focus();
  }, []);

  useEffect(() => {
    if (isSaving || isDeleting) {
      return undefined;
    }
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onCancel();
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isDeleting, isSaving, onCancel]);

  const validation = useMemo(() => {
    // The settings form has no per-provider checkboxes, so any provider the
    // draft already supports is a valid "active" provider for validation.
    const activeProviderId = draft.supportedAgentProviderIds[0]
      ?? providerOptions[0]?.id
      ?? '';
    return validateModelAccessChannelConfigurationDraft(draft, activeProviderId);
  }, [draft, providerOptions]);
  const valid = isModelAccessChannelConfigurationDraftValid(validation);

  const setKind = (nextKind: ModelManagementChannelKind) => {
    setDraft((current) => (nextKind === current.kind ? current : { ...current, kind: nextKind }));
    setActiveOfferingIndex(0);
  };

  const updateOffering = (index: number, offering: ModelOfferingConfigurationDraft) => {
    setDraft((current) => {
      const previousOffering = current.offerings[index];
      const affectsDefaultOffering = Boolean(previousOffering) && (
        sameCode(current.defaultVendorCode, previousOffering.vendorCode)
        || sameCode(current.defaultVendorCode, offering.vendorCode)
      );
      const defaultModelAvailable = offering.modelIds.some((modelId) => (
        sameCode(modelId, current.defaultModelId)
      ));
      return {
        ...current,
        offerings: current.offerings.map((item, offeringIndex) => (
          offeringIndex === index ? offering : item
        )),
        defaultModelId: affectsDefaultOffering && !defaultModelAvailable
          ? ''
          : current.defaultModelId,
      };
    });
  };

  const updateRelayVendor = (index: number, vendorCode: string) => {
    const vendor = vendorOptions.find((item) => sameCode(item.code, vendorCode));
    const vendorModels = vendor
      ? models.filter((model) => sameCode(model.vendorCode, vendor.code))
      : [];
    setDraft((current) => {
      const currentOffering = current.offerings[index];
      const offering = currentOffering && sameCode(currentOffering.vendorCode, vendorCode)
        ? currentOffering
        : vendor
          ? createModelOfferingConfigurationDraft(vendor.code, vendor.name, vendorModels)
          : createModelOfferingConfigurationDraft(vendorCode, vendorCode);
      return {
        ...current,
        offerings: current.offerings.map((item, offeringIndex) => (
          offeringIndex === index ? offering : item
        )),
        defaultVendorCode: sameCode(
          current.defaultVendorCode,
          current.offerings[index]?.vendorCode ?? '',
        )
          ? offering.vendorCode
          : current.defaultVendorCode,
        defaultModelId: sameCode(
          current.defaultVendorCode,
          current.offerings[index]?.vendorCode ?? '',
        )
          ? offering.modelIds[0] ?? ''
          : current.defaultModelId,
      };
    });
  };

  const removeOffering = (index: number) => {
    setDraft((current) => {
      const removedVendorCode = current.offerings[index]?.vendorCode ?? '';
      return {
        ...current,
        offerings: current.offerings.filter((_, offeringIndex) => offeringIndex !== index),
        defaultVendorCode: sameCode(current.defaultVendorCode, removedVendorCode)
          ? ''
          : current.defaultVendorCode,
        defaultModelId: sameCode(current.defaultVendorCode, removedVendorCode)
          ? ''
          : current.defaultModelId,
      };
    });
    setActiveOfferingIndex((current) => {
      if (current === index) {
        return 0;
      }
      return current > index ? current - 1 : current;
    });
  };

  const addOffering = () => {
    setDraft((current) => ({
      ...current,
      offerings: [...current.offerings, createModelOfferingConfigurationDraft()],
    }));
    setActiveOfferingIndex(draft.offerings.length);
  };

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitted(true);
    setSaveError(null);
    if (!valid) {
      return;
    }
    setIsSaving(true);
    try {
      const savedCode = await onSave(normalizeModelAccessChannelConfigurationDraft(draft));
      onCancel();
      if (savedCode) {
        onSaved?.(savedCode);
      }
    } catch (error) {
      setSaveError(error instanceof Error && error.message
        ? error.message
        : messages.saveFailed);
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!onDelete) {
      return;
    }
    if (!deleteConfirmed) {
      setDeleteConfirmed(true);
      return;
    }
    setIsDeleting(true);
    setSaveError(null);
    try {
      await onDelete();
    } catch (error) {
      setSaveError(error instanceof Error && error.message
        ? error.message
        : messages.deleteFailed);
      setDeleteConfirmed(false);
    } finally {
      setIsDeleting(false);
    }
  };

  const defaultOfferingModels = draft.offerings.find((offering) => (
    sameCode(offering.vendorCode, draft.defaultVendorCode)
  ))?.models ?? [];

  return (
    <form
      ref={formRef}
      aria-busy={isSaving || isDeleting}
      className="sdkwork-model-management-form"
      onSubmit={handleSubmit}
    >
      <div className="sdkwork-model-management-form-header">
        <span className="sdkwork-model-management-form-title">
          {editing ? messages.edit : (kind === 'relay'
            ? messages.addRelayStation
            : messages.addCustomConfig)}
        </span>
        <button
          aria-label={messages.cancel}
          className="sdkwork-model-management-icon-button"
          disabled={isSaving || isDeleting}
          onClick={onCancel}
          title={messages.cancel}
          type="button"
        >
          <X aria-hidden="true" size={18} />
        </button>
      </div>

      <div className="sdkwork-model-access-kind-tabs" role="tablist">
        <button
          aria-selected={draft.kind === 'relay'}
          disabled={isSaving}
          onClick={() => setKind('relay')}
          role="tab"
          title={formMessages.relayChannelDescription}
          type="button"
        >
          <Route aria-hidden="true" size={14} />
          <span>{formMessages.relayChannelLabel}</span>
        </button>
        <button
          aria-selected={draft.kind === 'custom'}
          disabled={isSaving}
          onClick={() => setKind('custom')}
          role="tab"
          title={formMessages.customChannelDescription}
          type="button"
        >
          <Settings2 aria-hidden="true" size={14} />
          <span>{formMessages.customChannelLabel}</span>
        </button>
      </div>

      <div className="sdkwork-model-access-field-rows">
        <label className="sdkwork-model-access-field-inline">
          <span className="sdkwork-model-access-field-inline-label">
            <span><strong aria-hidden="true">*</strong>{messages.channelNameLabel}</span>
          </span>
          <input
            autoComplete="off"
            data-initial-focus="true"
            maxLength={128}
            onChange={(event) => setDraft((current) => ({
              ...current,
              name: event.target.value,
            }))}
            placeholder={formMessages.channelNamePlaceholder}
            value={draft.name}
          />
        </label>
        {submitted && validation.channelNameRequired ? (
          <small className="sdkwork-model-access-field-error" role="alert">
            {formMessages.channelNameRequired}
          </small>
        ) : null}

        <label className="sdkwork-model-access-field-inline">
          <span className="sdkwork-model-access-field-inline-label">
            <span><strong aria-hidden="true">*</strong>{messages.baseUrlLabel}</span>
          </span>
          <input
            autoComplete="url"
            maxLength={2048}
            onChange={(event) => setDraft((current) => ({
              ...current,
              baseUrl: event.target.value,
            }))}
            placeholder={formMessages.baseUrlPlaceholder}
            type="url"
            value={draft.baseUrl}
          />
        </label>
        {submitted && validation.baseUrlInvalid ? (
          <small className="sdkwork-model-access-field-error" role="alert">
            {formMessages.baseUrlInvalid}
          </small>
        ) : null}

        <label className="sdkwork-model-access-field-inline">
          <span className="sdkwork-model-access-field-inline-label">
            <span>
              {!draft.apiKeyConfigured ? <strong aria-hidden="true">*</strong> : null}
              {messages.apiKeyLabel}
            </span>
          </span>
          <span className="sdkwork-model-access-secret-input">
            <input
              autoComplete="new-password"
              maxLength={16384}
              onChange={(event) => setDraft((current) => ({
                ...current,
                apiKey: event.target.value,
              }))}
              placeholder={draft.apiKeyConfigured
                ? formMessages.apiKeyPlaceholder
                : formMessages.apiKeyRequired}
              type={apiKeyVisible ? 'text' : 'password'}
              value={draft.apiKey}
            />
            <button
              aria-label={apiKeyVisible ? formMessages.apiKeyPlaceholder : formMessages.apiKeyLabel}
              disabled={isSaving}
              onClick={() => setApiKeyVisible((current) => !current)}
              title={apiKeyVisible ? formMessages.apiKeyPlaceholder : formMessages.apiKeyLabel}
              type="button"
            >
              {apiKeyVisible ? <EyeOff aria-hidden="true" size={15} /> : <Eye aria-hidden="true" size={15} />}
            </button>
          </span>
        </label>
        {draft.apiKeyConfigured ? (
          <small className="sdkwork-model-management-form-hint">
            {formMessages.apiKeyConfiguredHint}
          </small>
        ) : null}
        {submitted && validation.apiKeyRequired ? (
          <small className="sdkwork-model-access-field-error" role="alert">
            {formMessages.apiKeyRequired}
          </small>
        ) : null}
      </div>

      <fieldset className="sdkwork-model-access-offerings-fieldset">
        <legend>{messages.offeringsLabel}</legend>
        <p>{formMessages.offeringsHint}</p>
        <div className="sdkwork-model-access-offering-tabs-row">
          <div
            aria-label={messages.offeringsLabel}
            className="sdkwork-model-access-offering-tabs"
            role="tablist"
          >
            {draft.offerings.map((offering, index) => {
              const tabLabel = offering.vendorName
                || offering.vendorCode
                || formMessages.vendorCodeLabel;
              return (
                <button
                  aria-selected={activeOfferingIndex === index}
                  disabled={isSaving || isDeleting}
                  key={`${offering.vendorCode || 'vendor'}-${index}`}
                  onClick={() => setActiveOfferingIndex(index)}
                  role="tab"
                  title={tabLabel}
                  type="button"
                >
                  <span>{tabLabel}</span>
                  {index === activeOfferingIndex ? (
                    <small>{formMessages.modelCount(offering.models.length)}</small>
                  ) : null}
                </button>
              );
            })}
          </div>
          <button
            className="sdkwork-model-access-add-vendor-tab"
            disabled={isSaving || isDeleting}
            onClick={addOffering}
            title={formMessages.addVendor}
            type="button"
          >
            <Plus aria-hidden="true" size={14} />
            <span>{formMessages.addVendor}</span>
          </button>
        </div>
        <div className="sdkwork-model-access-offering-tab-panel" role="tabpanel">
          {draft.offerings.map((offering, index) => {
            if (index !== activeOfferingIndex) {
              return null;
            }
            const catalogModels = models.filter((model) => (
              sameCode(model.vendorCode, offering.vendorCode)
            ));
            return (
              <ModelOfferingConfigurationEditor
                catalogModels={catalogModels}
                disabled={isSaving || isDeleting}
                key={`${offering.vendorCode || 'vendor'}-${index}`}
                messages={formMessages}
                offering={offering}
                onChange={(nextOffering) => updateOffering(index, nextOffering)}
                onRemove={draft.offerings.length > 1
                  ? () => removeOffering(index)
                  : undefined}
                onVendorCodeChange={(vendorCode) => updateRelayVendor(index, vendorCode)}
                showVendorFields
                vendorOptions={vendorOptions}
              />
            );
          })}
        </div>
        {submitted && validation.vendorRequired ? (
          <small role="alert">{formMessages.vendorRequired}</small>
        ) : submitted && (
          validation.offeringsRequired || validation.offeringModelsRequired
        ) ? (
          <small role="alert">{formMessages.atLeastOneOfferingRequired}</small>
        ) : submitted && validation.duplicateVendor ? (
          <small role="alert">{formMessages.duplicateVendor}</small>
        ) : null}
      </fieldset>

      <div className="sdkwork-model-access-field-rows">
        <label className="sdkwork-model-access-field-inline">
          <span className="sdkwork-model-access-field-inline-label">
            <span><strong aria-hidden="true">*</strong>{messages.defaultVendorLabel}</span>
          </span>
          <select
            onChange={(event) => setDraft((current) => ({
              ...current,
              defaultVendorCode: event.target.value,
              defaultModelId: '',
            }))}
            value={draft.defaultVendorCode}
          >
            <option value="">{formMessages.vendorCodePlaceholder}</option>
            {draft.offerings.map((offering) => (
              <option key={offering.vendorCode} value={offering.vendorCode}>
                {offering.vendorName || offering.vendorCode}
              </option>
            ))}
          </select>
        </label>

        <label className="sdkwork-model-access-field-inline">
          <span className="sdkwork-model-access-field-inline-label">
            <span><strong aria-hidden="true">*</strong>{messages.defaultModelLabel}</span>
          </span>
          <select
            disabled={!draft.defaultVendorCode}
            onChange={(event) => setDraft((current) => ({
              ...current,
              defaultModelId: event.target.value,
            }))}
            value={draft.defaultModelId}
          >
            <option value="">{formMessages.defaultModelPlaceholder}</option>
            {defaultOfferingModels.map((model) => (
              <option key={model.modelId} value={model.modelId}>
                {model.displayName || model.modelId}
              </option>
            ))}
          </select>
        </label>
        {submitted && validation.defaultModelRequired ? (
          <small className="sdkwork-model-access-field-error" role="alert">
            {formMessages.defaultModelRequired}
          </small>
        ) : null}
      </div>

      {saveError ? (
        <p className="sdkwork-model-access-submit-error" role="alert">
          {saveError}
        </p>
      ) : null}

      <div className="sdkwork-model-management-form-actions">
        {onDelete ? (
          <button
            className={deleteConfirmed
              ? 'sdkwork-model-management-delete-confirm'
              : 'sdkwork-model-management-delete'}
            disabled={isSaving || isDeleting}
            onClick={handleDelete}
            type="button"
          >
            {isDeleting ? <Loader2 aria-hidden="true" size={15} /> : <Trash2 aria-hidden="true" size={15} />}
            <span>
              {isDeleting
                ? messages.deleting
                : (deleteConfirmed ? messages.deleteConfirm : messages.delete)}
            </span>
          </button>
        ) : <span />}
        <button disabled={isSaving || isDeleting} onClick={onCancel} type="button">
          {messages.cancel}
        </button>
        <button className="sdkwork-model-management-save" disabled={isSaving || isDeleting} type="submit">
          {isSaving ? <Loader2 aria-hidden="true" size={15} /> : null}
          <span>{isSaving ? messages.saving : messages.save}</span>
        </button>
      </div>
    </form>
  );
}
