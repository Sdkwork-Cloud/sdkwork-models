import {
  ensureSdkworkApiSuccess,
  isRecord,
  readApiRecord,
  readBoolean,
  readNullableString,
  readRequiredNonNegativeInt64String,
  readRequiredApiItem,
  readRequiredApiItems,
  readRequiredString,
  readString,
  type ApiRecord,
} from '@sdkwork/cloudroutes-pc-commons/api-result';
import { getModelsBackendSdkClient } from '@sdkwork/cloudroutes-pc-commons/sdk-clients';
import { requiredSafePathSegment } from '@sdkwork/cloudroutes-pc-commons/sdk-request-boundary';
import type {
  AdminAiResourceGroupCreateRequest,
  AdminAiResourceGroupMemberInput,
  AdminAiResourceGroupMemberUpdateRequest,
  AdminAiResourceGroupUpdateRequest,
} from '@sdkwork/models-backend-sdk';

export interface ResourceGroupItem {
  id: string;
  groupCode: string;
  groupName: string;
  groupType: 'api_group';
  selectionMode: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description: string | null;
  sortOrder: string;
  status: 'active' | 'disabled' | 'inactive';
  resourceCount: string;
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
  sortOrder: string | null;
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
  mode: 'offset';
  page: number;
  pageSize: number;
  totalItems: string;
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
  resourceType?: 'vendor' | 'modality' | 'api_endpoint' | 'model_api' | 'bundle';
}

export interface ResourceGroupMemberInput {
  resourceCode: string;
  itemRole?: 'included' | 'optional' | 'fallback';
  sortOrder?: string;
}

export interface ResourceGroupCreateInput {
  groupCode: string;
  groupName: string;
  groupType: 'api_group';
  selectionMode: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description?: string | null;
  sortOrder?: string;
  status?: 'active' | 'disabled' | 'inactive';
  members?: ResourceGroupMemberInput[];
}

export interface ResourceGroupUpdateInput {
  groupCode?: string;
  groupName?: string;
  groupType?: 'api_group';
  selectionMode?: 'manual' | 'all' | 'any' | 'dynamic_all_api';
  description?: string | null;
  sortOrder?: string;
  status?: 'active' | 'disabled' | 'inactive';
  members?: ResourceGroupMemberInput[];
}

export class ResourceGroupService {
  static async fetchResourceGroupsPage(
    query: ResourceListQuery = {},
  ): Promise<ResourceListPage<ResourceGroupItem>> {
    const result = await getModelsBackendSdkClient().ai.resourceGroups.list(toSdkListParams(query));
    ensureSdkworkApiSuccess(result, 'Failed to fetch resource groups');
    const record = readApiRecord(result);
    return {
      items: readRequiredApiItems(result, 'Failed to fetch resource groups').map(normalizeResourceGroupItem),
      pageInfo: readPageInfo(record),
    };
  }

  static async fetchResourceGroupResourcesPage(
    groupCode: string,
    query: ResourceListQuery = {},
  ): Promise<ResourceListPage<ResourceGroupResourceItem>> {
    const result = await getModelsBackendSdkClient().ai.resourceGroups.resources.list(
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

  static async fetchAssignableResourcesPage(
    query: ResourceListQuery = {},
  ): Promise<ResourceListPage<ResourceGroupAssignableResourceItem>> {
    const result = await getModelsBackendSdkClient().ai.resources.list(toSdkListParams(query));
    ensureSdkworkApiSuccess(result, 'Failed to fetch assignable resources');
    const record = readApiRecord(result);
    return {
      items: readRequiredApiItems(result, 'Failed to fetch assignable resources').map(normalizeAssignableResourceItem),
      pageInfo: readPageInfo(record),
    };
  }

  static async upsertResourceGroupMember(
    groupId: string,
    member: ResourceGroupMemberInput,
  ): Promise<ResourceGroupResourceItem> {
    const body: AdminAiResourceGroupMemberUpdateRequest = {
      itemRole: member.itemRole ?? 'included',
      sortOrder: member.sortOrder,
    };
    const result = await getModelsBackendSdkClient().ai.resourceGroups.resources.update(
      requiredSafePathSegment(groupId, 'groupId'),
      requiredSafePathSegment(normalizeCatalogCode(member.resourceCode), 'resourceCode'),
      body,
    );
    ensureSdkworkApiSuccess(result, 'Failed to assign resource group member');
    return normalizeResourceGroupResourceItem(result);
  }

  static async deleteResourceGroupMember(groupId: string, resourceCode: string): Promise<void> {
    await getModelsBackendSdkClient().ai.resourceGroups.resources.delete(
      requiredSafePathSegment(groupId, 'groupId'),
      requiredSafePathSegment(normalizeCatalogCode(resourceCode), 'resourceCode'),
    );
  }

  static async createResourceGroup(input: ResourceGroupCreateInput): Promise<ResourceGroupItem> {
    if (input.groupType !== 'api_group') {
      throw new Error(`Unsupported AI resource group type: ${input.groupType}`);
    }
    const result = await getModelsBackendSdkClient().ai.resourceGroups.create(toCreateRequest(input));
    ensureSdkworkApiSuccess(result, 'Failed to create resource group');
    return normalizeResourceGroupItem(readRequiredApiItem(result, 'Failed to create resource group'));
  }

  static async updateResourceGroup(groupId: string, input: ResourceGroupUpdateInput): Promise<ResourceGroupItem> {
    if (input.groupType !== undefined && input.groupType !== 'api_group') {
      throw new Error(`Unsupported AI resource group type: ${input.groupType}`);
    }
    const result = await getModelsBackendSdkClient().ai.resourceGroups.update(
      requiredSafePathSegment(groupId, 'groupId'),
      toUpdateRequest(input),
    );
    ensureSdkworkApiSuccess(result, 'Failed to update resource group');
    return normalizeResourceGroupItem(readRequiredApiItem(result, 'Failed to update resource group'));
  }

  static async deleteResourceGroup(groupId: string): Promise<void> {
    await getModelsBackendSdkClient().ai.resourceGroups.delete(
      requiredSafePathSegment(groupId, 'groupId'),
    );
  }
}

function toSdkListParams(query: ResourceListQuery): {
  page?: number;
  pageSize?: number;
  q?: string;
  resourceType?: ResourceListQuery['resourceType'];
} {
  return {
    page: query.page,
    pageSize: query.pageSize,
    q: query.q?.trim() ? query.q.trim() : undefined,
    resourceType: query.resourceType,
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
  const mode = readRequiredString(pageInfo, 'mode', 'Page info mode is required');
  if (mode !== 'offset') {
    throw new Error(`Unsupported resource pagination mode: ${mode}`);
  }
  return {
    mode,
    page: readPositiveInteger(pageInfo, 'page', 1),
    pageSize: readPositiveInteger(pageInfo, 'pageSize', 20),
    totalItems: readRequiredNonNegativeInt64String(
      pageInfo,
      'totalItems',
      'Page info totalItems must be a non-negative int64 string',
    ),
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
    sortOrder: readOptionalNonNegativeInt64String(item, 'sortOrder') ?? '100',
    status: readGroupStatus(item),
    resourceCount: readRequiredNonNegativeInt64String(
      item,
      'resourceCount',
      'Resource count must be a non-negative int64 string',
    ),
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
    sortOrder: readOptionalNonNegativeInt64String(item, 'sortOrder'),
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

function readOptionalNonNegativeInt64String(item: ApiRecord, key: string): string | null {
  if (!(key in item)) {
    return null;
  }
  const value = item[key];
  if (value === null || value === undefined || value === '') {
    return null;
  }
  if (typeof value === 'string' && /^(0|[1-9]\d*)$/u.test(value.trim())) {
    return value.trim();
  }
  throw new Error(`${key} must be a non-negative int64 string`);
}
