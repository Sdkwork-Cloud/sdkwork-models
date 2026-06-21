import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AdminAiModelCreateRequest, AdminAiModelUpdateRequest, AdminAiResourceCreateRequest, AdminAiResourceGroupCreateRequest, AdminAiResourceGroupUpdateRequest, AdminAiResourceUpdateRequest, AdminModelCatalogSyncRequest, AdminModelMappingCreateRequest, AdminModelMappingResolveRequest, AdminModelMappingUpdateRequest, AdminModelVendorCreateRequest, AiResourceGroupsCreateResult, AiResourceGroupsDeleteResult, AiResourceGroupsListResult, AiResourceGroupsResourcesListResult, AiResourceGroupsUpdateResult, AiResourcesCreateResult, AiResourcesListResult, AiResourcesUpdateResult, ModelMappingsCreateResult, ModelMappingsDeleteResult, ModelMappingsListResult, ModelMappingsResolveCreateResult, ModelMappingsUpdateResult, ModelRankingRefreshTriggerRequest, ModelRankingsJobsListResult, ModelRankingsListResult, ModelRankingsRefreshResult, ModelRankingsStatusRetrieveResult, ModelsCreateResult, ModelsDeleteResult, ModelsListResult, ModelsRefreshResult, ModelsUpdateResult, ModelVendorsCreateResult, ModelVendorsListResult } from '../types';


export class AiAiResourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List ai resources */
  async list(): Promise<AiResourcesListResult> {
    return this.client.get<AiResourcesListResult>(backendApiPath(`/ai/resources`));
  }

/** Create ai resource */
  async create(body: AdminAiResourceCreateRequest): Promise<AiResourcesCreateResult> {
    return this.client.post<AiResourcesCreateResult>(backendApiPath(`/ai/resources`), body, undefined, undefined, 'application/json');
  }

/** Update ai resource */
  async update(resourceId: string, body: AdminAiResourceUpdateRequest): Promise<AiResourcesUpdateResult> {
    return this.client.put<AiResourcesUpdateResult>(backendApiPath(`/ai/resources/${serializePathParameter(resourceId, { name: 'resourceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class AiAiResourceGroupsResourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List resource group resources */
  async list(groupIdOrCode: string): Promise<AiResourceGroupsResourcesListResult> {
    return this.client.get<AiResourceGroupsResourcesListResult>(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupIdOrCode, { name: 'groupIdOrCode', style: 'simple', explode: false })}/resources`));
  }
}

export class AiAiResourceGroupsApi {
  private client: HttpClient;
  public readonly resources: AiAiResourceGroupsResourcesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.resources = new AiAiResourceGroupsResourcesApi(client);
  }


/** List resource groups */
  async list(): Promise<AiResourceGroupsListResult> {
    return this.client.get<AiResourceGroupsListResult>(backendApiPath(`/ai/resource_groups`));
  }

/** Create resource group */
  async create(body: AdminAiResourceGroupCreateRequest): Promise<AiResourceGroupsCreateResult> {
    return this.client.post<AiResourceGroupsCreateResult>(backendApiPath(`/ai/resource_groups`), body, undefined, undefined, 'application/json');
  }

/** Delete resource group */
  async delete(groupId: string): Promise<AiResourceGroupsDeleteResult> {
    return this.client.delete<AiResourceGroupsDeleteResult>(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupId, { name: 'groupId', style: 'simple', explode: false })}`));
  }

/** Update resource group */
  async update(groupId: string, body: AdminAiResourceGroupUpdateRequest): Promise<AiResourceGroupsUpdateResult> {
    return this.client.patch<AiResourceGroupsUpdateResult>(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupId, { name: 'groupId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class AiModelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List models */
  async list(): Promise<ModelsListResult> {
    return this.client.get<ModelsListResult>(backendApiPath(`/ai/models`));
  }

/** Create model */
  async create(body: AdminAiModelCreateRequest): Promise<ModelsCreateResult> {
    return this.client.post<ModelsCreateResult>(backendApiPath(`/ai/models`), body, undefined, undefined, 'application/json');
  }

/** Sync vendors and models */
  async refresh(body: AdminModelCatalogSyncRequest): Promise<ModelsRefreshResult> {
    return this.client.post<ModelsRefreshResult>(backendApiPath(`/ai/models/refresh`), body, undefined, undefined, 'application/json');
  }

/** Delete model */
  async delete(modelId: string): Promise<ModelsDeleteResult> {
    return this.client.delete<ModelsDeleteResult>(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}`));
  }

/** Update model */
  async update(modelId: string, body: AdminAiModelUpdateRequest): Promise<ModelsUpdateResult> {
    return this.client.patch<ModelsUpdateResult>(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class AiModelVendorsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List vendors */
  async list(): Promise<ModelVendorsListResult> {
    return this.client.get<ModelVendorsListResult>(backendApiPath(`/ai/model_vendors`));
  }

/** Create vendor */
  async create(body: AdminModelVendorCreateRequest): Promise<ModelVendorsCreateResult> {
    return this.client.post<ModelVendorsCreateResult>(backendApiPath(`/ai/model_vendors`), body, undefined, undefined, 'application/json');
  }
}

export interface AiModelRankingsStatusRetrieveParams {
  rankScope?: string;
}

export class AiModelRankingsStatusApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model ranking refresh status */
  async retrieve(params?: AiModelRankingsStatusRetrieveParams): Promise<ModelRankingsStatusRetrieveResult> {
    const query = buildQueryString([
      { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsStatusRetrieveResult>(appendQueryString(backendApiPath(`/ai/model_rankings/status`), query));
  }
}

export interface AiModelRankingsJobsListParams {
  rankScope?: string;
  limit?: string;
}

export class AiModelRankingsJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model ranking refresh jobs */
  async list(params?: AiModelRankingsJobsListParams): Promise<ModelRankingsJobsListResult> {
    const query = buildQueryString([
      { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
      { name: 'limit', value: params?.limit, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsJobsListResult>(appendQueryString(backendApiPath(`/ai/model_rankings/jobs`), query));
  }
}

export interface AiModelRankingsListParams {
  rankScope?: string;
  vendorCode?: string;
  modality?: string;
  q?: string;
  limit?: string;
}

export class AiModelRankingsApi {
  private client: HttpClient;
  public readonly jobs: AiModelRankingsJobsApi;
  public readonly status: AiModelRankingsStatusApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.jobs = new AiModelRankingsJobsApi(client);
    this.status = new AiModelRankingsStatusApi(client);
  }


/** List model rankings */
  async list(params?: AiModelRankingsListParams): Promise<ModelRankingsListResult> {
    const query = buildQueryString([
      { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'modality', value: params?.modality, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'limit', value: params?.limit, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsListResult>(appendQueryString(backendApiPath(`/ai/model_rankings`), query));
  }

/** Trigger model ranking refresh */
  async refresh(body: ModelRankingRefreshTriggerRequest): Promise<ModelRankingsRefreshResult> {
    return this.client.post<ModelRankingsRefreshResult>(backendApiPath(`/ai/model_rankings/refresh`), body, undefined, undefined, 'application/json');
  }
}

export class AiModelMappingsResolveApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Resolve model mapping */
  async create(body: AdminModelMappingResolveRequest): Promise<ModelMappingsResolveCreateResult> {
    return this.client.post<ModelMappingsResolveCreateResult>(backendApiPath(`/ai/model_mappings/resolve`), body, undefined, undefined, 'application/json');
  }
}

export interface AiModelMappingsListParams {
  bindingType?: string;
  vendorCode?: string;
  channelId?: string;
  channelCode?: string;
  q?: string;
}

export class AiModelMappingsApi {
  private client: HttpClient;
  public readonly resolve: AiModelMappingsResolveApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.resolve = new AiModelMappingsResolveApi(client);
  }


/** List model mappings */
  async list(params?: AiModelMappingsListParams): Promise<ModelMappingsListResult> {
    const query = buildQueryString([
      { name: 'binding_type', value: params?.bindingType, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'channel_id', value: params?.channelId, style: 'form', explode: true, allowReserved: false },
      { name: 'channel_code', value: params?.channelCode, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelMappingsListResult>(appendQueryString(backendApiPath(`/ai/model_mappings`), query));
  }

/** Create model mapping */
  async create(body: AdminModelMappingCreateRequest): Promise<ModelMappingsCreateResult> {
    return this.client.post<ModelMappingsCreateResult>(backendApiPath(`/ai/model_mappings`), body, undefined, undefined, 'application/json');
  }

/** Delete model mapping */
  async delete(mappingId: string): Promise<ModelMappingsDeleteResult> {
    return this.client.delete<ModelMappingsDeleteResult>(backendApiPath(`/ai/model_mappings/${serializePathParameter(mappingId, { name: 'mappingId', style: 'simple', explode: false })}`));
  }

/** Update model mapping */
  async update(mappingId: string, body: AdminModelMappingUpdateRequest): Promise<ModelMappingsUpdateResult> {
    return this.client.patch<ModelMappingsUpdateResult>(backendApiPath(`/ai/model_mappings/${serializePathParameter(mappingId, { name: 'mappingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class AiApi {
  private client: HttpClient;
  public readonly modelMappings: AiModelMappingsApi;
  public readonly modelRankings: AiModelRankingsApi;
  public readonly modelVendors: AiModelVendorsApi;
  public readonly models: AiModelsApi;
  public readonly aiResourceGroups: AiAiResourceGroupsApi;
  public readonly aiResources: AiAiResourcesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.modelMappings = new AiModelMappingsApi(client);
    this.modelRankings = new AiModelRankingsApi(client);
    this.modelVendors = new AiModelVendorsApi(client);
    this.models = new AiModelsApi(client);
    this.aiResourceGroups = new AiAiResourceGroupsApi(client);
    this.aiResources = new AiAiResourcesApi(client);
  }

}

export function createAiApi(client: HttpClient): AiApi {
  return new AiApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
