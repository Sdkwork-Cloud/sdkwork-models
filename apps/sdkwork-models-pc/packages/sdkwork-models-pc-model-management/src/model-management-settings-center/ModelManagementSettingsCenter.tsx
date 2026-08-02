import { useMemo, useState } from 'react';
import { Building2, ChevronRight, Pencil, Plus, Route, Settings2 } from 'lucide-react';
import type {
  AgentModelCatalogOption,
  AgentProviderOption,
  ModelAccessChannel,
  ModelAccessChannelConfigurationDraft,
  OfficialModelVendorPreset,
} from '@sdkwork/models-pc-picker';
import '@sdkwork/models-pc-picker/style.css';
import './model-management-settings-center.css';
import { ModelManagementChannelForm } from './ModelManagementChannelForm';
import {
  BIRDOODER_OFFICIAL_SUPPLIER_ID,
  type ModelManagementChannelKind,
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
          <dd>{preset.baseUrl}</dd>
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
          <dd>{channel.baseUrl}</dd>
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
  const [editingChannelCode, setEditingChannelCode] = useState<string | null>(null);
  const [creatingKind, setCreatingKind] = useState<ModelManagementChannelKind | null>(null);
  const [deleteConfirmedFor, setDeleteConfirmedFor] = useState<string | null>(null);
  const [deletingFor, setDeletingFor] = useState<string | null>(null);
  const [deleteErrorFor, setDeleteErrorFor] = useState<string | null>(null);

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

  const selectedChannel = channels.find(
    (channel) => channelIdentity(channel) === selectedSupplierId,
  );
  const isOfficialSupplier = selectedSupplierId === BIRDOODER_OFFICIAL_SUPPLIER_ID;
  const showCreateForm = creatingKind !== null;
  const showEditForm = Boolean(
    selectedChannel && editingChannelCode === selectedSupplierId && !showCreateForm,
  );
  const formKind: ModelManagementChannelKind = creatingKind
    ?? (selectedChannel?.kind === 'custom' ? 'custom' : 'relay');

  const renderSupplierRow = (
    id: string,
    label: string,
    description: string | undefined,
    icon: React.ReactNode,
    selected: boolean,
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
        <strong>{label}</strong>
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
            <h3>{messages.officialSupplierLabel}</h3>
            <div className="sdkwork-model-management-group-list">
              {renderSupplierRow(
                BIRDOODER_OFFICIAL_SUPPLIER_ID,
                messages.officialSupplierLabel,
                messages.officialSupplierDescription,
                <Building2 size={17} />,
                selectedSupplierId === BIRDOODER_OFFICIAL_SUPPLIER_ID,
              )}
              {officialChannels.map((channel) => renderSupplierRow(
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
              <h3>{messages.relayStationsLabel}</h3>
              <button
                aria-label={messages.addRelayStation}
                disabled={showCreateForm || showEditForm}
                onClick={() => {
                  setCreatingKind('relay');
                  setEditingChannelCode(null);
                  setDeleteConfirmedFor(null);
                }}
                title={messages.addRelayStation}
                type="button"
              >
                <Plus aria-hidden="true" size={14} />
              </button>
            </header>
            <div className="sdkwork-model-management-group-list">
              {relayChannels.length === 0 ? (
                <p className="sdkwork-model-management-group-empty">{messages.emptyRelayStations}</p>
              ) : relayChannels.map((channel) => renderSupplierRow(
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
              <h3>{messages.customConfigsLabel}</h3>
              <button
                aria-label={messages.addCustomConfig}
                disabled={showCreateForm || showEditForm}
                onClick={() => {
                  setCreatingKind('custom');
                  setEditingChannelCode(null);
                  setDeleteConfirmedFor(null);
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
        {showCreateForm || showEditForm ? (
          <ModelManagementChannelForm
            initialChannel={showEditForm ? selectedChannel : undefined}
            kind={formKind}
            messages={messages}
            formMessages={formMessages}
            models={models}
            providerOptions={providerOptions}
            onCancel={() => {
              setCreatingKind(null);
              setEditingChannelCode(null);
              setDeleteConfirmedFor(null);
              setDeleteErrorFor(null);
            }}
            onDelete={showEditForm && selectedChannel
              ? () => handleDelete(selectedChannel)
              : undefined}
            onSave={onSaveChannel}
            onSaved={(channelCode) => {
              setSelectedSupplierId(channelCode);
              setCreatingKind(null);
              setEditingChannelCode(null);
              setDeleteConfirmedFor(null);
              setDeleteErrorFor(null);
            }}
          />
        ) : isOfficialSupplier ? (
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
    </div>
  );
}

export type {
  ModelManagementChannelKind,
  ModelManagementSettingsCenterProps,
  ModelManagementSettingsMessages,
  ModelManagementEngineSelection,
} from './modelManagementSettingsTypes';
