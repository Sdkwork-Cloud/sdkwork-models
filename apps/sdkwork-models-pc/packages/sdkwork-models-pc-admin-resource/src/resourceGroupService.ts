import {
  ensureSdkworkApiSuccess,
  getModelsBackendSdkClient,
  isRecord,
  readApiRecord,
  readBoolean,
  readNullableString,
  readRequiredApiItem,
  readRequiredApiItems,
  readRequiredString,
  readString,
  requiredSafePathSegment,
  type ApiRecord,
} from '@sdkwork/clawroutes-pc-commons/runtime';
import type {
  AdminAiResourceGroupCreateRequest,
  AdminAiResourceGroupMemberInput,
  AdminAiResourceGroupUpdateRequest,
} from '@sdkwork/models-backend-sdk';

export interface ResourceGroupItem {
  id: string;
  groupCode: string;
  groupName: string;
  groupType: 'api_group';
  selectionMode: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description: string | null;
  sortOrder: number;
  status: 'active' | 'disabled' | 'inactive';
  resourceCount: number;
  dynamic: boolean;
}

export interface ResourceGroupResourceItem {
  id: string;
  resourceCode: string;
  resourceType: string;
  displayName: string;
  vendorCode: string | null;
  modalityCode: string | null;
  apiEndpointCode: string | null;
  catalogKey: string | null;
  model: string | null;
  providerNativeModel: string | null;
  status: 'active' | 'disabled' | 'inactive';
  sortOrder: number | null;
  memberRole: 'included' | 'optional' | 'fallback';
}

export interface ResourceGroupAssignableResourceItem {
  id: string;
  resourceCode: string;
  resourceType: string;
  displayName: string;
  vendorCode: string | null;
  modalityCode: string | null;
  apiEndpointCode: string | null;
  catalogKey: string | null;
  model: string | null;
  providerNativeModel: string | null;
  status: 'active' | 'disabled' | 'inactive';
}

export interface ResourcePageInfo {
  page: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
  hasMore: boolean;
}

export interface ResourceListPage<T> {
  items: T[];
  pageInfo: ResourcePageInfo;
}

export interface ResourceListQuery {
  page?: number;
  pageSize?: number;
  q?: string;
}

export interface ResourceGroupMemberInput {
  resourceCode: string;
  itemRole?: 'included' | 'optional' | 'fallback';
  sortOrder?: number;
}

export interface ResourceGroupCreateInput {
  groupCode: string;
  groupName: string;
  groupType: 'api_group';
  selectionMode: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description?: string | null;
  sortOrder?: number;
  status?: 'active' | 'disabled' | 'inactive';
  members?: ResourceGroupMemberInput[];
}

export interface ResourceGroupUpdateInput {
  groupCode?: string;
  groupName?: string;
  groupType?: 'api_group';
  selectionMode?: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description?: string | null;
  sortOrder?: number;
  status?: 'active' | 'disabled' | 'inactive';
  members?: ResourceGroupMemberInput[];
}

export class ResourceGroupService {
  static async fetchResourceGroups(): Promise<ResourceGroupItem[]> {
    const result = await getModelsBackendSdkClient().ai.aiResourceGroups.list();
    ensureSdkworkApiSuccess(result, 'Failed to fetch resource groups');
    return readRequiredApiItems(result, 'Failed to fetch resource groups').map(normalizeResourceGroupItem);
  }

  static async fetchResourceGroupResourcesPage(
    groupCode: string,
    query: ResourceListQuery = {},
  ): Promise<ResourceListPage<ResourceGroupResourceItem>> {
    const result = await getModelsBackendSdkClient().ai.aiResourceGroups.resources.list(
      requiredSafePathSegment(normalizeCatalogCode(groupCode), 'groupCode'),
      toSdkListParams(query),
    );
    ensureSdkworkApiSuccess(result, 'Failed to fetch group resources');
    const record = readApiRecord(result);
    return {
      items: readRequiredApiItems(result, 'Failed to fetch group resources').map(normalizeResourceGroupResourceItem),
      pageInfo: readPageInfo(record),
    };
  }

  static async fetchResourceGroupResources(groupCode: string): Promise<ResourceGroupResourceItem[]> {
    const page = await ResourceGroupService.fetchResourceGroupResourcesPage(groupCode, {
      page: 1,
      pageSize: 200,
    });
    return page.items;
  }

  static async fetchResourceGroupResourcesForUpdate(groupCode: string): Promise<ResourceGroupResourceItem[]> {
    const items: ResourceGroupResourceItem[] = [];
    let page = 1;
    let hasMore = true;
    while (hasMore) {
      const result = await ResourceGroupService.fetchResourceGroupResourcesPage(groupCode, {
        page,
        pageSize: 200,
      });
      items.push(...result.items);
      hasMore = result.pageInfo.hasMore;
      page += 1;
    }
    return items;
  }

  static async fetchAssignableResources(): Promise<ResourceGroupAssignableResourceItem[]> {
    const result = await getModelsBackendSdkClient().ai.aiResources.list({
      page: '1',
      pageSize: '200',
    });
    ensureSdkworkApiSuccess(result, 'Failed to fetch assignable resources');
    return readRequiredApiItems(result, 'Failed to fetch assignable resources').map(normalizeAssignableResourceItem);
  }

  static async createResourceGroup(input: ResourceGroupCreateInput): Promise<ResourceGroupItem> {
    if (input.groupType !== 'api_group') {
      throw new Error(`Unsupported AI resource group type: ${input.groupType}`);
    }
    const result = await getModelsBackendSdkClient().ai.aiResourceGroups.create(toCreateRequest(input));
    ensureSdkworkApiSuccess(result, 'Failed to create resource group');
    return normalizeResourceGroupItem(readRequiredApiItem(result, 'Failed to create resource group'));
  }

  static async updateResourceGroup(groupId: string, input: ResourceGroupUpdateInput): Promise<ResourceGroupItem> {
    if (input.groupType !== undefined && input.groupType !== 'api_group') {
      throw new Error(`Unsupported AI resource group type: ${input.groupType}`);
    }
    const result = await getModelsBackendSdkClient().ai.aiResourceGroups.update(
      requiredSafePathSegment(groupId, 'groupId'),
      toUpdateRequest(input),
    );
    ensureSdkworkApiSuccess(result, 'Failed to update resource group');
    return normalizeResourceGroupItem(readRequiredApiItem(result, 'Failed to update resource group'));
  }

  static async deleteResourceGroup(groupId: string): Promise<boolean> {
    const result = await getModelsBackendSdkClient().ai.aiResourceGroups.delete(
      requiredSafePathSegment(groupId, 'groupId'),
    );
    ensureSdkworkApiSuccess(result, 'Failed to delete resource group');
    return readBoolean(readApiRecord(result), 'deleted', false);
  }
}

function toSdkListParams(query: ResourceListQuery): { page?: string; pageSize?: string; q?: string } {
  return {
    page: query.page === undefined ? undefined : String(query.page),
    pageSize: query.pageSize === undefined ? undefined : String(query.pageSize),
    q: query.q?.trim() ? query.q.trim() : undefined,
  };
}

function normalizeCatalogCode(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) {
    throw new Error('Code is required');
  }
  return trimmed.toLowerCase();
}

function normalizeText(value: string, fieldName: string): string {
  const trimmed = value.trim();
  if (!trimmed) {
    throw new Error(`${fieldName} is required`);
  }
  return trimmed;
}

function toCreateRequest(input: ResourceGroupCreateInput): AdminAiResourceGroupCreateRequest {
  return {
    groupCode: normalizeCatalogCode(input.groupCode),
    groupName: normalizeText(input.groupName, 'groupName'),
    groupType: 'api_group',
    selectionMode: input.selectionMode,
    description: input.description?.trim() ? input.description.trim() : null,
    sortOrder: input.sortOrder === undefined ? undefined : String(input.sortOrder),
    status: input.status ?? 'active',
    members: (input.members ?? []).map(toMemberInput),
  };
}

function toUpdateRequest(input: ResourceGroupUpdateInput): AdminAiResourceGroupUpdateRequest {
  const body: AdminAiResourceGroupUpdateRequest = {};
  if (input.groupCode !== undefined) {
    body.groupCode = normalizeCatalogCode(input.groupCode);
  }
  if (input.groupName !== undefined) {
    body.groupName = normalizeText(input.groupName, 'groupName');
  }
  if (input.groupType !== undefined) {
    body.groupType = input.groupType;
  }
  if (input.selectionMode !== undefined) {
    body.selectionMode = input.selectionMode;
  }
  if (input.description !== undefined) {
    body.description = input.description?.trim() ? input.description.trim() : null;
  }
  if (input.sortOrder !== undefined) {
    body.sortOrder = String(input.sortOrder);
  }
  if (input.status !== undefined) {
    body.status = input.status;
  }
  if (input.members !== undefined) {
    body.members = input.members.map(toMemberInput);
  }
  return body;
}

function toMemberInput(member: ResourceGroupMemberInput): AdminAiResourceGroupMemberInput {
  return {
    resourceCode: normalizeCatalogCode(member.resourceCode),
    itemRole: member.itemRole ?? 'included',
    sortOrder: member.sortOrder === undefined ? undefined : String(member.sortOrder),
  };
}

function readRequiredRecord(value: unknown, message: string): ApiRecord {
  if (!isRecord(value)) {
    throw new Error(message);
  }
  return value;
}

function readPageInfo(item: ApiRecord): ResourcePageInfo {
  const pageInfo = readRequiredRecord(item.pageInfo, 'Page info must be an object');
  return {
    page: readPositiveInteger(pageInfo, 'page', 1),
    pageSize: readPositiveInteger(pageInfo, 'pageSize', 20),
    totalItems: readNonNegativeInteger(pageInfo, 'totalItems', 0),
    totalPages: readNonNegativeInteger(pageInfo, 'totalPages', 0),
    hasMore: readBoolean(pageInfo, 'hasMore', false),
  };
}

function normalizeResourceGroupItem(value: unknown): ResourceGroupItem {
  const item = readRequiredRecord(value, 'Resource group item must be an object');
  const groupType = readGroupType(item);
  const selectionMode = readSelectionMode(item);
  return {
    id: readRequiredString(item, 'id', 'Resource group id is required'),
    groupCode: readRequiredString(item, 'groupCode', 'Resource group code is required'),
    groupName: readRequiredString(item, 'groupName', 'Resource group name is required'),
    groupType,
    selectionMode,
    description: readNullableString(item, 'description'),
    sortOrder: readNonNegativeInteger(item, 'sortOrder', 100),
    status: readGroupStatus(item),
    resourceCount: readResourceCount(item),
    dynamic: readDynamic(item, selectionMode),
  };
}

function normalizeResourceGroupResourceItem(value: unknown): ResourceGroupResourceItem {
  const item = readRequiredRecord(value, 'Resource group resource item must be an object');
  return {
    id: readRequiredString(item, 'id', 'Resource id is required'),
    resourceCode: readRequiredString(item, 'resourceCode', 'Resource code is required'),
    resourceType: readRequiredString(item, 'resourceType', 'Resource type is required'),
    displayName: readRequiredString(item, 'displayName', 'Resource display name is required'),
    vendorCode: readNullableString(item, 'vendorCode'),
    modalityCode: readNullableString(item, 'modalityCode'),
    apiEndpointCode: readNullableString(item, 'apiEndpointCode'),
    catalogKey: readNullableString(item, 'catalogKey'),
    model: readNullableString(item, 'model'),
    providerNativeModel: readNullableString(item, 'providerNativeModel'),
    status: readResourceStatus(item),
    sortOrder: readOptionalNonNegativeInteger(item, 'sortOrder'),
    memberRole: readMemberRole(item),
  };
}

function normalizeAssignableResourceItem(value: unknown): ResourceGroupAssignableResourceItem {
  const item = readRequiredRecord(value, 'Assignable resource item must be an object');
  return {
    id: readRequiredString(item, 'id', 'Resource id is required'),
    resourceCode: readRequiredString(item, 'resourceCode', 'Resource code is required'),
    resourceType: readRequiredString(item, 'resourceType', 'Resource type is required'),
    displayName: readRequiredString(item, 'displayName', 'Resource display name is required'),
    vendorCode: readNullableString(item, 'vendorCode'),
    modalityCode: readNullableString(item, 'modalityCode'),
    apiEndpointCode: readNullableString(item, 'apiEndpointCode'),
    catalogKey: readNullableString(item, 'catalogKey'),
    model: readNullableString(item, 'model'),
    providerNativeModel: readNullableString(item, 'providerNativeModel'),
    status: readResourceStatus(item),
  };
}

function readGroupType(item: ApiRecord): 'api_group' {
  const value = readRequiredString(item, 'groupType', 'Resource group type is required');
  if (value !== 'api_group') {
    throw new Error(`Unsupported AI resource group type: ${value}`);
  }
  return value;
}

function readSelectionMode(item: ApiRecord): ResourceGroupItem['selectionMode'] {
  const value = readRequiredString(item, 'selectionMode', 'Resource group selection mode is required');
  if (value === 'manual' || value === 'all' || value === 'any' || value === 'dynamic_all_api') {
    return value;
  }
  throw new Error(`Unsupported AI resource group selection mode: ${value}`);
}

function readGroupStatus(item: ApiRecord): ResourceGroupItem['status'] {
  const value = readRequiredString(item, 'status', 'Resource group status is required');
  if (value === 'active' || value === 'disabled' || value === 'inactive') {
    return value;
  }
  throw new Error(`Unsupported AI resource group status: ${value}`);
}

function readResourceStatus(item: ApiRecord): ResourceGroupResourceItem['status'] {
  const value = readRequiredString(item, 'status', 'Resource status is required');
  if (value === 'active' || value === 'disabled' || value === 'inactive') {
    return value;
  }
  throw new Error(`Unsupported AI resource status: ${value}`);
}

function readMemberRole(item: ApiRecord): ResourceGroupResourceItem['memberRole'] {
  const value = readString(item, 'memberRole') ?? readString(item, 'itemRole');
  if (value === 'included' || value === 'optional' || value === 'fallback') {
    return value;
  }
  throw new Error('Resource member role is required');
}

function readDynamic(item: ApiRecord, selectionMode: ResourceGroupItem['selectionMode']): boolean {
  if (typeof item.dynamic === 'boolean') {
    return item.dynamic;
  }
  return selectionMode === 'dynamic_all_api';
}

function readResourceCount(item: ApiRecord): number {
  const value = item.resourceCount;
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return 0;
}

function readNonNegativeInteger(item: ApiRecord, key: string, fallback: number): number {
  const value = item[key];
  if (typeof value === 'number' && Number.isSafeInteger(value) && value >= 0) {
    return value;
  }
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value);
    if (Number.isSafeInteger(parsed) && parsed >= 0) {
      return parsed;
    }
  }
  return fallback;
}

function readPositiveInteger(item: ApiRecord, key: string, fallback: number): number {
  const value = readNonNegativeInteger(item, key, fallback);
  return value > 0 ? value : fallback;
}

function readOptionalNonNegativeInteger(item: ApiRecord, key: string): number | null {
  if (!(key in item)) {
    return null;
  }
  const value = item[key];
  if (value === null || value === undefined || value === '') {
    return null;
  }
  if (typeof value === 'number' && Number.isSafeInteger(value) && value >= 0) {
    return value;
  }
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value);
    if (Number.isSafeInteger(parsed) && parsed >= 0) {
      return parsed;
    }
  }
  throw new Error(`${key} must be a non-negative integer`);
}
