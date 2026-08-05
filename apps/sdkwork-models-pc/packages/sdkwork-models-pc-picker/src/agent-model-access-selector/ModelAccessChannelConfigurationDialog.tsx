import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  type RefObject,
} from 'react';
import { createPortal } from 'react-dom';
import {
  Building2,
  Eye,
  EyeOff,
  Loader2,
  Plus,
  Route,
  Settings2,
  X,
} from 'lucide-react';
import type {
  AgentModelAccessSelectorMessages,
  AgentModelCatalogOption,
  AgentProviderOption,
  ModelAccessApiKeyRequestContext,
  ModelAccessChannel,
  ModelAccessChannelConfigurationDraft,
  ModelAccessChannelKind,
  ModelOfferingConfigurationDraft,
  ModelVendor,
  OfficialModelVendorPreset,
} from './agentModelAccessSelectorTypes';
import {
  createEmptyModelAccessChannelConfigurationDraft,
  createModelAccessChannelConfigurationDraft,
  createModelOfferingConfigurationDraft,
  isModelAccessChannelConfigurationDraftValid,
  normalizeModelAccessChannelConfigurationDraft,
  validateModelAccessChannelConfigurationDraft,
} from './modelAccessChannelConfigurationValidation';
import { ModelOfferingConfigurationEditor } from './ModelOfferingConfigurationEditor';
import {
  applyOfficialModelVendorCatalogEntry,
  resolveOfficialModelVendorCatalog,
  resolveOfficialModelVendorPresets,
  type OfficialModelVendorCatalogEntry,
} from './officialModelVendorCatalog';

export interface ModelAccessChannelConfigurationDialogProps {
  activeProviderId: string;
  initialChannel?: ModelAccessChannel;
  /** Create-mode entry kind; ignored when an initial channel is edited.
   *  Defaults to 'official' so the chat surface keeps its current behavior. */
  initialKind?: ModelAccessChannelKind;
  messages: AgentModelAccessSelectorMessages;
  onClose: () => void;
  onGetApiKey?: (context: ModelAccessApiKeyRequestContext) => void;
  /** Removes the edited channel; only wired for user-owned channels. */
  onDelete?: (channel: ModelAccessChannel) => void | Promise<void>;
  onSave: (
    draft: ModelAccessChannelConfigurationDraft,
  ) => void | Promise<void>;
  open: boolean;
  models: readonly AgentModelCatalogOption[];
  officialVendorPresets?: readonly OfficialModelVendorPreset[];
  providerOptions: readonly AgentProviderOption[];
  returnFocusRef: RefObject<HTMLElement | null>;
  vendorOptions: readonly ModelVendor[];
}

function sameCode(left: string, right: string): boolean {
  return left.trim().toLowerCase() === right.trim().toLowerCase();
}

function findOfficialEntry(
  draft: ModelAccessChannelConfigurationDraft,
  officialCatalog: readonly OfficialModelVendorCatalogEntry[],
): OfficialModelVendorCatalogEntry | undefined {
  const vendorCode = draft.defaultVendorCode || draft.offerings[0]?.vendorCode || '';
  return officialCatalog.find((entry) => sameCode(entry.vendorCode, vendorCode));
}

/** Whether the official preset entry matches the channel's own base URL
 * (same origin). A channel pointing at a gateway/relay endpoint must not be
 * replaced by the official preset's data. */
function officialEntryMatchesChannelBaseUrl(
  entry: OfficialModelVendorCatalogEntry,
  draft: ModelAccessChannelConfigurationDraft,
): boolean {
  const channelBaseUrl = draft.baseUrl.trim();
  if (!channelBaseUrl || !entry.baseUrl) {
    return true;
  }
  try {
    return new URL(channelBaseUrl).origin === new URL(entry.baseUrl).origin;
  } catch {
    return true;
  }
}

function createInitialDraft(
  initialChannel: ModelAccessChannel | undefined,
  providerOptions: readonly AgentProviderOption[],
  officialCatalog: readonly OfficialModelVendorCatalogEntry[],
  initialKind: ModelAccessChannelKind = 'official',
): ModelAccessChannelConfigurationDraft {
  const baseDraft = initialChannel
    ? createModelAccessChannelConfigurationDraft(initialChannel)
    : createEmptyModelAccessChannelConfigurationDraft(providerOptions, initialKind);
  if (baseDraft.kind !== 'official') {
    return baseDraft;
  }
  // Replay the official preset only when it actually matches the channel
  // (same vendor and base URL origin). Official-kind channels that point
  // elsewhere — e.g. a gateway relay imported with `kind=official` — keep
  // their own data and stay editable instead of being silently overwritten
  // with an unrelated official preset.
  const selectedEntry = findOfficialEntry(baseDraft, officialCatalog);
  return selectedEntry && officialEntryMatchesChannelBaseUrl(selectedEntry, baseDraft)
    ? applyOfficialModelVendorCatalogEntry(baseDraft, selectedEntry)
    : baseDraft;
}

export function ModelAccessChannelConfigurationDialog({
  activeProviderId,
  initialChannel,
  initialKind,
  messages,
  onClose,
  onGetApiKey,
  onDelete,
  onSave,
  open,
  models,
  officialVendorPresets,
  providerOptions,
  returnFocusRef,
  vendorOptions,
}: ModelAccessChannelConfigurationDialogProps) {
  const titleId = useId();
  const offeringPanelId = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  const offeringTabsRef = useRef<HTMLDivElement>(null);
  const previousActiveElementRef = useRef<HTMLElement | null>(null);
  const officialCatalog = useMemo(
    () => resolveOfficialModelVendorCatalog(
      models,
      resolveOfficialModelVendorPresets(officialVendorPresets),
    ),
    [models, officialVendorPresets],
  );
  const officialVendorCodes = useMemo(
    () => officialCatalog.map((entry) => entry.vendorCode),
    [officialCatalog],
  );
  const [draft, setDraft] = useState<ModelAccessChannelConfigurationDraft>(() => (
    createInitialDraft(initialChannel, providerOptions, officialCatalog, initialKind)
  ));
  const [activeOfferingIndex, setActiveOfferingIndex] = useState(0);
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteConfirmed, setDeleteConfirmed] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const validation = useMemo(
    () => validateModelAccessChannelConfigurationDraft(
      draft,
      activeProviderId,
      officialVendorCodes,
    ),
    [activeProviderId, draft, officialVendorCodes],
  );
  const valid = isModelAccessChannelConfigurationDraftValid(validation);
  const editing = Boolean(initialChannel);
  const selectedOfficialEntry = draft.kind === 'official'
    ? findOfficialEntry(draft, officialCatalog)
    : undefined;
  const initialOfficialEntry = initialChannel?.kind === 'official'
    ? officialCatalog.find((entry) => sameCode(
      entry.vendorCode,
      initialChannel.offerings[0]?.vendorCode ?? '',
    ))
    : undefined;
  const defaultVendorModels = draft.offerings.find((offering) => (
    sameCode(offering.vendorCode, draft.defaultVendorCode)
  ))?.models ?? [];

  useEffect(() => {
    if (!open) {
      return;
    }
    previousActiveElementRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    setDraft(createInitialDraft(initialChannel, providerOptions, officialCatalog, initialKind));
    setActiveOfferingIndex(0);
    setApiKeyVisible(false);
    setSubmitted(false);
    setSaveError(null);
    setIsSaving(false);
    setIsDeleting(false);
    setDeleteConfirmed(false);
    const frame = window.requestAnimationFrame(() => (
      dialogRef.current?.querySelector<HTMLElement>('[data-initial-focus="true"]')?.focus()
    ));
    return () => {
      window.cancelAnimationFrame(frame);
      (returnFocusRef.current ?? previousActiveElementRef.current)?.focus();
    };
  }, [initialChannel, initialKind, officialCatalog, open, providerOptions, returnFocusRef]);

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !isSaving && !isDeleting) {
        event.preventDefault();
        onClose();
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isDeleting, isSaving, onClose, open]);

  if (!open || typeof document === 'undefined') {
    return null;
  }

  const handleFocusTrap = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key !== 'Tab') {
      return;
    }
    const focusable = Array.from(dialogRef.current?.querySelectorAll<HTMLElement>(
      'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled)',
    ) ?? []);
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitted(true);
    setSaveError(null);
    if (!valid) {
      return;
    }
    setIsSaving(true);
    try {
      await onSave(normalizeModelAccessChannelConfigurationDraft(draft));
      onClose();
    } catch (error) {
      // Surface the concrete failure reason (for example a missing platform
      // service in standalone mode) instead of a generic message.
      setSaveError(error instanceof Error && error.message
        ? error.message
        : messages.createFailed);
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!onDelete || !initialChannel) {
      return;
    }
    if (!deleteConfirmed) {
      setDeleteConfirmed(true);
      return;
    }
    setIsDeleting(true);
    setSaveError(null);
    try {
      await onDelete(initialChannel);
      onClose();
    } catch (error) {
      setSaveError(error instanceof Error && error.message
        ? error.message
        : messages.createFailed);
      setDeleteConfirmed(false);
    } finally {
      setIsDeleting(false);
    }
  };

  const setKind = (kind: ModelAccessChannelKind) => {
    setDraft((current) => {
      if (kind === current.kind) {
        return current;
      }
      if (kind === 'official') {
        const entry = findOfficialEntry(current, officialCatalog)
          ?? officialCatalog.find((item) => item.models.length > 0);
        return entry
          ? applyOfficialModelVendorCatalogEntry(current, entry)
          : { ...current, kind };
      }
      return {
        ...current,
        channelId: initialChannel?.code ?? initialChannel?.id ?? '',
        kind,
      };
    });
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
      // Re-selecting the same vendor (for example from the code combobox on
      // edit) must keep the user's configured model rows instead of wiping
      // them with a fresh catalog pre-fill.
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
      offerings: [
        ...current.offerings,
        createModelOfferingConfigurationDraft(),
      ],
    }));
    setActiveOfferingIndex(draft.offerings.length);
    window.requestAnimationFrame(() => {
      offeringTabsRef.current?.scrollTo({
        left: offeringTabsRef.current.scrollWidth,
        behavior: 'smooth',
      });
    });
  };

  const apiKeyField = (
    <>
      <label className="sdkwork-model-access-field-inline">
        <span className="sdkwork-model-access-field-inline-label">
          <span>
            {!draft.apiKeyConfigured ? <strong aria-hidden="true">*</strong> : null}
            {messages.apiKeyLabel}
          </span>
          {onGetApiKey ? (
            <button
              className="sdkwork-model-access-link-button"
              disabled={isSaving}
              onClick={() => onGetApiKey({
                channelId: draft.channelId || undefined,
                kind: draft.kind,
                vendorCode: draft.kind === 'official'
                  ? draft.offerings[0]?.vendorCode || undefined
                  : undefined,
              })}
              type="button"
            >
              {messages.getApiKey}
            </button>
          ) : null}
        </span>
        <span className="sdkwork-model-access-secret-input">
          <input
            autoComplete="new-password"
            maxLength={16384}
            onChange={(event) => setDraft((current) => ({
              ...current,
              apiKey: event.target.value,
            }))}
            placeholder={messages.apiKeyPlaceholder}
            type={apiKeyVisible ? 'text' : 'password'}
            value={draft.apiKey}
          />
          <button
            aria-label={apiKeyVisible ? messages.close : messages.apiKeyLabel}
            className="sdkwork-model-access-secret-toggle"
            onClick={() => setApiKeyVisible((visible) => !visible)}
            type="button"
          >
            {apiKeyVisible
              ? <EyeOff aria-hidden="true" size={17} />
              : <Eye aria-hidden="true" size={17} />}
          </button>
        </span>
      </label>
      {draft.apiKeyConfigured && !draft.apiKey ? (
        <small className="sdkwork-model-access-field-hint">
          {messages.apiKeyConfiguredHint}
        </small>
      ) : submitted && validation.apiKeyRequired ? (
        <small className="sdkwork-model-access-field-error" role="alert">
          {messages.apiKeyRequired}
        </small>
      ) : null}
    </>
  );

  return createPortal(
    <div
      className="sdkwork-model-access-dialog-backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !isSaving) {
          onClose();
        }
      }}
    >
      <div
        ref={dialogRef}
        aria-labelledby={titleId}
        aria-modal="true"
        className="sdkwork-model-access-dialog"
        onKeyDown={handleFocusTrap}
        role="dialog"
      >
        <header className="sdkwork-model-access-dialog-header">
          <div>
            <h2 id={titleId}>
              {editing ? messages.editAccessChannelTitle : messages.createAccessChannelTitle}
            </h2>
          </div>
          <button
            aria-label={messages.close}
            className="sdkwork-model-access-icon-button"
            disabled={isSaving}
            onClick={onClose}
            title={messages.close}
            type="button"
          >
            <X aria-hidden="true" size={20} />
          </button>
        </header>

        <form aria-busy={isSaving} onSubmit={handleSubmit}>
          <div className="sdkwork-model-access-dialog-body">
            <div
              aria-label={messages.channelKindLabel}
              className="sdkwork-model-access-kind-tabs"
              role="tablist"
            >
              <button
                aria-selected={draft.kind === 'official'}
                disabled={isSaving}
                onClick={() => setKind('official')}
                role="tab"
                title={messages.officialChannelDescription}
                type="button"
              >
                <Building2 aria-hidden="true" size={14} />
                <span>{messages.officialChannelLabel}</span>
              </button>
              <button
                aria-selected={draft.kind === 'relay'}
                disabled={isSaving}
                onClick={() => setKind('relay')}
                role="tab"
                title={messages.relayChannelDescription}
                type="button"
              >
                <Route aria-hidden="true" size={14} />
                <span>{messages.relayChannelLabel}</span>
              </button>
              <button
                aria-selected={draft.kind === 'custom'}
                disabled={isSaving}
                onClick={() => setKind('custom')}
                role="tab"
                title={messages.customChannelDescription}
                type="button"
              >
                <Settings2 aria-hidden="true" size={14} />
                <span>{messages.customChannelLabel}</span>
              </button>
            </div>

            {draft.kind === 'official' ? (
              <section className="sdkwork-model-access-official-configuration">
                <label className="sdkwork-model-access-field-inline">
                  <span className="sdkwork-model-access-field-inline-label">
                    <span><strong aria-hidden="true">*</strong>{messages.officialVendorLabel}</span>
                  </span>
                  <select
                    data-initial-focus="true"
                    disabled={isSaving || Boolean(initialOfficialEntry)}
                    onChange={(event) => {
                      const entry = officialCatalog.find((item) => (
                        sameCode(item.vendorCode, event.target.value)
                      ));
                      if (entry) {
                        setDraft((current) => applyOfficialModelVendorCatalogEntry(current, entry));
                      }
                    }}
                    value={selectedOfficialEntry?.vendorCode ?? ''}
                  >
                    <option value="">{messages.officialVendorPlaceholder}</option>
                    {officialCatalog.map((entry) => (
                      <option
                        disabled={entry.models.length === 0}
                        key={entry.providerCode}
                        value={entry.vendorCode}
                      >
                        {entry.vendorName} · {messages.modelCount(entry.models.length)}
                      </option>
                    ))}
                  </select>
                </label>
                {selectedOfficialEntry ? (
                  <>
                    <div className="sdkwork-model-access-official-summary">
                      <dl>
                        <div>
                          <dt>{messages.channelNameLabel}</dt>
                          <dd>{selectedOfficialEntry.channelName}</dd>
                        </div>
                        <div>
                          <dt>{messages.modelsForVendorLabel}</dt>
                          <dd>{messages.modelCount(selectedOfficialEntry.models.length)}</dd>
                        </div>
                        <div>
                          <dt>{messages.defaultModelLabel}</dt>
                          <dd>
                            {selectedOfficialEntry.models.find((model) => (
                              sameCode(model.modelId, draft.defaultModelId)
                            ))?.label ?? draft.defaultModelId}
                          </dd>
                        </div>
                      </dl>
                    </div>
                    <label className="sdkwork-model-access-field-inline">
                      <span className="sdkwork-model-access-field-inline-label">
                        <span><strong aria-hidden="true">*</strong>{messages.baseUrlLabel}</span>
                      </span>
                      {/* Official endpoints are fixed by sdkwork-models; the
                          server rejects any deviation, so the field is read-only. */}
                      <input
                        autoComplete="url"
                        disabled={isSaving || draft.kind === 'official'}
                        maxLength={2048}
                        onChange={(event) => setDraft((current) => ({
                          ...current,
                          baseUrl: event.target.value,
                        }))}
                        placeholder={messages.baseUrlPlaceholder}
                        type="url"
                        value={draft.baseUrl}
                      />
                    </label>
                  </>
                ) : null}
                {submitted && (
                  validation.vendorRequired || validation.officialVendorUnsupported
                ) ? (
                  <small className="sdkwork-model-access-field-error" role="alert">
                    {messages.vendorRequired}
                  </small>
                ) : submitted && validation.baseUrlInvalid ? (
                  <small className="sdkwork-model-access-field-error" role="alert">
                    {messages.baseUrlInvalid}
                  </small>
                ) : null}
                {apiKeyField}
              </section>
            ) : (
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
                    placeholder={messages.channelNamePlaceholder}
                    value={draft.name}
                  />
                </label>
                {submitted && validation.channelNameRequired ? (
                  <small className="sdkwork-model-access-field-error" role="alert">
                    {messages.channelNameRequired}
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
                    placeholder={messages.baseUrlPlaceholder}
                    type="url"
                    value={draft.baseUrl}
                  />
                </label>
                {submitted && validation.baseUrlInvalid ? (
                  <small className="sdkwork-model-access-field-error" role="alert">
                    {messages.baseUrlInvalid}
                  </small>
                ) : null}

                {apiKeyField}
              </div>
            )}

            {draft.kind !== 'official' ? (
              <>
                <fieldset className="sdkwork-model-access-offerings-fieldset">
                  <legend>{messages.offeringsLabel}</legend>
                  <p>{messages.offeringsHint}</p>
                  <div className="sdkwork-model-access-offering-tabs-row">
                    <div
                      aria-label={messages.offeringsLabel}
                      className="sdkwork-model-access-offering-tabs"
                      ref={offeringTabsRef}
                      role="tablist"
                    >
                      {draft.offerings.map((offering, index) => {
                        const tabLabel = offering.vendorName
                          || offering.vendorCode
                          || messages.vendorCodeLabel;
                        return (
                          <button
                            aria-controls={offeringPanelId}
                            aria-selected={activeOfferingIndex === index}
                            disabled={isSaving}
                            key={`${offering.vendorCode || 'vendor'}-${index}`}
                            onClick={() => setActiveOfferingIndex(index)}
                            role="tab"
                            title={tabLabel}
                            type="button"
                          >
                            <span>{tabLabel}</span>
                            {index === activeOfferingIndex ? (
                              <small>{messages.modelCount(offering.models.length)}</small>
                            ) : null}
                          </button>
                        );
                      })}
                    </div>
                    <button
                      className="sdkwork-model-access-add-vendor-tab"
                      disabled={isSaving}
                      onClick={addOffering}
                      title={messages.addVendor}
                      type="button"
                    >
                      <Plus aria-hidden="true" size={14} />
                      <span>{messages.addVendor}</span>
                    </button>
                  </div>
                  <div
                    className="sdkwork-model-access-offering-tab-panel"
                    id={offeringPanelId}
                    role="tabpanel"
                  >
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
                          disabled={isSaving}
                          key={`${offering.vendorCode || 'vendor'}-${index}`}
                          messages={messages}
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
                    <small role="alert">{messages.vendorRequired}</small>
                  ) : submitted && (
                    validation.offeringsRequired || validation.offeringModelsRequired
                  ) ? (
                    <small role="alert">{messages.atLeastOneOfferingRequired}</small>
                  ) : submitted && validation.duplicateVendor ? (
                    <small role="alert">{messages.duplicateVendor}</small>
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
                      <option value="">{messages.vendorCodePlaceholder}</option>
                      {draft.offerings.filter((offering) => offering.vendorCode).map((offering) => (
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
                      <option value="">{messages.defaultModelPlaceholder}</option>
                      {defaultVendorModels.map((model) => (
                        <option key={model.modelId} value={model.modelId}>
                          {model.displayName} ({model.modelId})
                        </option>
                      ))}
                    </select>
                  </label>
                  {submitted && validation.defaultModelRequired ? (
                    <small className="sdkwork-model-access-field-error" role="alert">
                      {messages.defaultModelRequired}
                    </small>
                  ) : null}
                </div>
              </>
            ) : null}

            <fieldset className="sdkwork-model-access-provider-fieldset">
              <legend>{messages.providerSection}</legend>
              <p>{messages.supportedProvidersHint}</p>
              <div className="sdkwork-model-access-provider-grid">
                {providerOptions.map((provider) => {
                  const checked = draft.supportedAgentProviderIds.includes(provider.id);
                  return (
                    <label key={provider.id}>
                      <input
                        checked={checked}
                        disabled={provider.disabled || isSaving}
                        onChange={(event) => setDraft((current) => ({
                          ...current,
                          supportedAgentProviderIds: event.target.checked
                            ? [...current.supportedAgentProviderIds, provider.id]
                            : current.supportedAgentProviderIds.filter((id) => id !== provider.id),
                        }))}
                        type="checkbox"
                      />
                      <span>{provider.label}</span>
                    </label>
                  );
                })}
              </div>
              {submitted && validation.providerRequired ? (
                <small role="alert">{messages.providerRequired}</small>
              ) : null}
            </fieldset>

            {saveError ? (
              <p className="sdkwork-model-access-submit-error" role="alert">
                {saveError}
              </p>
            ) : null}
          </div>

          <footer className="sdkwork-model-access-dialog-footer">
            {onDelete && editing && initialChannel?.isCustom !== false ? (
              <button
                className={deleteConfirmed
                  ? 'sdkwork-model-access-delete-confirm'
                  : 'sdkwork-model-access-delete'}
                disabled={isSaving || isDeleting}
                onClick={handleDelete}
                type="button"
              >
                {isDeleting ? <Loader2 aria-hidden="true" size={16} /> : null}
                <span>
                  {isDeleting
                    ? messages.saving
                    : (deleteConfirmed
                      ? messages.deleteChannelConfirm
                      : messages.deleteChannel)}
                </span>
              </button>
            ) : <span />}
            <button disabled={isSaving || isDeleting} onClick={onClose} type="button">
              {messages.cancel}
            </button>
            <button disabled={isSaving || isDeleting} type="submit">
              {isSaving ? <Loader2 aria-hidden="true" size={16} /> : null}
              <span>
                {isSaving
                  ? (editing ? messages.saving : messages.creating)
                  : (editing ? messages.saveChanges : messages.addAccessChannel)}
              </span>
            </button>
          </footer>
        </form>
      </div>
    </div>,
    document.body,
  );
}
