import {
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type FocusEvent as ReactFocusEvent,
  type MouseEvent as ReactMouseEvent,
} from 'react';
import { createPortal } from 'react-dom';
import {
  Check,
  ChevronDown,
  ChevronRight,
  Loader2,
  Pencil,
  Plus,
  Search,
  X,
} from 'lucide-react';
import {
  createAgentModelAccessSelection,
  filterAgentModelCatalogOptions,
  filterModelAccessChannels,
  resolveModelAccessChannels,
  resolveOfferingModel,
  sortModelOfferings,
  agentModelMatchesQuery,
} from './agentModelAccessCatalog';
import type {
  AgentModelAccessSelection,
  AgentModelAccessSelectorProps,
  AgentModelCatalogOption,
  ModelAccessChannel,
  ModelOffering,
  ModelOfferingModel,
} from './agentModelAccessSelectorTypes';
import { resolveOfficialModelVendorPresets } from './officialModelVendorCatalog';
import { resolveAgentModelCatalog } from './generatedAgentModelFallback';
import { ModelAccessChannelConfigurationDialog } from './ModelAccessChannelConfigurationDialog';
import { useAgentModelAccessSelectorAnchor } from './useAgentModelAccessSelectorAnchor';
import { VendorIcon } from '../vendor-icons/VendorIcon';
import { resolveVendorIconKey } from '../vendor-icons/vendorIconCatalog';
import { hasVendorIconSvg } from '../vendor-icons/vendorIconSvgs';
import './agent-model-access-selector.css';

function supportsProvider(
  supportedAgentProviderIds: readonly string[] | undefined,
  providerId: string,
): boolean {
  return !supportedAgentProviderIds?.length
    || supportedAgentProviderIds.includes(providerId);
}

/** Vendors whose latest model is highlighted in the recommended list. */
const RECOMMENDED_VENDOR_CODES = [
  'anthropic',
  'openai',
  'deepseek',
  'moonshot',
  'zhipu',
];

/** Matches the fixed .sdkwork-model-access-popover width in the CSS. */
const HOVER_PANEL_WIDTH = 320;
const HOVER_PANEL_VIEWPORT_GUTTER = 12;

export function AgentModelAccessSelector({
  accessChannels: databaseChannels,
  activeProviderId,
  className = '',
  disabled = false,
  fallbackLabel,
  fallbackModels,
  messages,
  models: databaseModels,
  officialVendorPresets,
  onCreateAccessChannel,
  onDeleteAccessChannel,
  onGetApiKey,
  onOpenChange,
  onSelectModelAccess,
  onUpdateAccessChannel,
  open,
  providerOptions,
  renderModelIcon,
  selectedAccessChannelId,
  selectedModelOptionId,
  vendorOptions = [],
}: AgentModelAccessSelectorProps) {
  const menuId = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const moreSearchInputRef = useRef<HTMLInputElement>(null);
  const [hoveredChannel, setHoveredChannel] = useState<ModelAccessChannel | null>(null);
  const [showMoreModels, setShowMoreModels] = useState(false);
  const [moreSearchQuery, setMoreSearchQuery] = useState('');
  const panelRef = useRef<HTMLDivElement>(null);
  const clearHoverPanel = () => {
    setHoveredChannel(null);
    setShowMoreModels(false);
  };
  // Leave/blur events keep the panel open when the pointer or focus moves
  // into the hover detail panel itself, or when it only crosses the menu's
  // non-interactive padding (for example the gap between a full-width row and
  // the menu edge on the way to the panel). Landing on another interactive
  // menu item closes the panel.
  const keepsHoverPanelOpen = (
    event: ReactMouseEvent<HTMLElement> | ReactFocusEvent<HTMLElement>,
  ): boolean => {
    const nextTarget = event.relatedTarget;
    if (!(nextTarget instanceof Node)) {
      return false;
    }
    if (event.currentTarget.contains(nextTarget) || panelRef.current?.contains(nextTarget)) {
      return true;
    }
    if (menuRef.current?.contains(nextTarget)) {
      let current: Node | null = nextTarget;
      while (current && current !== menuRef.current) {
        if (current instanceof HTMLElement
          && current.matches('button, input, select, textarea, a, [role="option"], [role="tab"]')) {
          return false;
        }
        current = current.parentNode;
      }
      return true;
    }
    return false;
  };
  const [selectingKey, setSelectingKey] = useState<string | null>(null);
  const [selectionFailed, setSelectionFailed] = useState(false);
  const [configurationDialogOpen, setConfigurationDialogOpen] = useState(false);
  const [editingChannel, setEditingChannel] = useState<ModelAccessChannel | undefined>();
  const models = useMemo(
    () => resolveAgentModelCatalog(databaseModels, fallbackModels),
    [databaseModels, fallbackModels],
  );
  const resolvedOfficialVendorPresets = useMemo(
    () => resolveOfficialModelVendorPresets(officialVendorPresets),
    [officialVendorPresets],
  );
  const channels = useMemo(
    () => resolveModelAccessChannels(
      databaseChannels,
      models,
      resolvedOfficialVendorPresets,
    ),
    [databaseChannels, models, resolvedOfficialVendorPresets],
  );
  const vendors = useMemo(() => {
    if (vendorOptions.length > 0) {
      return [...vendorOptions];
    }
    const byCode = new Map<string, { code: string; name: string }>();
    for (const model of models) {
      const key = model.vendorCode.trim().toLowerCase();
      if (!key || key === 'unknown') {
        continue;
      }
      if (!byCode.has(key)) {
        byCode.set(key, { code: model.vendorCode, name: model.vendorName });
      }
    }
    return [...byCode.values()];
  }, [models, vendorOptions]);
  const visibleChannels = channels.filter((channel) => channel.source !== 'fallback');
  const selectedModel = models.find((model) => model.id === selectedModelOptionId);
  const selectedChannel = channels.find((channel) => channel.id === selectedAccessChannelId);
  // Custom channel models are not flattened into the model list, so a custom
  // selection's label is resolved from the owning channel's offerings.
  const selectedOfferedModel = channels.flatMap((channel) => (
    channel.offerings.flatMap((offering) => offering.models)
  )).find((offeredModel) => offeredModel.modelOptionId === selectedModelOptionId);
  const selectedLabel = selectedModel?.label
    || selectedOfferedModel?.displayName
    || selectedOfferedModel?.model
    || fallbackLabel;
  const builtInModels = models.filter((model) => model.kind !== 'custom');
  // The recommended list shows the latest model per highlighted vendor; the
  // remaining catalog models are reachable through the "more" entry.
  const recommendedModels = useMemo(() => {
    const seenVendors = new Set<string>();
    const recommended: AgentModelCatalogOption[] = [];
    for (const model of builtInModels) {
      const vendorCode = model.vendorCode.trim().toLowerCase();
      if (!RECOMMENDED_VENDOR_CODES.includes(vendorCode) || seenVendors.has(vendorCode)) {
        continue;
      }
      seenVendors.add(vendorCode);
      recommended.push(model);
    }
    return recommended;
  }, [builtInModels]);
  const remainingModels = useMemo(() => {
    const recommendedIds = new Set(recommendedModels.map((model) => model.id));
    return builtInModels.filter((model) => !recommendedIds.has(model.id));
  }, [builtInModels, recommendedModels]);
  const filteredMoreModels = useMemo(() => (
    moreSearchQuery.trim()
      ? remainingModels.filter((model) => agentModelMatchesQuery(model, moreSearchQuery))
      : remainingModels
  ), [moreSearchQuery, remainingModels]);
  // Only user-added channels are listed; generated fallback channels stay
  // available for model-selection resolution without cluttering the list.
  const officialChannels = visibleChannels.filter((channel) => channel.kind === 'official');
  const relayChannels = visibleChannels.filter((channel) => channel.kind === 'relay');
  const customChannels = visibleChannels.filter((channel) => channel.kind === 'custom');
  const menuStyle = useAgentModelAccessSelectorAnchor(triggerRef, open);
  // The hover panel is a detached overlay anchored flush against the menu's
  // right edge so the menu itself never resizes or reflows. It is recomputed
  // after the menu repositions (window scroll/resize) and flips to the left
  // side when the right side of the viewport cannot fit it.
  const [panelAnchor, setPanelAnchor] = useState<{
    height: number;
    left: number;
    top: number;
  } | undefined>(undefined);

  useLayoutEffect(() => {
    if (!open || !(hoveredChannel || showMoreModels)) {
      setPanelAnchor(undefined);
      return;
    }
    const menuRect = menuRef.current?.getBoundingClientRect();
    if (!menuRect) {
      setPanelAnchor(undefined);
      return;
    }
    const fitsRight = menuRect.right + HOVER_PANEL_WIDTH
      <= window.innerWidth - HOVER_PANEL_VIEWPORT_GUTTER;
    const fitsLeft = menuRect.left - HOVER_PANEL_WIDTH
      >= HOVER_PANEL_VIEWPORT_GUTTER;
    setPanelAnchor({
      height: menuRect.height,
      left: fitsRight || !fitsLeft
        ? menuRect.right
        : menuRect.left - HOVER_PANEL_WIDTH,
      top: menuRect.top,
    });
  }, [hoveredChannel, menuStyle, open, showMoreModels]);

  useEffect(() => {
    if (!open) {
      return;
    }
    setHoveredChannel(null);
    setShowMoreModels(false);
    setMoreSearchQuery('');
    setSelectionFailed(false);
  }, [open]);

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
        && !panelRef.current?.contains(target)
      ) {
        onOpenChange(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !selectingKey) {
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
  }, [onOpenChange, open, selectingKey]);

  const commitSelection = async (
    selection: AgentModelAccessSelection,
    selectionKey: string,
  ) => {
    if (selectingKey) {
      return;
    }
    setSelectingKey(selectionKey);
    setSelectionFailed(false);
    try {
      const outcome = await onSelectModelAccess(selection);
      if (outcome && outcome.status === 'configuration-required') {
        // Configuration is a manual action: keep the menu open and surface the
        // failure instead of jumping to the channel editor automatically.
        setSelectionFailed(true);
        return;
      }
      onOpenChange(false);
      triggerRef.current?.focus();
    } catch {
      setSelectionFailed(true);
    } finally {
      setSelectingKey(null);
    }
  };

  const selectModel = (model: AgentModelCatalogOption) => {
    const compatibleChannels = channels.filter((channel) => (
      supportsProvider(channel.supportedAgentProviderIds, activeProviderId)
    ));
    const selection = createAgentModelAccessSelection(
      model,
      compatibleChannels,
      selectedAccessChannelId,
    );
    if (selection) {
      void commitSelection(selection, `model:${model.id}`);
    }
  };

  const selectOffering = (
    channel: ModelAccessChannel,
    offering: ModelOffering,
    offeredModel: ModelOfferingModel,
  ) => {
    const model = resolveOfferingModel(offering, offeredModel, models);
    void commitSelection(
      { channel, model, offering, offeredModel },
      `offering:${channel.id}:${offering.vendorCode}:${offeredModel.model}`,
    );
  };

  const isModelDisabled = (model: AgentModelCatalogOption): boolean => {
    if (
      model.disabled
      || !supportsProvider(model.supportedAgentProviderIds, activeProviderId)
    ) {
      return true;
    }
    return !channels.some((channel) => (
      !channel.disabled
      && supportsProvider(channel.supportedAgentProviderIds, activeProviderId)
      && channel.offerings.some((offering) => (
        offering.models.some((offeredModel) => (
          (offeredModel.modelOptionId && offeredModel.modelOptionId === model.id)
          || (offeredModel.catalogKey && offeredModel.catalogKey === model.catalogKey)
          || (
            offering.vendorCode.toLowerCase() === model.vendorCode.toLowerCase()
            && offeredModel.model.toLowerCase() === model.modelId.toLowerCase()
          )
        ))
      ))
    ));
  };

  const renderModelOption = (model: AgentModelCatalogOption) => {
    const selected = model.id === selectedModelOptionId;
    const selecting = selectingKey === `model:${model.id}`;
    // The consumer may opt out of the leading icon entirely; the row then
    // collapses to the label and status columns for a tighter layout. Without
    // a custom renderer the vendor brand icon is used when one resolves.
    const modelIcon = renderModelIcon?.(model) ?? resolveDefaultModelIcon(model);
    return (
      <button
        key={model.id}
        aria-pressed={selected}
        className="sdkwork-model-access-model-option"
        data-no-icon={modelIcon == null ? 'true' : 'false'}
        data-selected={selected ? 'true' : 'false'}
        disabled={isModelDisabled(model) || Boolean(selectingKey)}
        onClick={() => selectModel(model)}
        type="button"
      >
        {modelIcon != null ? (
          <span aria-hidden="true" className="sdkwork-model-access-model-icon">
            {modelIcon}
          </span>
        ) : null}
        <span className="sdkwork-model-access-model-heading">
          <span>{model.label}</span>
          {model.kind === 'custom' ? (
            <small>{messages.customTag}</small>
          ) : model.releaseStage === 'preview' ? (
            <small data-stage="preview">{messages.previewTag}</small>
          ) : null}
        </span>
        {selecting ? (
          <Loader2 aria-hidden="true" className="sdkwork-model-access-spinner" size={16} />
        ) : selected ? (
          <Check aria-hidden="true" size={16} />
        ) : null}
      </button>
    );
  };

  const renderModelSection = (
    label: string,
    sectionModels: readonly AgentModelCatalogOption[],
    moreEntry?: { count: number },
  ) => {
    if (sectionModels.length === 0) {
      return null;
    }
    return (
      <section aria-label={label} className="sdkwork-model-access-model-section">
        <div className="sdkwork-model-access-section-title">{label}</div>
        <div className="sdkwork-model-access-model-list">
          {sectionModels.map(renderModelOption)}
          {moreEntry ? (
            <button
              className="sdkwork-model-access-more-models"
              onBlur={(event) => {
                if (!keepsHoverPanelOpen(event)) {
                  setShowMoreModels(false);
                }
              }}
              onFocus={() => {
                setHoveredChannel(null);
                setShowMoreModels(true);
              }}
              onMouseEnter={() => {
                setHoveredChannel(null);
                setShowMoreModels(true);
              }}
              onMouseLeave={(event) => {
                if (!keepsHoverPanelOpen(event)) {
                  setShowMoreModels(false);
                }
              }}
              type="button"
            >
              <span>{messages.moreModels}</span>
              <small>{messages.modelCount(moreEntry.count)}</small>
              <ChevronRight aria-hidden="true" size={15} />
            </button>
          ) : null}
        </div>
      </section>
    );
  };

  const renderChannelEntry = (channel: ModelAccessChannel) => {
    const channelSupported = supportsProvider(
      channel.supportedAgentProviderIds,
      activeProviderId,
    );
    const selected = channel.id === selectedAccessChannelId;
    return (
      <div
        className="sdkwork-model-access-channel-entry"
        data-disabled={!channelSupported || channel.disabled ? 'true' : 'false'}
        data-selected={selected ? 'true' : 'false'}
        key={channel.id}
        onBlur={(event) => {
          if (!keepsHoverPanelOpen(event)) {
            setHoveredChannel((current) => (
              current?.id === channel.id ? null : current
            ));
          }
        }}
        onFocus={() => {
          setHoveredChannel(channel);
          setShowMoreModels(false);
        }}
        onMouseEnter={() => {
          setHoveredChannel(channel);
          setShowMoreModels(false);
        }}
        onMouseLeave={(event) => {
          if (!keepsHoverPanelOpen(event)) {
            setHoveredChannel((current) => (
              current?.id === channel.id ? null : current
            ));
          }
        }}
      >
        <button
          aria-label={channel.name}
          className="sdkwork-model-access-channel-entry-open"
          data-active={hoveredChannel?.id === channel.id ? 'true' : 'false'}
          disabled={!channelSupported || channel.disabled || Boolean(selectingKey)}
          onClick={() => {
            // Clicking a channel lists its models in the right-side detail
            // panel, matching the hover interaction.
            setHoveredChannel(channel);
            setShowMoreModels(false);
          }}
          title={channel.name}
          type="button"
        >
          <span className="sdkwork-model-access-channel-copy">
            <span className="sdkwork-model-access-channel-heading">
              <strong>{channel.name}</strong>
            </span>
            <span>{channel.description || channel.baseUrl}</span>
          </span>
          {selected ? (
            <Check
              aria-hidden="true"
              className="sdkwork-model-access-channel-entry-check"
              size={16}
            />
          ) : (
            <ChevronRight
              aria-hidden="true"
              className="sdkwork-model-access-channel-entry-chevron"
              size={16}
            />
          )}
        </button>
      </div>
    );
  };

  const renderChannelSection = (
    label: string,
    sectionChannels: readonly ModelAccessChannel[],
  ) => {
    if (sectionChannels.length === 0) {
      return null;
    }
    return (
      <section aria-label={label} className="sdkwork-model-access-channel-section">
        <div className="sdkwork-model-access-section-title">{label}</div>
        <div className="sdkwork-model-access-channel-list">
          {sectionChannels.map(renderChannelEntry)}
        </div>
      </section>
    );
  };

  const renderChannelOfferings = (
    channel: ModelAccessChannel,
    offerings: readonly ModelOffering[] = channel.offerings,
  ) => (
    <div className="sdkwork-model-access-channel-offerings">
      {sortModelOfferings(offerings).map((offering) => (
        <section key={offering.vendorCode}>
          {/* The vendor label is only needed to group multi-vendor relay
              channels; a single offering would duplicate the channel name. */}
          {offerings.length > 1 ? (
            <div className="sdkwork-model-access-vendor-name">
              <span>{offering.vendorName}</span>
            </div>
          ) : null}
          <div className="sdkwork-model-access-offering-grid">
            {offering.models.map((offeredModel) => {
              const model = resolveOfferingModel(offering, offeredModel, models);
              // Match the selection by the offering's own option identity
              // first: custom channels resolve to the built-in catalog option
              // for the same model id, which must not mark the wrong row.
              const offeringSelected = channel.id === selectedAccessChannelId
                && (
                  (offeredModel.modelOptionId
                    && offeredModel.modelOptionId === selectedModelOptionId)
                  || model.id === selectedModelOptionId
                );
              const offeringSelecting = selectingKey
                === `offering:${channel.id}:${offering.vendorCode}:${offeredModel.model}`;
              const offeringDisabled = channel.disabled
                || !supportsProvider(channel.supportedAgentProviderIds, activeProviderId)
                || model.disabled
                || !supportsProvider(model.supportedAgentProviderIds, activeProviderId);
              return (
                <button
                  key={offeredModel.catalogKey ?? offeredModel.model}
                  aria-pressed={offeringSelected}
                  disabled={offeringDisabled || Boolean(selectingKey)}
                  onClick={() => selectOffering(channel, offering, offeredModel)}
                  type="button"
                >
                  <span>{offeredModel.displayName || offeredModel.model}</span>
                  {offeringSelecting ? (
                    <Loader2
                      aria-hidden="true"
                      className="sdkwork-model-access-spinner"
                      size={14}
                    />
                  ) : offeringSelected ? (
                    <Check aria-hidden="true" size={14} />
                  ) : null}
                </button>
              );
            })}
          </div>
        </section>
      ))}
    </div>
  );

  const renderChannelDetail = (channel: ModelAccessChannel) => {
    const offerings = channel.offerings;
    const canEditChannel = Boolean(onUpdateAccessChannel)
      && (channel.isCustom !== false || channel.apiKeyConfigured !== true);
    return (
      <div className="sdkwork-model-access-detail">
        {canEditChannel ? (
          <div className="sdkwork-model-access-detail-actions">
            <button
              aria-label={messages.editAccessChannel}
              className="sdkwork-model-access-channel-edit"
              disabled={Boolean(selectingKey)}
              onClick={() => {
                setEditingChannel(channel);
                setConfigurationDialogOpen(true);
                onOpenChange(false);
              }}
              title={messages.editAccessChannel}
              type="button"
            >
              <Pencil aria-hidden="true" size={14} />
            </button>
          </div>
        ) : null}
        <div className="sdkwork-model-access-detail-scroll">
          {offerings.length === 0 ? (
            <div className="sdkwork-model-access-empty">{messages.noModels}</div>
          ) : (
            renderChannelOfferings(channel, offerings)
          )}
        </div>
      </div>
    );
  };

  const renderMoreModelsDetail = () => (
    <div className="sdkwork-model-access-detail">
      <div className="sdkwork-model-access-search">
        <Search aria-hidden="true" size={16} />
        <input
          ref={moreSearchInputRef}
          aria-label={messages.searchPlaceholder}
          autoComplete="off"
          onChange={(event) => setMoreSearchQuery(event.target.value)}
          placeholder={messages.searchPlaceholder}
          spellCheck={false}
          type="search"
          value={moreSearchQuery}
        />
        {moreSearchQuery ? (
          <button
            aria-label={messages.clearSearch}
            onClick={() => {
              setMoreSearchQuery('');
              moreSearchInputRef.current?.focus();
            }}
            title={messages.clearSearch}
            type="button"
          >
            <X aria-hidden="true" size={15} />
          </button>
        ) : null}
      </div>
      <div className="sdkwork-model-access-detail-scroll">
        {remainingModels.length === 0 ? (
          <div className="sdkwork-model-access-empty">{messages.noModels}</div>
        ) : filteredMoreModels.length === 0 ? (
          <div className="sdkwork-model-access-empty" role="status">
            {messages.noSearchResults}
          </div>
        ) : (
          <div className="sdkwork-model-access-model-list">
            {filteredMoreModels.map(renderModelOption)}
          </div>
        )}
      </div>
    </div>
  );

  const hoverPanel = open && typeof document !== 'undefined' && panelAnchor
    ? createPortal(
      <div
        ref={panelRef}
        aria-label={hoveredChannel
          ? hoveredChannel.name
          : messages.moreModels}
        className="sdkwork-model-access-popover"
        onMouseLeave={(event) => {
          if (!keepsHoverPanelOpen(event)) {
            clearHoverPanel();
          }
        }}
        role="complementary"
        style={panelAnchor}
      >
        {hoveredChannel
          ? renderChannelDetail(hoveredChannel)
          : renderMoreModelsDetail()}
      </div>,
      document.body,
    )
    : null;

  const menu = open && typeof document !== 'undefined' ? createPortal(
    <div
      ref={menuRef}
      aria-label={messages.modelAccessSelectorLabel}
      className="sdkwork-model-access-menu"
      id={menuId}
      onMouseLeave={(event) => {
        if (!keepsHoverPanelOpen(event)) {
          clearHoverPanel();
        }
      }}
      role="dialog"
      style={menuStyle}
    >
      <div className="sdkwork-model-access-scroll">
        {models.length === 0 && visibleChannels.length === 0 ? (
          <div className="sdkwork-model-access-empty">{messages.noModels}</div>
        ) : (
          <>
            {renderModelSection(
              messages.builtInModels,
              recommendedModels,
              remainingModels.length > 0 ? { count: remainingModels.length } : undefined,
            )}
            {renderChannelSection(messages.officialChannels, officialChannels)}
            {renderChannelSection(messages.relayChannels, relayChannels)}
            {renderChannelSection(messages.customChannels, customChannels)}
          </>
        )}
      </div>

      {selectionFailed ? (
        <p className="sdkwork-model-access-selection-error" role="alert">
          {messages.selectFailed}
        </p>
      ) : null}

      {onCreateAccessChannel ? (
        <footer className="sdkwork-model-access-menu-footer">
          <button
            disabled={Boolean(selectingKey)}
            onClick={() => {
              setEditingChannel(undefined);
              setConfigurationDialogOpen(true);
              onOpenChange(false);
            }}
            type="button"
          >
            <Plus aria-hidden="true" size={17} />
            <span>{messages.addAccessChannel}</span>
          </button>
        </footer>
      ) : null}
    </div>,
    document.body,
  ) : null;

  const saveConfiguration = editingChannel && onUpdateAccessChannel
    ? onUpdateAccessChannel
    : onCreateAccessChannel;

  return (
    <div className={`sdkwork-model-access-selector ${className}`.trim()}>
      <button
        ref={triggerRef}
        aria-controls={open ? menuId : undefined}
        aria-expanded={open}
        aria-haspopup="dialog"
        aria-label={selectedLabel}
        className="sdkwork-model-access-trigger"
        disabled={disabled}
        onClick={() => onOpenChange(!open)}
        title={selectedChannel ? `${selectedLabel} · ${selectedChannel.name}` : selectedLabel}
        type="button"
      >
        <span>{selectedLabel}</span>
        <ChevronDown aria-hidden="true" data-open={open ? 'true' : 'false'} size={15} />
      </button>
      {menu}
      {hoverPanel}
      {saveConfiguration ? (
        <ModelAccessChannelConfigurationDialog
          activeProviderId={activeProviderId}
          initialChannel={editingChannel}
          messages={messages}
          models={models}
          officialVendorPresets={resolvedOfficialVendorPresets}
          onClose={() => {
            setConfigurationDialogOpen(false);
            setEditingChannel(undefined);
          }}
          onDelete={onDeleteAccessChannel}
          onGetApiKey={onGetApiKey}
          onSave={saveConfiguration}
          open={configurationDialogOpen}
          providerOptions={providerOptions}
          returnFocusRef={triggerRef}
          vendorOptions={vendors}
        />
      ) : null}
    </div>
  );
}

/**
 * Default model row icon when the consumer provides no renderer: the vendor
 * brand icon resolved from the option's icon key or vendor code. Unknown
 * vendors resolve to `null` so the row keeps collapsing via `data-no-icon`.
 */
function resolveDefaultModelIcon(model: AgentModelCatalogOption) {
  const iconKey = model.iconKey ?? resolveVendorIconKey(model.vendorCode);
  return hasVendorIconSvg(iconKey) ? (
    <VendorIcon
      iconKey={iconKey}
      vendorCode={model.vendorCode}
      name={model.label}
      size={16}
    />
  ) : null;
}
