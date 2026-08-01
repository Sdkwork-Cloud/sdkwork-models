import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
} from 'react';
import { createPortal } from 'react-dom';
import { Bot, Check, ChevronDown, Loader2, Settings2 } from 'lucide-react';
import { UnifiedModelConfigurationDialog } from './UnifiedModelConfigurationDialog';
import type {
  UnifiedAgentModelOption,
  UnifiedAgentModelSelectorProps,
} from './unifiedAgentModelSelectorTypes';
import { useUnifiedAgentModelSelectorAnchor } from './useUnifiedAgentModelSelectorAnchor';
import './unified-agent-model-selector.css';

export function UnifiedAgentModelSelector({
  activeProviderId,
  className = '',
  disabled = false,
  fallbackLabel,
  messages,
  onCreateModelConfiguration,
  onGetApiKey,
  onOpenChange,
  onSelectModelOption,
  open,
  options,
  providerOptions,
  renderModelIcon,
  selectedModelOptionId,
}: UnifiedAgentModelSelectorProps) {
  const menuId = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [configurationDialogOpen, setConfigurationDialogOpen] = useState(false);
  const [selectingOptionId, setSelectingOptionId] = useState<string | null>(null);
  const [selectionFailed, setSelectionFailed] = useState(false);
  const enabledOptions = useMemo(
    () => options.filter((option) => !option.disabled),
    [options],
  );
  const selectedOption = options.find((option) => option.id === selectedModelOptionId);
  const selectedLabel = selectedOption?.label || fallbackLabel;
  const builtInOptions = options.filter((option) => option.kind === 'built-in');
  const customOptions = options.filter((option) => option.kind === 'custom');
  const [activeIndex, setActiveIndex] = useState(() => Math.max(
    0,
    enabledOptions.findIndex((option) => option.id === selectedModelOptionId),
  ));
  const menuStyle = useUnifiedAgentModelSelectorAnchor(triggerRef, open);
  const canAddModel = Boolean(onCreateModelConfiguration);

  useEffect(() => {
    if (!open) {
      return;
    }
    setSelectionFailed(false);
    const nextIndex = enabledOptions.findIndex((option) => option.id === selectedModelOptionId);
    setActiveIndex(Math.max(0, nextIndex));
    const frame = window.requestAnimationFrame(() => {
      const optionId = enabledOptions[Math.max(0, nextIndex)]?.id;
      if (!optionId) {
        menuRef.current?.focus();
        return;
      }
      const escapedId = typeof CSS !== 'undefined' && CSS.escape
        ? CSS.escape(optionId)
        : optionId.replace(/["\\]/gu, '\\$&');
      menuRef.current?.querySelector<HTMLElement>(`[data-model-id="${escapedId}"]`)?.focus();
    });
    return () => window.cancelAnimationFrame(frame);
  }, [enabledOptions, open, selectedModelOptionId]);

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (
        target instanceof Node
        && !triggerRef.current?.contains(target)
        && !menuRef.current?.contains(target)
      ) {
        onOpenChange(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !selectingOptionId) {
        event.preventDefault();
        onOpenChange(false);
        triggerRef.current?.focus();
      }
    };
    document.addEventListener('pointerdown', handlePointerDown);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('pointerdown', handlePointerDown);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onOpenChange, open, selectingOptionId]);

  const selectOption = async (option: UnifiedAgentModelOption) => {
    if (option.disabled || selectingOptionId) {
      return;
    }
    setSelectingOptionId(option.id);
    setSelectionFailed(false);
    try {
      await onSelectModelOption(option);
      onOpenChange(false);
      triggerRef.current?.focus();
    } catch {
      setSelectionFailed(true);
    } finally {
      setSelectingOptionId(null);
    }
  };

  const focusOption = (index: number) => {
    const optionId = enabledOptions[index]?.id;
    if (!optionId) {
      return;
    }
    const escapedId = typeof CSS !== 'undefined' && CSS.escape
      ? CSS.escape(optionId)
      : optionId.replace(/["\\]/gu, '\\$&');
    menuRef.current?.querySelector<HTMLElement>(`[data-model-id="${escapedId}"]`)?.focus();
  };

  const handleMenuKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (enabledOptions.length === 0 || selectingOptionId) {
      return;
    }
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      const direction = event.key === 'ArrowDown' ? 1 : -1;
      const nextIndex = (activeIndex + direction + enabledOptions.length) % enabledOptions.length;
      setActiveIndex(nextIndex);
      focusOption(nextIndex);
    } else if (event.key === 'Enter' || event.key === ' ') {
      const active = enabledOptions[activeIndex];
      if (active) {
        event.preventDefault();
        void selectOption(active);
      }
    } else if (event.key === 'Home' || event.key === 'End') {
      event.preventDefault();
      const nextIndex = event.key === 'Home' ? 0 : enabledOptions.length - 1;
      setActiveIndex(nextIndex);
      focusOption(nextIndex);
    }
  };

  const renderSection = (
    label: string,
    sectionOptions: readonly UnifiedAgentModelOption[],
  ) => {
    if (sectionOptions.length === 0) {
      return null;
    }
    return (
      <section aria-label={label} className="sdkwork-unified-model-section">
        <div className="sdkwork-unified-model-section-title">{label}</div>
        <div className="sdkwork-unified-model-options" role="group">
          {sectionOptions.map((option) => {
            const selected = option.id === selectedModelOptionId;
            const selecting = option.id === selectingOptionId;
            return (
              <button
                key={option.id}
                aria-selected={selected}
                className="sdkwork-unified-model-option"
                data-model-id={option.id}
                data-selected={selected ? 'true' : 'false'}
                disabled={option.disabled || Boolean(selectingOptionId)}
                onClick={() => void selectOption(option)}
                onFocus={() => {
                  const nextIndex = enabledOptions.findIndex((item) => item.id === option.id);
                  if (nextIndex >= 0) {
                    setActiveIndex(nextIndex);
                  }
                }}
                role="option"
                type="button"
              >
                <span aria-hidden="true" className="sdkwork-unified-model-option-icon">
                  {renderModelIcon?.(option) ?? <Bot size={17} />}
                </span>
                <span className="sdkwork-unified-model-option-copy">
                  <span className="sdkwork-unified-model-option-heading">
                    <span className="sdkwork-unified-model-option-label">{option.label}</span>
                    {option.kind === 'custom' ? (
                      <span className="sdkwork-unified-model-option-tag">{messages.customTag}</span>
                    ) : null}
                  </span>
                  {option.description ? (
                    <span className="sdkwork-unified-model-option-description">
                      {option.description}
                    </span>
                  ) : null}
                </span>
                {option.metadataLabel ? (
                  <span className="sdkwork-unified-model-option-meta">{option.metadataLabel}</span>
                ) : null}
                {selecting ? (
                  <Loader2 aria-hidden="true" className="sdkwork-unified-model-spinner" size={16} />
                ) : selected ? (
                  <Check aria-hidden="true" className="sdkwork-unified-model-option-check" size={16} />
                ) : null}
              </button>
            );
          })}
        </div>
      </section>
    );
  };

  const menu = open && typeof document !== 'undefined' ? createPortal(
    <div
      ref={menuRef}
      aria-label={messages.modelSelectorLabel}
      className="sdkwork-unified-model-menu"
      id={menuId}
      onKeyDown={handleMenuKeyDown}
      role="listbox"
      style={menuStyle}
      tabIndex={-1}
    >
      <div className="sdkwork-unified-model-scroll">
        {options.length === 0 ? (
          <div className="sdkwork-unified-model-empty">{messages.noModels}</div>
        ) : (
          <>
            {renderSection(messages.builtInModels, builtInOptions)}
            {renderSection(messages.customModels, customOptions)}
          </>
        )}
      </div>
      {selectionFailed ? (
        <p className="sdkwork-unified-model-selection-error" role="alert">
          {messages.selectFailed}
        </p>
      ) : null}
      {canAddModel ? (
        <div className="sdkwork-unified-model-menu-footer">
          <button
            className="sdkwork-unified-model-add"
            onClick={() => {
              onOpenChange(false);
              setConfigurationDialogOpen(true);
            }}
            type="button"
          >
            <Settings2 aria-hidden="true" size={18} />
            <span>{messages.addModel}</span>
          </button>
        </div>
      ) : null}
    </div>,
    document.body,
  ) : null;

  return (
    <div className={`sdkwork-unified-model-selector ${className}`.trim()}>
      <button
        ref={triggerRef}
        aria-controls={open ? menuId : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={selectedLabel}
        className="sdkwork-unified-model-trigger"
        disabled={disabled}
        onClick={() => onOpenChange(!open)}
        title={selectedLabel}
        type="button"
      >
        <span className="sdkwork-unified-model-trigger-label">{selectedLabel}</span>
        <ChevronDown
          aria-hidden="true"
          data-open={open ? 'true' : 'false'}
          size={15}
        />
      </button>
      {menu}
      {onCreateModelConfiguration ? (
        <UnifiedModelConfigurationDialog
          activeProviderId={activeProviderId}
          messages={messages}
          onClose={() => setConfigurationDialogOpen(false)}
          onCreate={onCreateModelConfiguration}
          onGetApiKey={onGetApiKey}
          open={configurationDialogOpen}
          options={options}
          providerOptions={providerOptions}
          returnFocusRef={triggerRef}
        />
      ) : null}
    </div>
  );
}
