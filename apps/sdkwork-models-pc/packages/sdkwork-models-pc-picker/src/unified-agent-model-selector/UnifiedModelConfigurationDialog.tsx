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
import { ChevronDown, Eye, EyeOff, Loader2, X } from 'lucide-react';
import type {
  AgentModelConfigurationDraft,
  UnifiedAgentModelOption,
  UnifiedAgentModelSelectorMessages,
  UnifiedAgentProviderOption,
} from './unifiedAgentModelSelectorTypes';
import {
  createEmptyAgentModelConfigurationDraft,
  isAgentModelConfigurationDraftValid,
  normalizeAgentModelConfigurationDraft,
  parseSupportedModelIds,
  validateAgentModelConfigurationDraft,
} from './unifiedAgentModelValidation';

export interface UnifiedModelConfigurationDialogProps {
  activeProviderId: string;
  messages: UnifiedAgentModelSelectorMessages;
  onClose: () => void;
  onCreate: (draft: AgentModelConfigurationDraft) => void | Promise<void>;
  onGetApiKey?: (vendorCode: string) => void;
  open: boolean;
  options: readonly UnifiedAgentModelOption[];
  providerOptions: readonly UnifiedAgentProviderOption[];
  returnFocusRef: RefObject<HTMLElement | null>;
}

function optionalPositiveInteger(value: string): number | undefined {
  const normalized = value.trim();
  if (!normalized) {
    return undefined;
  }
  const parsed = Number(normalized);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : undefined;
}

export function UnifiedModelConfigurationDialog({
  activeProviderId,
  messages,
  onClose,
  onCreate,
  onGetApiKey,
  open,
  options,
  providerOptions,
  returnFocusRef,
}: UnifiedModelConfigurationDialogProps) {
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  const defaultModelInputRef = useRef<HTMLInputElement>(null);
  const previousActiveElementRef = useRef<HTMLElement | null>(null);
  const [draft, setDraft] = useState(() => (
    createEmptyAgentModelConfigurationDraft(providerOptions)
  ));
  const [supportedModelsText, setSupportedModelsText] = useState('');
  const [advancedOpen, setAdvancedOpen] = useState(true);
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [createFailed, setCreateFailed] = useState(false);
  const validation = useMemo(
    () => validateAgentModelConfigurationDraft(draft, options, activeProviderId),
    [activeProviderId, draft, options],
  );
  const valid = isAgentModelConfigurationDraftValid(validation);

  useEffect(() => {
    if (!open) {
      return;
    }
    previousActiveElementRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    setDraft(createEmptyAgentModelConfigurationDraft(providerOptions));
    setSupportedModelsText('');
    setAdvancedOpen(true);
    setApiKeyVisible(false);
    setSubmitted(false);
    setCreateFailed(false);
    setIsCreating(false);
    const frame = window.requestAnimationFrame(() => defaultModelInputRef.current?.focus());
    return () => {
      window.cancelAnimationFrame(frame);
      (returnFocusRef.current ?? previousActiveElementRef.current)?.focus();
    };
  }, [open, providerOptions, returnFocusRef]);

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !isCreating) {
        event.preventDefault();
        onClose();
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isCreating, onClose, open]);

  if (!open || typeof document === 'undefined') {
    return null;
  }

  const handleFocusTrap = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key !== 'Tab') {
      return;
    }
    const focusable = Array.from(dialogRef.current?.querySelectorAll<HTMLElement>(
      'a[href], button:not(:disabled), input:not(:disabled), textarea:not(:disabled)',
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
    setCreateFailed(false);
    if (!valid) {
      return;
    }
    setIsCreating(true);
    try {
      await onCreate(normalizeAgentModelConfigurationDraft(draft));
      onClose();
    } catch {
      setCreateFailed(true);
    } finally {
      setIsCreating(false);
    }
  };

  const updateNumber = (
    field: 'inputContextTokens' | 'outputContextTokens' | 'toolCallRounds',
    value: string,
  ) => {
    setDraft((current) => ({
      ...current,
      [field]: optionalPositiveInteger(value),
    }));
  };

  return createPortal(
    <div
      className="sdkwork-unified-model-dialog-backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !isCreating) {
          onClose();
        }
      }}
    >
      <div
        ref={dialogRef}
        aria-labelledby={titleId}
        aria-modal="true"
        className="sdkwork-unified-model-dialog"
        onKeyDown={handleFocusTrap}
        role="dialog"
      >
        <header className="sdkwork-unified-model-dialog-header">
          <h2 id={titleId}>{messages.addModelTitle}</h2>
          <button
            aria-label={messages.close}
            className="sdkwork-unified-model-icon-button"
            disabled={isCreating}
            onClick={onClose}
            title={messages.close}
            type="button"
          >
            <X aria-hidden="true" size={20} />
          </button>
        </header>

        <form aria-busy={isCreating} onSubmit={handleSubmit}>
          <div className="sdkwork-unified-model-dialog-body">
            <div className="sdkwork-unified-model-field-grid">
              <label className="sdkwork-unified-model-field">
                <span><strong aria-hidden="true">*</strong>{messages.vendorLabel}</span>
                <input
                  autoComplete="off"
                  maxLength={128}
                  onChange={(event) => setDraft((current) => ({
                    ...current,
                    vendorCode: event.target.value,
                  }))}
                  placeholder={messages.vendorPlaceholder}
                  value={draft.vendorCode}
                />
                {submitted && validation.vendorRequired ? (
                  <small role="alert">{messages.vendorRequired}</small>
                ) : null}
              </label>

              <label className="sdkwork-unified-model-field">
                <span><strong aria-hidden="true">*</strong>{messages.baseUrlLabel}</span>
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
                {submitted && validation.baseUrlInvalid ? (
                  <small role="alert">{messages.baseUrlInvalid}</small>
                ) : null}
              </label>
            </div>

            <label className="sdkwork-unified-model-field">
              <span><strong aria-hidden="true">*</strong>{messages.defaultModelLabel}</span>
              <input
                ref={defaultModelInputRef}
                autoComplete="off"
                maxLength={256}
                onChange={(event) => setDraft((current) => ({
                  ...current,
                  defaultModelId: event.target.value,
                }))}
                placeholder={messages.defaultModelPlaceholder}
                value={draft.defaultModelId}
              />
              {submitted && validation.defaultModelRequired ? (
                <small role="alert">{messages.defaultModelRequired}</small>
              ) : validation.duplicateModel ? (
                <small role="alert">{messages.modelAlreadyExists}</small>
              ) : null}
            </label>

            <label className="sdkwork-unified-model-field">
              <span>{messages.supportedModelsLabel}</span>
              <textarea
                maxLength={8192}
                onChange={(event) => {
                  setSupportedModelsText(event.target.value);
                  setDraft((current) => ({
                    ...current,
                    supportedModelIds: parseSupportedModelIds(event.target.value),
                  }));
                }}
                placeholder={messages.supportedModelsPlaceholder}
                rows={3}
                value={supportedModelsText}
              />
            </label>

            <label className="sdkwork-unified-model-field">
              <span className="sdkwork-unified-model-field-heading">
                <span><strong aria-hidden="true">*</strong>{messages.apiKeyLabel}</span>
                {onGetApiKey ? (
                  <button
                    className="sdkwork-unified-model-link-button"
                    disabled={!draft.vendorCode.trim() || isCreating}
                    onClick={() => onGetApiKey(draft.vendorCode.trim())}
                    type="button"
                  >
                    {messages.getApiKey}
                  </button>
                ) : null}
              </span>
              <span className="sdkwork-unified-model-secret-input">
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
                  className="sdkwork-unified-model-secret-toggle"
                  onClick={() => setApiKeyVisible((visible) => !visible)}
                  type="button"
                >
                  {apiKeyVisible ? <EyeOff aria-hidden="true" size={17} /> : <Eye aria-hidden="true" size={17} />}
                </button>
              </span>
              {submitted && validation.apiKeyRequired ? (
                <small role="alert">{messages.apiKeyRequired}</small>
              ) : null}
            </label>

            <fieldset className="sdkwork-unified-model-provider-fieldset">
              <legend>{messages.providerSection}</legend>
              <p>{messages.supportedProvidersHint}</p>
              <div className="sdkwork-unified-model-provider-grid">
                {providerOptions.map((provider) => {
                  const checked = draft.supportedProviderIds.includes(provider.id);
                  const requiredForCurrentSelection = provider.id === activeProviderId;
                  return (
                    <label key={provider.id}>
                      <input
                        checked={checked}
                        disabled={provider.disabled || requiredForCurrentSelection || isCreating}
                        onChange={(event) => setDraft((current) => ({
                          ...current,
                          supportedProviderIds: event.target.checked
                            ? [...current.supportedProviderIds, provider.id]
                            : current.supportedProviderIds.filter((id) => id !== provider.id),
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

            <section className="sdkwork-unified-model-advanced">
              <button
                aria-expanded={advancedOpen}
                className="sdkwork-unified-model-advanced-trigger"
                onClick={() => setAdvancedOpen((expanded) => !expanded)}
                type="button"
              >
                <span>{messages.advancedSettings}</span>
                <ChevronDown aria-hidden="true" data-open={advancedOpen ? 'true' : 'false'} size={18} />
              </button>
              {advancedOpen ? (
                <div className="sdkwork-unified-model-advanced-content">
                  {([
                    ['inputContextTokens', messages.inputContextLabel],
                    ['outputContextTokens', messages.outputContextLabel],
                    ['toolCallRounds', messages.toolCallRoundsLabel],
                  ] as const).map(([field, label]) => (
                    <label className="sdkwork-unified-model-field" key={field}>
                      <span>{label}</span>
                      <input
                        min={1}
                        onChange={(event) => updateNumber(field, event.target.value)}
                        placeholder={messages.useSystemDefaultPlaceholder}
                        type="number"
                        value={draft[field] ?? ''}
                      />
                    </label>
                  ))}
                  <fieldset className="sdkwork-unified-model-multimodal">
                    <legend>{messages.multimodalLabel}</legend>
                    <label>
                      <input
                        checked={!draft.supportsMultimodal}
                        name="supports-multimodal"
                        onChange={() => setDraft((current) => ({
                          ...current,
                          supportsMultimodal: false,
                        }))}
                        type="radio"
                      />
                      <span>{messages.notSupported}</span>
                    </label>
                    <label>
                      <input
                        checked={draft.supportsMultimodal}
                        name="supports-multimodal"
                        onChange={() => setDraft((current) => ({
                          ...current,
                          supportsMultimodal: true,
                        }))}
                        type="radio"
                      />
                      <span>{messages.supported}</span>
                    </label>
                  </fieldset>
                </div>
              ) : null}
            </section>

            {createFailed ? (
              <p className="sdkwork-unified-model-submit-error" role="alert">
                {messages.createFailed}
              </p>
            ) : null}
          </div>

          <footer className="sdkwork-unified-model-dialog-footer">
            <button disabled={isCreating} onClick={onClose} type="button">
              {messages.cancel}
            </button>
            <button disabled={isCreating || !valid} type="submit">
              {isCreating ? <Loader2 aria-hidden="true" size={16} /> : null}
              <span>{isCreating ? messages.creating : messages.submit}</span>
            </button>
          </footer>
        </form>
      </div>
    </div>,
    document.body,
  );
}
