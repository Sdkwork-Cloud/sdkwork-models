import { useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { Check, ChevronDown } from 'lucide-react';
import type { ModelsPickerBucket, ModelsPickerGroup, ModelsPickerOption } from './model-picker-types';
import type { ModelPickerMenuPlacement } from './modelPickerMenuLayout';
import {
  resolveModelPickerMenuGridTemplate,
  resolveModelPickerMenuWidth,
  resolveModelPickerVendorColumnWidth,
} from './modelPickerVendorLayout';
import { useModelPickerMenuLayout } from './useModelPickerMenuLayout';
import { usePopoverDismiss } from './usePopoverDismiss';

export interface ModelPickerProps {
  bucket: ModelsPickerBucket;
  modelGroups: ModelsPickerGroup[];
  selectedModelId: string;
  onSelectModel: (modelId: string) => void;
  showModelMenu: boolean;
  setShowModelMenu: (value: boolean) => void;
  fallback: ModelsPickerOption;
  menuPlacement?: ModelPickerMenuPlacement;
  compact?: boolean;
  variant?: 'default' | 'flat';
  disabled?: boolean;
  /** Show model description in the popup list and trigger subtitle. Defaults to false. */
  showModelDescription?: boolean;
}

export function ModelPicker({
  bucket,
  modelGroups,
  selectedModelId,
  onSelectModel,
  showModelMenu,
  setShowModelMenu,
  fallback,
  menuPlacement = 'auto',
  compact = false,
  variant = 'default',
  disabled = false,
  showModelDescription = false,
}: ModelPickerProps) {
  const { t } = useTranslation();
  const groupsWithModels = useMemo(() => modelGroups.filter((group) => group[bucket].length > 0), [bucket, modelGroups]);
  const selectedGroup = findModelGroup(groupsWithModels, bucket, selectedModelId) || groupsWithModels[0];
  const selectedModel = findModel(groupsWithModels, bucket, selectedModelId) || firstModel(selectedGroup, bucket) || fallback;
  const selectedModelLabel = selectedModel.displayName || selectedModel.name || selectedModel.model;
  const [activeVendorCode, setActiveVendorCode] = useState(() => selectedGroup?.vendor.code || selectedModel.vendorCode);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [menuWidthHint, setMenuWidthHint] = useState<number | undefined>();
  const activeGroup = groupsWithModels.find((group) => group.vendor.code === activeVendorCode) || selectedGroup;
  const activeVendorModels = activeGroup ? activeGroup[bucket] : [];
  const vendorNames = useMemo(
    () => groupsWithModels.map((group) => group.vendor.name),
    [groupsWithModels],
  );

  useEffect(() => {
    const nextVendorCode = selectedGroup?.vendor.code || selectedModel.vendorCode;
    setActiveVendorCode((current) => (current === nextVendorCode ? current : nextVendorCode));
  }, [selectedGroup?.vendor.code, selectedModel.vendorCode]);

  useEffect(() => {
    if (!showModelMenu) {
      setMenuWidthHint(undefined);
      return;
    }
    const trigger = triggerRef.current;
    if (!trigger) {
      return;
    }
    setMenuWidthHint(trigger.getBoundingClientRect().width);
  }, [showModelMenu, vendorNames.length]);

  usePopoverDismiss(triggerRef, showModelMenu, () => setShowModelMenu(false), menuRef);

  const isFlat = variant === 'flat';
  const matchTriggerWidth = !isFlat;
  const preferredMaxHeight = isFlat ? 380 : 420;
  const vendorColumnWidth = useMemo(
    () => resolveModelPickerVendorColumnWidth({
      vendorNames,
      variant,
      menuWidth: matchTriggerWidth ? menuWidthHint : undefined,
    }),
    [matchTriggerWidth, menuWidthHint, vendorNames, variant],
  );
  const preferredMenuWidth = useMemo(
    () => resolveModelPickerMenuWidth({
      vendorColumnWidth,
      variant,
      matchTriggerWidth,
    }),
    [matchTriggerWidth, variant, vendorColumnWidth],
  );
  const menuGridStyle = useMemo(() => ({
    display: 'grid',
    gridTemplateColumns: resolveModelPickerMenuGridTemplate(vendorColumnWidth),
    gridTemplateRows: 'minmax(0, 1fr)',
    alignContent: 'stretch',
  }), [vendorColumnWidth]);

  const menuLayoutStyle = useModelPickerMenuLayout({
    triggerRef,
    menuRef,
    open: showModelMenu,
    preferredPlacement: menuPlacement,
    preferredMenuWidth,
    preferredMaxHeight,
    matchTriggerWidth,
    layoutKey: `${activeVendorCode}:${selectedModel.id}:${activeVendorModels.length}:${vendorColumnWidth}:${showModelDescription ? 'desc' : 'plain'}`,
  });

  const triggerClassName = [
    'sdkwork-model-picker-trigger',
    isFlat ? 'sdkwork-model-picker-trigger--flat' : '',
    compact ? 'sdkwork-model-picker-trigger--compact' : '',
  ].filter(Boolean).join(' ');
  const menuClassName = isFlat
    ? 'theme-aware-dark-surface sdkwork-model-picker-menu sdkwork-model-picker-menu--flat'
    : 'theme-aware-dark-surface sdkwork-model-picker-menu';
  const menuReady = typeof menuLayoutStyle.top === 'number'
    && typeof menuLayoutStyle.width === 'number'
    && typeof menuLayoutStyle.height === 'number';

  const handleSelectModel = (modelId: string) => {
    onSelectModel(modelId);
    setShowModelMenu(false);
  };

  const modelMenu = showModelMenu ? (
    <div
      ref={menuRef}
      className={`${menuClassName} ${menuReady ? 'opacity-100' : 'pointer-events-none opacity-0'}`}
      style={{
        ...menuGridStyle,
        ...(isFlat && !menuReady
          ? { width: preferredMenuWidth, minWidth: preferredMenuWidth, boxSizing: 'border-box' as const }
          : null),
        ...menuLayoutStyle,
      }}
    >
      <div className="sdkwork-model-picker-vendors">
        <div className="sdkwork-model-picker-panel-head">
          <span className="sdkwork-model-picker-panel-title">{t('playground.modelPicker.vendorSection')}</span>
        </div>
        <div className="sdkwork-model-picker-vendor-list">
          {groupsWithModels.length === 0 ? (
            <div className="sdkwork-model-picker-empty sdkwork-model-picker-empty--compact">
              {t('playground.modelPicker.noVendors')}
            </div>
          ) : (
            groupsWithModels.map((group) => {
              const isActive = group.vendor.code === activeVendorCode;
              return (
                <button
                  key={group.vendor.code}
                  type="button"
                  data-active={isActive ? 'true' : 'false'}
                  title={group.vendor.name}
                  onClick={() => setActiveVendorCode(group.vendor.code)}
                  className="sdkwork-model-picker-vendor-button"
                >
                  <span className="sdkwork-model-picker-vendor-name">{group.vendor.name}</span>
                  <span className="sdkwork-model-picker-vendor-count">{group[bucket].length}</span>
                </button>
              );
            })
          )}
        </div>
      </div>

      <div className="sdkwork-model-picker-models custom-scrollbar">
        {activeVendorModels.length === 0 ? (
          <div className="sdkwork-model-picker-empty sdkwork-model-picker-empty--compact">
            {t('playground.modelPicker.noVendorModels')}
          </div>
        ) : (
          <div className="sdkwork-model-picker-model-list">
            {activeVendorModels.map((model) => {
              const isActive = model.id === selectedModel.id;
              return (
                <button
                  key={model.id}
                  type="button"
                  data-active={isActive ? 'true' : 'false'}
                  onClick={() => handleSelectModel(model.id)}
                  className="sdkwork-model-picker-model-button"
                >
                  <div className="sdkwork-model-picker-model-copy min-w-0 flex-1">
                    <span className={`sdkwork-model-picker-model-name ${isActive ? 'is-active' : ''}`}>
                      {model.name}
                    </span>
                    {showModelDescription && model.desc ? (
                      <p className="sdkwork-model-picker-model-desc line-clamp-2">{model.desc}</p>
                    ) : null}
                  </div>
                  {isActive ? <Check className="sdkwork-model-picker-model-check h-4 w-4 shrink-0" aria-hidden="true" /> : null}
                </button>
              );
            })}
          </div>
        )}
      </div>
    </div>
  ) : null;

  return (
    <div className={`relative ${showModelMenu ? 'z-50' : ''}`}>
      <button
        ref={triggerRef}
        type="button"
        disabled={disabled}
        onClick={() => {
          if (!disabled) {
            setShowModelMenu(!showModelMenu);
          }
        }}
        className={triggerClassName}
        title={selectedModelLabel}
        aria-label={selectedModelLabel}
        aria-expanded={showModelMenu}
        aria-haspopup="listbox"
      >
        <div className="min-w-0 flex-1">
          <div
            className={`sdkwork-model-picker-trigger__label ${
              compact ? 'sdkwork-model-picker-trigger__label--compact' : 'sdkwork-model-picker-trigger__label--default'
            }`}
          >
            {selectedModelLabel}
          </div>
          {!compact && showModelDescription && (
            <div className="sdkwork-model-picker-trigger__subtitle">
              {selectedModel.vendorName} | {selectedModel.desc}
            </div>
          )}
        </div>
        <ChevronDown
          className={`sdkwork-model-picker-trigger__chevron ${showModelMenu ? 'is-open' : ''}`}
          aria-hidden="true"
        />
      </button>

      {modelMenu ? createPortal(modelMenu, document.body) : null}
    </div>
  );
}

export function createFallbackModel(
  name: string,
  desc: string,
  versionLabel: string,
  bucket: ModelsPickerBucket,
  vendorName: string,
): ModelsPickerOption {
  const outputModality = bucket === 'llms' ? 'text' : bucket === 'audios' ? 'audio' : bucket.replace(/s$/, '');
  return {
    id: `fallback/${bucket}/${name}`,
    catalogKey: `fallback/${bucket}/${name}`,
    model: name,
    name,
    displayName: name,
    desc,
    description: desc,
    ver: versionLabel,
    versionLabel,
    vendorCode: 'pending',
    vendorName,
    modalities: [bucket],
    inputModalities: [],
    outputModalities: [outputModality],
    capabilities: [],
    officialReferencePrices: [],
    priceAvailability: { status: 'unavailable' },
    providerCodes: [],
    supportsStreaming: false,
    supportsTools: false,
    supportsJsonSchema: false,
  };
}

function findModelGroup(
  groups: ModelsPickerGroup[],
  bucket: ModelsPickerBucket,
  modelId: string,
): ModelsPickerGroup | undefined {
  return groups.find((group) => group[bucket].some((model) => model.id === modelId));
}

function findModel(
  groups: ModelsPickerGroup[],
  bucket: ModelsPickerBucket,
  modelId: string,
): ModelsPickerOption | undefined {
  for (const group of groups) {
    const model = group[bucket].find((item) => item.id === modelId);
    if (model) {
      return model;
    }
  }
  return undefined;
}

function firstModel(
  group: ModelsPickerGroup | undefined,
  bucket: ModelsPickerBucket,
): ModelsPickerOption | undefined {
  return group ? group[bucket][0] : undefined;
}
