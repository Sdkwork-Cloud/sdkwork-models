import React, { useEffect, useMemo, useState } from 'react';
import {
  AiResourceSelectorModal,
  BottomPagination,
  ConfirmDialog,
  type AiResourceSelectorOption,
} from '@sdkwork/clawroutes-pc-commons';
import { Edit, Loader2, Plus, RefreshCw, Search, Trash2, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import {
  ResourceGroupService,
  type ResourceGroupCreateInput,
  type ResourceGroupItem,
  type ResourcePageInfo,
  type ResourceGroupResourceItem,
  type ResourceGroupUpdateInput,
} from './resourceGroupService';

type GroupFormState = {
  id: string;
  groupCode: string;
  groupName: string;
  description: string;
  sortOrder: number;
  status: 'active' | 'disabled';
  memberCodes: string[];
};

const emptyForm = (): GroupFormState => ({
  id: '',
  groupCode: '',
  groupName: '',
  description: '',
  sortOrder: 100,
  status: 'active',
  memberCodes: [],
});

const emptyPageInfo = (pageSize: number): ResourcePageInfo => ({
  page: 1,
  pageSize,
  totalItems: 0,
  totalPages: 0,
  hasMore: false,
});

export function ResourceAdmin() {
  const { t } = useTranslation();
  const [groups, setGroups] = useState<ResourceGroupItem[]>([]);
  const [selectedGroupCode, setSelectedGroupCode] = useState<string | null>(null);
  const [groupResources, setGroupResources] = useState<ResourceGroupResourceItem[]>([]);
  const [allResources, setAllResources] = useState<AiResourceSelectorOption[]>([]);
  const [resourceSearch, setResourceSearch] = useState('');
  const [resourcePage, setResourcePage] = useState(1);
  const [resourcePageSize, setResourcePageSize] = useState(20);
  const [resourcePageInfo, setResourcePageInfo] = useState<ResourcePageInfo>(() => emptyPageInfo(20));
  const [loadingGroups, setLoadingGroups] = useState(true);
  const [loadingResources, setLoadingResources] = useState(false);
  const [saving, setSaving] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [form, setForm] = useState<GroupFormState>(emptyForm());
  const [resourceSelectorOpen, setResourceSelectorOpen] = useState(false);
  const [resourceAssignmentSelectorOpen, setResourceAssignmentSelectorOpen] = useState(false);
  const [resourceAssignmentDraftCodes, setResourceAssignmentDraftCodes] = useState<string[]>([]);
  const [deleteTarget, setDeleteTarget] = useState<ResourceGroupItem | null>(null);

  const selectedGroup = groups.find((group) => group.groupCode === selectedGroupCode) ?? null;
  const canManageSelectedGroupResources =
    Boolean(selectedGroup) && !selectedGroup?.dynamic && selectedGroup?.groupCode !== 'api.all';

  const allResourceOptions = allResources;
  const resourceOptionsByCode = useMemo(
    () => new Map(allResourceOptions.map((option) => [option.resourceCode, option])),
    [allResourceOptions],
  );

  const loadGroups = async () => {
    setLoadingGroups(true);
    setLoadError(null);
    try {
      const nextGroups = await ResourceGroupService.fetchResourceGroups();
      setGroups(nextGroups);
      setSelectedGroupCode(nextGroups.find(group => group.groupCode === 'api.all')?.groupCode ?? nextGroups[0]?.groupCode ?? null);
      setResourcePage(1);
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.loadGroups'));
    } finally {
      setLoadingGroups(false);
    }
  };

  const loadAllResources = async () => {
    try {
      const items = await ResourceGroupService.fetchAssignableResources();
      setAllResources(
        items.map((item) => ({
          id: item.id,
          resourceCode: item.resourceCode,
          displayName: item.displayName,
          resourceType: item.resourceType,
          vendorCode: item.vendorCode,
          modalityCode: item.modalityCode,
          apiEndpointCode: item.apiEndpointCode,
          catalogKey: item.catalogKey,
          model: item.model,
          providerNativeModel: item.providerNativeModel,
          status: item.status,
        })),
      );
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.loadAssignableResources'));
    }
  };

  const loadSelectedGroupResources = async (
    group: ResourceGroupItem | null,
    query = {
      page: resourcePage,
      pageSize: resourcePageSize,
      q: resourceSearch,
    },
  ) => {
    if (!group) {
      setGroupResources([]);
      setResourcePageInfo(emptyPageInfo(resourcePageSize));
      return;
    }
    setLoadingResources(true);
    setLoadError(null);
    try {
      const resources = await ResourceGroupService.fetchResourceGroupResourcesPage(group.groupCode, query);
      setGroupResources(resources.items);
      setResourcePageInfo(resources.pageInfo);
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.loadResources'));
      setGroupResources([]);
      setResourcePageInfo(emptyPageInfo(resourcePageSize));
    } finally {
      setLoadingResources(false);
    }
  };

  useEffect(() => {
    void loadGroups();
    void loadAllResources();
  }, []);

  useEffect(() => {
    if (!selectedGroup) {
      setGroupResources([]);
      setResourcePageInfo(emptyPageInfo(resourcePageSize));
      return;
    }
    void loadSelectedGroupResources(selectedGroup, {
      page: resourcePage,
      pageSize: resourcePageSize,
      q: resourceSearch,
    });
  }, [selectedGroup?.id, selectedGroup?.groupCode, resourcePage, resourcePageSize, resourceSearch, t]);

  const startCreate = () => {
    setForm(emptyForm());
    setFormOpen(true);
  };

  const startEdit = (group: ResourceGroupItem) => {
    void (async () => {
      setLoadError(null);
      try {
        const resources = await ResourceGroupService.fetchResourceGroupResourcesForUpdate(group.groupCode);
        setForm({
          id: group.id,
          groupCode: group.groupCode,
          groupName: group.groupName,
          description: group.description ?? '',
          sortOrder: group.sortOrder,
          status: group.status === 'disabled' ? 'disabled' : 'active',
          memberCodes: resources.map((resource) => resource.resourceCode),
        });
        setFormOpen(true);
      } catch (error) {
        setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.loadResources'));
      }
    })();
  };

  const saveGroupForm = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSaving(true);
    setLoadError(null);
    try {
      const input = {
        groupCode: form.groupCode,
        groupName: form.groupName,
        groupType: 'api_group' as const,
        selectionMode: (form.groupCode === 'api.all' ? 'all' : 'manual') as ResourceGroupUpdateInput['selectionMode'],
        description: form.description || null,
        sortOrder: form.sortOrder,
        status: form.status,
        members: form.memberCodes.map((resourceCode, index) => ({
          resourceCode,
          itemRole: 'included' as const,
          sortOrder: index + 1,
        })),
      };
      if (form.id) {
        const updated = await ResourceGroupService.updateResourceGroup(form.id, input);
        setGroups((current) => current.map((group) => (group.id === updated.id ? updated : group)));
        setSelectedGroupCode(updated.groupCode);
      } else {
        const created = await ResourceGroupService.createResourceGroup(input as ResourceGroupCreateInput);
        setGroups((current) => [...current, created]);
        setSelectedGroupCode(created.groupCode);
      }
      setFormOpen(false);
      await loadGroups();
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.saveGroup'));
    } finally {
      setSaving(false);
    }
  };

  const confirmDeleteGroup = async () => {
    if (!deleteTarget) {
      return;
    }
    setSaving(true);
    setLoadError(null);
    try {
      const deleted = await ResourceGroupService.deleteResourceGroup(deleteTarget.id);
      if (deleted) {
        setGroups((current) => current.filter((group) => group.id !== deleteTarget.id));
        setSelectedGroupCode((current) => (current === deleteTarget.groupCode ? null : current));
      }
      setDeleteTarget(null);
      await loadGroups();
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.deleteGroup'));
    } finally {
      setSaving(false);
    }
  };

  const saveResourceAssignmentDraft = async () => {
    if (!selectedGroup) {
      setResourceAssignmentSelectorOpen(false);
      return;
    }
    setSaving(true);
    setLoadError(null);
    try {
      const memberCodes = resourceAssignmentDraftCodes;
      await ResourceGroupService.updateResourceGroup(selectedGroup.id, {
        members: memberCodes.map((resourceCode, index) => ({
          resourceCode,
          itemRole: 'included' as const,
          sortOrder: index + 1,
        })),
      });
      setResourceAssignmentSelectorOpen(false);
      await loadSelectedGroupResources(selectedGroup);
      await loadGroups();
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.saveGroup'));
    } finally {
      setSaving(false);
    }
  };

  const removeSelectedGroupResource = async (resourceCode: string) => {
    if (!selectedGroup) {
      return;
    }
    setSaving(true);
    setLoadError(null);
    try {
      const resources = await ResourceGroupService.fetchResourceGroupResourcesForUpdate(selectedGroup.groupCode);
      const memberCodes = resources
        .map((resource) => resource.resourceCode)
        .filter((code) => code !== resourceCode);
      await ResourceGroupService.updateResourceGroup(selectedGroup.id, {
        members: memberCodes.map((code, index) => ({
          resourceCode: code,
          itemRole: 'included' as const,
          sortOrder: index + 1,
        })),
      });
      await loadSelectedGroupResources(selectedGroup);
      await loadGroups();
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.saveGroup'));
    } finally {
      setSaving(false);
    }
  };

  const openResourceAssignmentSelector = () => {
    if (!selectedGroup) {
      return;
    }
    void (async () => {
      setLoadingResources(true);
      setLoadError(null);
      try {
        const resources = await ResourceGroupService.fetchResourceGroupResourcesForUpdate(selectedGroup.groupCode);
        setResourceAssignmentDraftCodes(resources.map(resource => resource.resourceCode));
        setResourceAssignmentSelectorOpen(true);
      } catch (error) {
        setLoadError(error instanceof Error ? error.message : t('admin.model.resources.errors.loadResources'));
      } finally {
        setLoadingResources(false);
      }
    })();
  };

  const formSelectedResources = form.memberCodes
    .map((code) => resourceOptionsByCode.get(code))
    .filter((item): item is AiResourceSelectorOption => Boolean(item));

  return (
    <div
      data-admin-model-resource-page
      className="flex min-h-0 h-full w-full flex-col bg-slate-50 dark:bg-[#121212] rounded-xl overflow-hidden shadow-sm border border-slate-200 dark:border-white/5"
    >
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <aside data-admin-model-resource-sidebar className="flex w-72 min-w-72 flex-col border-r border-slate-200 dark:border-white/10">
          <div
            data-admin-model-resource-sidebar-header
            className="flex shrink-0 flex-col gap-3 border-b border-slate-200 px-4 py-4 dark:border-white/10"
          >
            <div className="flex items-center justify-between gap-2">
              <h2 className="text-sm font-bold text-slate-900 dark:text-white">{t('admin.model.resources.sidebarTitle')}</h2>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={startCreate}
                  className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-slate-200 text-slate-600 transition hover:bg-slate-100 dark:border-white/10 dark:text-slate-300 dark:hover:bg-white/5"
                  title={t('admin.model.resources.actions.newGroup')}
                >
                  <Plus className="h-4 w-4" />
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void loadGroups();
                    void loadAllResources();
                  }}
                  className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-slate-200 text-slate-600 transition hover:bg-slate-100 dark:border-white/10 dark:text-slate-300 dark:hover:bg-white/5"
                  title={t('admin.model.resources.actions.refresh')}
                >
                  <RefreshCw className="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
          <div data-admin-model-resource-sidebar-list className="min-h-0 flex-1 overflow-y-auto p-2">
            {loadingGroups ? (
              <div className="flex items-center justify-center py-8 text-sm text-slate-500">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                {t('admin.model.resources.loading')}
              </div>
            ) : (
              groups.map((group) => (
                <button
                  key={group.id}
                  type="button"
                  onClick={() => {
                    setSelectedGroupCode(group.groupCode);
                    setResourcePage(1);
                  }}
                  className={`mb-1 flex w-full flex-col rounded-lg px-3 py-2 text-left transition ${
                    selectedGroupCode === group.groupCode
                      ? 'bg-indigo-50 text-indigo-900 dark:bg-indigo-500/10 dark:text-indigo-100'
                      : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-white/5'
                  }`}
                >
                  <span className="text-sm font-semibold">{group.groupName}</span>
                  <span className="font-mono text-xs text-slate-500 dark:text-slate-400">{group.groupCode}</span>
                  {group.dynamic ? (
                    <span className="mt-1 inline-flex w-fit rounded bg-amber-50 px-1.5 py-0.5 text-[10px] font-semibold text-amber-700 dark:bg-amber-500/10 dark:text-amber-300">
                      {t('admin.model.resources.dynamic')}
                    </span>
                  ) : null}
                </button>
              ))
            )}
          </div>
        </aside>

        <main data-admin-model-resource-main className="flex min-h-0 flex-1 flex-col overflow-hidden">
          {!selectedGroup ? (
            <div className="flex flex-1 items-center justify-center text-sm text-slate-500">{t('admin.model.resources.noGroup')}</div>
          ) : (
            <div data-admin-model-resource-main-panel className="flex min-h-0 flex-1 flex-col overflow-hidden">
              <div className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-slate-200 px-5 py-4 dark:border-white/10">
                <div>
                  <h3 className="text-base font-bold text-slate-900 dark:text-white">{selectedGroup.groupName}</h3>
                  <p className="font-mono text-xs text-slate-500 dark:text-slate-400">{selectedGroup.groupCode}</p>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <div className="relative">
                    <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                    <input
                      value={resourceSearch}
                      onChange={(event) => {
                        setResourceSearch(event.target.value);
                        setResourcePage(1);
                      }}
                      placeholder={t('admin.model.resources.resourceSearch')}
                      className="h-10 w-64 rounded-lg border border-slate-200 bg-white pl-10 pr-3 text-sm outline-none focus:border-indigo-500 dark:border-white/10 dark:bg-[#1a1a1a] dark:text-white"
                    />
                  </div>
                  <button
                    type="button"
                    disabled={selectedGroup.dynamic || selectedGroup.groupCode === 'api.all'}
                    onClick={() => startEdit(selectedGroup)}
                    className="inline-flex items-center gap-2 rounded-lg border border-slate-200 px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-white/10 dark:text-slate-200 dark:hover:bg-white/5"
                  >
                    <Edit className="h-4 w-4" />
                    {t('admin.model.resources.actions.edit')}
                  </button>
                  <button
                    type="button"
                    disabled={selectedGroup.dynamic || selectedGroup.groupCode === 'api.all'}
                    onClick={() => setDeleteTarget(selectedGroup)}
                    className="inline-flex items-center gap-2 rounded-lg border border-red-200 px-3 py-2 text-sm font-medium text-red-700 transition hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-red-500/30 dark:text-red-300 dark:hover:bg-red-500/10"
                  >
                    <Trash2 className="h-4 w-4" />
                    {t('admin.model.resources.actions.delete')}
                  </button>
                </div>
              </div>

              <div
                data-admin-model-resource-main-resource-actions
                className="flex shrink-0 items-center justify-between gap-3 border-b border-slate-200 px-5 py-3 dark:border-white/10"
              >
                <p className="text-sm text-slate-500 dark:text-slate-400">
                  {selectedGroup.description || t('admin.model.resources.form.resourceSelectionHint')}
                </p>
                <button
                  type="button"
                  data-admin-model-resource-add-resource
                  disabled={!canManageSelectedGroupResources || loadingResources || saving}
                  onClick={openResourceAssignmentSelector}
                  className="inline-flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-indigo-700 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <Plus className="h-4 w-4" />
                  {t('admin.model.resources.actions.addResource')}
                </button>
              </div>

              <div data-admin-model-resource-table-scroll className="min-h-0 flex-1 overflow-auto">
                {loadingResources ? (
                  <div className="flex items-center justify-center py-12 text-sm text-slate-500">
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {t('admin.model.resources.loading')}
                  </div>
                ) : groupResources.length === 0 ? (
                  <div className="flex items-center justify-center py-12 text-sm text-slate-500">{t('admin.model.resources.emptyResources')}</div>
                ) : (
                  <table data-admin-model-resource-table className="w-full min-w-[860px] text-left text-sm">
                    <thead className="sticky top-0 z-10 border-b border-slate-200 bg-slate-50 text-xs font-semibold uppercase tracking-wide text-slate-500 dark:border-white/10 dark:bg-[#161616]">
                      <tr>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.resource')}</th>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.kind')}</th>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.vendor')}</th>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.modality')}</th>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.role')}</th>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.status')}</th>
                        <th className="px-5 py-3">{t('admin.model.resources.columns.actions')}</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-200 dark:divide-white/5">
                      {groupResources.map((resource) => (
                        <tr key={resource.id} className="hover:bg-slate-50 dark:hover:bg-white/[0.03]">
                          <td className="px-5 py-3">
                            <div className="font-medium text-slate-900 dark:text-white">{resource.displayName}</div>
                            <div className="font-mono text-xs text-slate-500">{resource.resourceCode}</div>
                          </td>
                          <td className="px-5 py-3">{resource.resourceType}</td>
                          <td className="px-5 py-3">{resource.vendorCode ?? '-'}</td>
                          <td className="px-5 py-3">{resource.modalityCode ?? '-'}</td>
                          <td className="px-5 py-3">{resource.memberRole}</td>
                          <td className="px-5 py-3">{resource.status}</td>
                          <td className="px-5 py-3">
                            <button
                              type="button"
                              data-admin-model-resource-row-action
                              disabled={!canManageSelectedGroupResources || loadingResources || saving}
                              onClick={() => void removeSelectedGroupResource(resource.resourceCode)}
                              className="rounded-lg px-2 py-1 text-xs font-semibold text-red-600 transition hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50 dark:text-red-300 dark:hover:bg-red-500/10"
                            >
                              {t('admin.model.resources.actions.removeResource')}
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>

              <div data-admin-model-resource-pagination className="shrink-0">
                <BottomPagination
                  page={resourcePage}
                  pageSize={resourcePageSize}
                  itemCount={groupResources.length}
                  hasNextPage={resourcePageInfo.hasMore}
                  showingLabel={t('admin.model.resources.pagination.showing')}
                  pageLabel={t('admin.model.resources.pagination.page', { page: resourcePage })}
                  pageSizeLabel={t('admin.model.resources.pagination.pageSize')}
                  disabled={loadingResources || saving}
                  onPreviousPage={() => setResourcePage((current) => Math.max(1, current - 1))}
                  onNextPage={() => setResourcePage((current) => current + 1)}
                  onPageSizeChange={(nextPageSize) => {
                    setResourcePageSize(nextPageSize);
                    setResourcePage(1);
                  }}
                />
              </div>
            </div>
          )}
        </main>
      </div>

      {loadError ? (
        <div className="shrink-0 border-t border-red-200 bg-red-50 px-4 py-2 text-sm text-red-700 dark:border-red-500/30 dark:bg-red-500/10 dark:text-red-200">
          {loadError}
        </div>
      ) : null}

      {formOpen ? (
        <div className="fixed inset-0 z-[60] flex justify-end bg-slate-950/40 backdrop-blur-sm">
          <div
            data-admin-model-resource-group-drawer
            className="flex h-full w-[80vw] max-w-[80vw] flex-col overflow-hidden border-l border-slate-200 bg-white shadow-2xl dark:border-white/10 dark:bg-[#141414]"
          >
            <div className="flex shrink-0 items-center justify-between border-b border-slate-200 px-5 py-4 dark:border-white/10">
              <h3 className="text-lg font-bold text-slate-900 dark:text-white">
                {form.id ? t('admin.model.resources.form.editTitle') : t('admin.model.resources.form.createTitle')}
              </h3>
              <button
                type="button"
                onClick={() => setFormOpen(false)}
                className="inline-flex h-9 w-9 items-center justify-center rounded-lg text-slate-400 hover:bg-slate-100 dark:hover:bg-white/10"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <form onSubmit={(event) => void saveGroupForm(event)} className="flex min-h-0 flex-1 flex-col overflow-hidden">
              <div data-admin-model-resource-group-drawer-basic className="grid shrink-0 gap-4 border-b border-slate-200 px-5 py-4 dark:border-white/10 md:grid-cols-2">
                <label className="block text-sm">
                  <span className="mb-1 block font-medium text-slate-700 dark:text-slate-200">{t('admin.model.resources.form.groupCode')}</span>
                  <input
                    value={form.groupCode}
                    disabled={Boolean(form.id) || form.groupCode === 'api.all'}
                    onChange={(event) => setForm({ ...form, groupCode: event.target.value })}
                    className="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm dark:border-white/10 dark:bg-[#1a1a1a] dark:text-white disabled:opacity-60"
                  />
                </label>
                <label className="block text-sm">
                  <span className="mb-1 block font-medium text-slate-700 dark:text-slate-200">{t('admin.model.resources.form.groupName')}</span>
                  <input
                    value={form.groupName}
                    onChange={(event) => setForm({ ...form, groupName: event.target.value })}
                    className="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm dark:border-white/10 dark:bg-[#1a1a1a] dark:text-white"
                  />
                </label>
                <label className="block text-sm">
                  <span className="mb-1 block font-medium text-slate-700 dark:text-slate-200">{t('admin.model.resources.form.sortOrder')}</span>
                  <input
                    type="number"
                    value={form.sortOrder}
                    onChange={(event) => setForm({ ...form, sortOrder: Number(event.target.value) || 0 })}
                    className="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm dark:border-white/10 dark:bg-[#1a1a1a] dark:text-white"
                  />
                </label>
                <label className="block text-sm">
                  <span className="mb-1 block font-medium text-slate-700 dark:text-slate-200">{t('admin.model.resources.form.status')}</span>
                  <select
                    value={form.status}
                    onChange={(event) => setForm({ ...form, status: event.target.value as GroupFormState['status'] })}
                    className="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm dark:border-white/10 dark:bg-[#1a1a1a] dark:text-white"
                  >
                    <option value="active">active</option>
                    <option value="disabled">disabled</option>
                  </select>
                </label>
                <label className="block text-sm md:col-span-2">
                  <span className="mb-1 block font-medium text-slate-700 dark:text-slate-200">{t('admin.model.resources.form.description')}</span>
                  <textarea
                    value={form.description}
                    onChange={(event) => setForm({ ...form, description: event.target.value })}
                    rows={3}
                    className="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm dark:border-white/10 dark:bg-[#1a1a1a] dark:text-white"
                  />
                </label>
              </div>

              <div data-admin-model-resource-group-drawer-resources className="flex min-h-0 flex-1 flex-col overflow-hidden px-5 py-4">
                <div className="mb-3 flex items-center justify-between gap-3">
                  <div className="text-sm text-slate-600 dark:text-slate-300">
                    {t('admin.model.resources.form.selectedResources', { count: form.memberCodes.length })}
                  </div>
                  <button
                    type="button"
                    disabled={form.groupCode === 'api.all'}
                    onClick={() => setResourceSelectorOpen(true)}
                    className="rounded-lg border border-slate-200 px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-white/10 dark:text-slate-200 dark:hover:bg-white/5"
                  >
                    {t('admin.model.resources.form.selectResources')}
                  </button>
                </div>
                {formSelectedResources.length === 0 ? (
                  <div className="rounded-lg border border-dashed border-slate-200 px-4 py-8 text-center text-sm text-slate-500 dark:border-white/10">
                    {t('admin.model.resources.form.emptySelectedResources')}
                  </div>
                ) : (
                  <div className="min-h-0 flex-1 overflow-auto">
                    <table data-admin-model-resource-group-form-resource-table className="w-full text-left text-sm">
                      <thead className="border-b border-slate-200 text-xs font-semibold uppercase text-slate-500 dark:border-white/10">
                        <tr>
                          <th className="px-3 py-2">{t('admin.model.resources.columns.resource')}</th>
                          <th className="px-3 py-2">{t('admin.model.resources.columns.vendor')}</th>
                          <th className="px-3 py-2">{t('admin.model.resources.columns.actions')}</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-slate-200 dark:divide-white/5">
                        {formSelectedResources.map((resource) => (
                          <tr key={resource.resourceCode}>
                            <td className="px-3 py-2">
                              <div className="font-medium">{resource.displayName}</div>
                              <div className="font-mono text-xs text-slate-500">{resource.resourceCode}</div>
                            </td>
                            <td className="px-3 py-2">{resource.vendorCode ?? '-'}</td>
                            <td className="px-3 py-2">
                              <button
                                type="button"
                                disabled={form.groupCode === 'api.all'}
                                onClick={() =>
                                  setForm({
                                    ...form,
                                    memberCodes: form.memberCodes.filter((code) => code !== resource.resourceCode),
                                  })
                                }
                                className="text-xs font-semibold text-red-600 disabled:opacity-50"
                              >
                                {t('admin.model.resources.form.removeResource')}
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>

              <div className="flex shrink-0 justify-end gap-3 border-t border-slate-200 px-5 py-4 dark:border-white/10">
                <button
                  type="button"
                  onClick={() => setFormOpen(false)}
                  className="rounded-lg border border-slate-200 px-4 py-2 text-sm font-medium text-slate-700 dark:border-white/10 dark:text-slate-200"
                >
                  {t('admin.model.resources.actions.cancel')}
                </button>
                <button
                  type="submit"
                  disabled={saving || form.groupCode === 'api.all'}
                  className="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white disabled:opacity-50"
                >
                  {saving ? t('admin.model.resources.loading') : t('admin.model.resources.actions.save')}
                </button>
              </div>
            </form>
          </div>
        </div>
      ) : null}

      {resourceSelectorOpen ? (
        <AiResourceSelectorModal
          loading={false}
          options={allResourceOptions}
          selectedCodes={form.memberCodes}
          selectionMode="multiple"
          onChange={(codes) => setForm({ ...form, memberCodes: codes })}
          onClose={() => setResourceSelectorOpen(false)}
          labels={{
            title: t('admin.model.resources.form.resourceSelectorTitle'),
            searchPlaceholder: t('admin.model.resources.resourceSearch'),
            loading: t('admin.model.resources.loading'),
            empty: t('admin.model.resources.form.emptyAssignableResources'),
            emptySearch: t('admin.model.resources.form.emptyAssignableResourceSearch'),
            selectedCount: (count) => t('admin.model.resources.form.selectedResources', { count }),
            done: t('admin.model.resources.actions.save'),
            columns: {
              resource: t('admin.model.resources.columns.resource'),
              kind: t('admin.model.resources.columns.kind'),
              vendor: t('admin.model.resources.columns.vendor'),
              status: t('admin.model.resources.columns.status'),
            },
          }}
        />
      ) : null}

      {resourceAssignmentSelectorOpen ? (
        <AiResourceSelectorModal
          loading={loadingResources}
          options={allResourceOptions}
          selectedCodes={resourceAssignmentDraftCodes}
          selectionMode="multiple"
          onChange={setResourceAssignmentDraftCodes}
          onClose={() => void saveResourceAssignmentDraft()}
          labels={{
            title: t('admin.model.resources.form.resourceSelectorTitle'),
            searchPlaceholder: t('admin.model.resources.resourceSearch'),
            loading: t('admin.model.resources.loading'),
            empty: t('admin.model.resources.form.emptyAssignableResources'),
            emptySearch: t('admin.model.resources.form.emptyAssignableResourceSearch'),
            selectedCount: (count) => t('admin.model.resources.form.selectedResources', { count }),
            done: t('admin.model.resources.actions.save'),
            columns: {
              resource: t('admin.model.resources.columns.resource'),
              kind: t('admin.model.resources.columns.kind'),
              vendor: t('admin.model.resources.columns.vendor'),
              status: t('admin.model.resources.columns.status'),
            },
          }}
        />
      ) : null}

      {deleteTarget && (
        <ConfirmDialog
          title={t('admin.model.resources.deleteDialog.title')}
          description={t('admin.model.resources.deleteDialog.description', { name: deleteTarget.groupName })}
          confirmLabel={t('admin.model.resources.actions.delete')}
          cancelLabel={t('admin.model.resources.actions.cancel')}
          isBusy={saving}
          tone="danger"
          onConfirm={() => void confirmDeleteGroup()}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  );
}
