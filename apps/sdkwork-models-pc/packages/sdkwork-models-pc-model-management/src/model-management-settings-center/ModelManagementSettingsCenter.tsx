import { useCallback, useMemo, useRef, useState } from 'react';
import { Building2, ChevronRight, Pencil, Plus, Route, Settings2 } from 'lucide-react';
import {
  ModelAccessChannelConfigurationDialog,
  type AgentModelCatalogOption,
  type AgentProviderOption,
  type ModelAccessChannel,
  type ModelAccessChannelConfigurationDraft,
  type ModelAccessChannelKind,
  type ModelVendor,
  type OfficialModelVendorPreset,
} from '@sdkwork/models-pc-picker';
import '@sdkwork/models-pc-picker/style.css';
import './model-management-settings-center.css';
import {
  BIRDOODER_OFFICIAL_SUPPLIER_ID,
  type ModelManagementSettingsCenterProps,
  type ModelManagementSettingsMessages,
} from './modelManagementSettingsTypes';

function channelIdentity(channel: ModelAccessChannel): string {
  return channel.code?.trim() || channel.id.trim();
}

function channelSortOrder(channel: ModelAccessChannel): number {
  const value = channel.sortOrder;
  if (typeof value === 'number') {
    return value;
  }
  if (typeof value === 'string') {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return Number.MAX_SAFE_INTEGER;
}

function sortChannels(channels: readonly ModelAccessChannel[]): ModelAccessChannel[] {
  return [...channels].sort((left, right) => (
    channelSortOrder(left) - channelSortOrder(right)
  ));
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

function officialSummary(
  preset: OfficialModelVendorPreset,
  messages: ModelManagementSettingsMessages,
) {
  return (
    <article className="sdkwork-model-management-official-entry" key={preset.providerCode}>
      <header className="sdkwork-model-management-official-entry-header">
        <span className="sdkwork-model-management-official-entry-name">
          {preset.vendorName || preset.providerDisplayName}
        </span>
        <small>{messages.modelCount(preset.models?.length ?? 0)}</small>
      </header>
      <dl className="sdkwork-model-management-official-entry-facts">
        <div>
          <dt>{messages.baseUrlLabel}</dt>
          <dd title={preset.baseUrl}>{preset.baseUrl}</dd>
        </div>
        <div>
          <dt>{messages.officialVendorProtocol}</dt>
          <dd>{preset.protocol}</dd>
        </div>
        <div>
          <dt>{messages.officialVendorDefaultModel}</dt>
          <dd>
            {preset.defaultModelId
              ?? preset.models?.[0]?.model
              ?? '—'}
          </dd>
        </div>
      </dl>
      {preset.models && preset.models.length > 0 ? (
        <div className="sdkwork-model-management-official-entry-models">
          {preset.models.map((model) => (
            <span key={model.catalogKey ?? model.model}>{model.displayName || model.model}</span>
          ))}
        </div>
      ) : null}
    </article>
  );
}

function channelDetail(
  channel: ModelAccessChannel,
  providerOptions: readonly AgentProviderOption[],
  engineSelections: ModelManagementSettingsCenterProps['engineSelections'],
  messages: ModelManagementSettingsMessages,
  canEdit: boolean,
  onEdit: () => void,
  onDelete: () => Promise<void>,
  deleting: boolean,
  deleteConfirmed: boolean,
  deleteError: string | null,
  setDeleteConfirmed: (value: boolean) => void,
) {
  const bindings = engineSelections.filter((selection) => (
    selection.channelCode.trim().toLowerCase() === channelIdentity(channel).trim().toLowerCase()
  ));
  const engineLabel = (engineId: string): string => (
    providerOptions.find((provider) => provider.id === engineId)?.label ?? engineId
  );
  return (
    <div className="sdkwork-model-management-channel-detail">
      <header className="sdkwork-model-management-channel-detail-header">
        <div>
          <h2>{channel.name}</h2>
          {/* The Base URL already has its own facts row; only the description
              belongs in the header so the value is never shown twice. */}
          {channel.description ? <p>{channel.description}</p> : null}
        </div>
        <div className="sdkwork-model-management-channel-detail-actions">
          {canEdit ? (
            <button onClick={onEdit} type="button">
              <Pencil aria-hidden="true" size={14} />
              <span>{messages.edit}</span>
            </button>
          ) : null}
          <button
            className={deleteConfirmed
              ? 'sdkwork-model-management-channel-delete-confirm'
              : 'sdkwork-model-management-channel-delete'}
            disabled={deleting}
            onClick={() => {
              if (deleteConfirmed) {
                void onDelete();
                return;
              }
              setDeleteConfirmed(true);
            }}
            type="button"
          >
            <span>
              {deleting
                ? messages.deleting
                : (deleteConfirmed ? messages.deleteConfirm : messages.delete)}
            </span>
          </button>
        </div>
      </header>
      {deleteError ? (
        <p className="sdkwork-model-management-detail-error" role="alert">
          {deleteError}
        </p>
      ) : null}
      <dl className="sdkwork-model-management-channel-facts">
        <div>
          <dt>{messages.kindLabel}</dt>
          <dd>{channel.kind === 'relay'
            ? messages.relayStationsLabel
            : channel.kind === 'official'
              ? messages.officialSupplierLabel
              : messages.customConfigsLabel}</dd>
        </div>
        <div>
          <dt>{messages.baseUrlLabel}</dt>
          <dd title={channel.baseUrl}>{channel.baseUrl}</dd>
        </div>
        <div>
          <dt>{messages.defaultVendorLabel}</dt>
          <dd>{channel.defaultVendorCode}</dd>
        </div>
        <div>
          <dt>{messages.defaultModelLabel}</dt>
          <dd>{channel.defaultModelId}</dd>
        </div>
        <div>
          <dt>{messages.apiKeyLabel}</dt>
          <dd>{channel.apiKeyConfigured
            ? messages.keyConfigured
            : messages.keyNotConfigured}</dd>
        </div>
      </dl>
      <section className="sdkwork-model-management-channel-vendors">
        <h3>{messages.vendorsLabel}</h3>
        {channel.offerings.map((offering) => (
          <div className="sdkwork-model-management-channel-vendor" key={offering.vendorCode}>
            <strong>{offering.vendorName}</strong>
            <div className="sdkwork-model-management-channel-models">
              {offering.models.map((model) => (
                <span key={model.model}>{model.displayName || model.model}</span>
              ))}
            </div>
          </div>
        ))}
      </section>
      <section className="sdkwork-model-management-engine-bindings">
        <h3>{messages.engineBindingsLabel}</h3>
        {bindings.length === 0 ? (
          <p className="sdkwork-model-management-empty-inline">{messages.engineBindingsEmpty}</p>
        ) : (
          <ul>
            {bindings.map((binding) => (
              <li key={binding.engineId}>
                <span>{engineLabel(binding.engineId)}</span>
                <ChevronRight aria-hidden="true" size={13} />
                <strong>{binding.modelId}</strong>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

export function ModelManagementSettingsCenter({
  officialPresets,
  channels,
  providerOptions,
  models,
  engineSelections,
  messages,
  formMessages,
  onSaveChannel,
  onDeleteChannel,
}: ModelManagementSettingsCenterProps) {
  const [selectedSupplierId, setSelectedSupplierId] = useState(BIRDOODER_OFFICIAL_SUPPLIER_ID);
  const [creatingKind, setCreatingKind] = useState<ModelAccessChannelKind | null>(null);
  const [editingChannelCode, setEditingChannelCode] = useState<string | null>(null);
  const [deleteConfirmedFor, setDeleteConfirmedFor] = useState<string | null>(null);
  const [deletingFor, setDeletingFor] = useState<string | null>(null);
  const [deleteErrorFor, setDeleteErrorFor] = useState<string | null>(null);
  const dialogTriggerRef = useRef<HTMLButtonElement>(null);

  const officialChannels = useMemo(
    () => sortChannels(channels.filter((channel) => channel.kind === 'official')),
    [channels],
  );
  const relayChannels = useMemo(
    () => sortChannels(channels.filter((channel) => channel.kind === 'relay')),
    [channels],
  );
  const customChannels = useMemo(
    () => sortChannels(channels.filter((channel) => channel.kind === 'custom')),
    [channels],
  );
  const vendorOptions = useMemo(() => deriveVendorOptions(models), [models]);

  const selectedChannel = channels.find(
    (channel) => channelIdentity(channel) === selectedSupplierId,
  );
  const isOfficialSupplier = selectedSupplierId === BIRDOODER_OFFICIAL_SUPPLIER_ID;
  const dialogOpen = creatingKind !== null || editingChannelCode !== null;
  // Settings-owned channels are usable by every Agent engine (per-engine
  // bindings live in the engine-config rows); the shared dialog validates at
  // least one provider checkbox, so pre-fill all of them for the edit draft.
  const editingChannel = useMemo(() => {
    if (editingChannelCode === null || !selectedChannel) {
      return undefined;
    }
    return selectedChannel.supportedAgentProviderIds.length > 0
      ? selectedChannel
      : {
          ...selectedChannel,
          supportedAgentProviderIds: providerOptions
            .filter((provider) => !provider.disabled)
            .map((provider) => provider.id),
        };
  }, [editingChannelCode, providerOptions, selectedChannel]);
  const activeProviderId = providerOptions[0]?.id ?? '';

  const closeDialog = useCallback(() => {
    setCreatingKind(null);
    setEditingChannelCode(null);
    setDeleteConfirmedFor(null);
    setDeleteErrorFor(null);
  }, []);

  const handleDialogSave = useCallback(async (draft: ModelAccessChannelConfigurationDraft) => {
    const code = await onSaveChannel(draft);
    // The dialog closes itself after a successful save; select the new
    // channel so its configuration is visible right away.
    if (code) {
      setSelectedSupplierId(code);
    }
  }, [onSaveChannel]);

  const renderSupplierRow = (
    id: string,
    label: string,
    description: string | undefined,
    icon: React.ReactNode,
    selected: boolean,
    tag?: string,
  ) => (
    <button
      aria-pressed={selected}
      className="sdkwork-model-management-supplier-row"
      data-selected={selected ? 'true' : 'false'}
      key={id}
      onClick={() => {
        setSelectedSupplierId(id);
        setEditingChannelCode(null);
        setCreatingKind(null);
        setDeleteConfirmedFor(null);
        setDeleteErrorFor(null);
      }}
      type="button"
    >
      <span className="sdkwork-model-management-supplier-icon" aria-hidden="true">
        {icon}
      </span>
      <span className="sdkwork-model-management-supplier-copy">
        <span className="sdkwork-model-management-supplier-heading">
          <strong>{label}</strong>
          {tag ? (
            <small className="sdkwork-model-management-supplier-tag">{tag}</small>
          ) : null}
        </span>
        {description ? <small>{description}</small> : null}
      </span>
      <ChevronRight aria-hidden="true" className="sdkwork-model-management-supplier-chevron" size={15} />
    </button>
  );

  const handleDelete = async (channel: ModelAccessChannel) => {
    const identity = channelIdentity(channel);
    setDeletingFor(identity);
    setDeleteErrorFor(null);
    try {
      await onDeleteChannel(channel);
      setSelectedSupplierId(BIRDOODER_OFFICIAL_SUPPLIER_ID);
      setCreatingKind(null);
      setEditingChannelCode(null);
      setDeleteConfirmedFor(null);
    } catch (error) {
      // Surface the failure in the panel instead of an unhandled rejection.
      setDeleteErrorFor(error instanceof Error && error.message
        ? error.message
        : messages.deleteFailed);
      setDeleteConfirmedFor(null);
    } finally {
      setDeletingFor(null);
    }
  };

  return (
    <div className="sdkwork-model-management-center">
      <aside className="sdkwork-model-management-sidebar">
        <div className="sdkwork-model-management-sidebar-header">
          <h2>{messages.title}</h2>
          <p>{messages.description}</p>
        </div>
        <div className="sdkwork-model-management-sidebar-groups">
          <section className="sdkwork-model-management-group">
            <header className="sdkwork-model-management-group-header">
              <h3>{messages.relayStationsLabel}</h3>
              <button
                aria-label={messages.addRelayStation}
                disabled={dialogOpen}
                onClick={(event) => {
                  dialogTriggerRef.current = event.currentTarget;
                  setEditingChannelCode(null);
                  setCreatingKind('relay');
                }}
                title={messages.addRelayStation}
                type="button"
              >
                <Plus aria-hidden="true" size={14} />
              </button>
            </header>
            <div className="sdkwork-model-management-group-list">
              {renderSupplierRow(
                BIRDOODER_OFFICIAL_SUPPLIER_ID,
                messages.officialSupplierLabel,
                messages.officialSupplierDescription,
                <Building2 size={17} />,
                selectedSupplierId === BIRDOODER_OFFICIAL_SUPPLIER_ID,
                messages.defaultSupplierTag,
              )}
              {relayChannels.map((channel) => renderSupplierRow(
                channelIdentity(channel),
                channel.name,
                channel.description || channel.baseUrl,
                <Route size={17} />,
                selectedSupplierId === channelIdentity(channel),
              ))}
            </div>
          </section>

          <section className="sdkwork-model-management-group">
            <header className="sdkwork-model-management-group-header">
              <h3>{messages.officialVendorsLabel}</h3>
              <button
                aria-label={messages.addOfficialSupplier}
                disabled={dialogOpen}
                onClick={(event) => {
                  dialogTriggerRef.current = event.currentTarget;
                  setEditingChannelCode(null);
                  setCreatingKind('official');
                }}
                title={messages.addOfficialSupplier}
                type="button"
              >
                <Plus aria-hidden="true" size={14} />
              </button>
            </header>
            <div className="sdkwork-model-management-group-list">
              {officialChannels.length === 0 ? (
                <p className="sdkwork-model-management-group-empty">{messages.emptyOfficialSuppliers}</p>
              ) : officialChannels.map((channel) => renderSupplierRow(
                channelIdentity(channel),
                channel.name,
                channel.description || channel.baseUrl,
                <Building2 size={17} />,
                selectedSupplierId === channelIdentity(channel),
              ))}
            </div>
          </section>

          <section className="sdkwork-model-management-group">
            <header className="sdkwork-model-management-group-header">
              <h3>{messages.customConfigsLabel}</h3>
              <button
                aria-label={messages.addCustomConfig}
                disabled={dialogOpen}
                onClick={(event) => {
                  dialogTriggerRef.current = event.currentTarget;
                  setEditingChannelCode(null);
                  setCreatingKind('custom');
                }}
                title={messages.addCustomConfig}
                type="button"
              >
                <Plus aria-hidden="true" size={14} />
              </button>
            </header>
            <div className="sdkwork-model-management-group-list">
              {customChannels.length === 0 ? (
                <p className="sdkwork-model-management-group-empty">{messages.emptyCustomConfigs}</p>
              ) : customChannels.map((channel) => renderSupplierRow(
                channelIdentity(channel),
                channel.name,
                channel.description || channel.baseUrl,
                <Settings2 size={17} />,
                selectedSupplierId === channelIdentity(channel),
              ))}
            </div>
          </section>
        </div>
      </aside>

      <main className="sdkwork-model-management-panel">
        {isOfficialSupplier ? (
          <div className="sdkwork-model-management-official-panel">
            <header className="sdkwork-model-management-panel-header">
              <h2>{messages.officialSupplierLabel}</h2>
              <p>{messages.officialSupplierDescription}</p>
            </header>
            <h3 className="sdkwork-model-management-official-section-title">
              {messages.officialVendorsLabel}
            </h3>
            <div className="sdkwork-model-management-official-list">
              {officialPresets.length === 0 ? (
                <p className="sdkwork-model-management-empty">{messages.noSelection}</p>
              ) : officialPresets.map((preset) => officialSummary(preset, messages))}
            </div>
          </div>
        ) : selectedChannel ? (
          channelDetail(
            selectedChannel,
            providerOptions,
            engineSelections,
            messages,
            selectedChannel.kind !== 'official',
            () => {
              dialogTriggerRef.current = null;
              setEditingChannelCode(selectedSupplierId);
              setCreatingKind(null);
              setDeleteConfirmedFor(null);
              setDeleteErrorFor(null);
            },
            () => handleDelete(selectedChannel),
            deletingFor === selectedSupplierId,
            deleteConfirmedFor === selectedSupplierId,
            deleteErrorFor === selectedSupplierId ? deleteErrorFor : null,
            (value) => setDeleteConfirmedFor(value ? selectedSupplierId : null),
          )
        ) : (
          <div className="sdkwork-model-management-empty">{messages.noSelection}</div>
        )}
      </main>

      {dialogOpen ? (
        <ModelAccessChannelConfigurationDialog
          activeProviderId={activeProviderId}
          initialChannel={editingChannel}
          initialKind={creatingKind ?? undefined}
          messages={formMessages}
          models={models}
          officialVendorPresets={officialPresets}
          onClose={closeDialog}
          onDelete={editingChannel ? handleDelete : undefined}
          onSave={handleDialogSave}
          open={dialogOpen}
          providerOptions={providerOptions}
          returnFocusRef={dialogTriggerRef}
          vendorOptions={vendorOptions}
        />
      ) : null}
    </div>
  );
}

export type {
  ModelManagementChannelKind,
  ModelManagementSettingsCenterProps,
  ModelManagementSettingsMessages,
  ModelManagementEngineSelection,
} from './modelManagementSettingsTypes';
