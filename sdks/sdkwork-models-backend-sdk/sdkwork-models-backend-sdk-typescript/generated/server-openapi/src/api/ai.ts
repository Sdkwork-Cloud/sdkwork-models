import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AiResourceGroupResourcesPage, AiResourcesPage, ModelCatalogPage, ModelRankingRefreshJobHistoryPage, ModelRankingsPage, NoData } from '../types';


export interface AiModelVideoProfilesListParams {
  vendorCode?: string;
}

export class AiModelVideoProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model video generation profiles */
  async list(modelId: string, params?: AiModelVideoProfilesListParams): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsPage>(appendQueryString(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}/video_profiles`), query));
  }
}

export interface AiVideoProfilesListParams {
  vendorCode?: string;
  regionCode?: string;
  catalogKey?: string;
  modelId?: string;
  generationMode?: string;
  durationTierCode?: string;
  resolution?: string;
}

export class AiVideoProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List video generation profiles */
  async list(params?: AiVideoProfilesListParams): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'region_code', value: params?.regionCode, style: 'form', explode: true, allowReserved: false },
      { name: 'catalog_key', value: params?.catalogKey, style: 'form', explode: true, allowReserved: false },
      { name: 'model_id', value: params?.modelId, style: 'form', explode: true, allowReserved: false },
      { name: 'generation_mode', value: params?.generationMode, style: 'form', explode: true, allowReserved: false },
      { name: 'duration_tier_code', value: params?.durationTierCode, style: 'form', explode: true, allowReserved: false },
      { name: 'resolution', value: params?.resolution, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsPage>(appendQueryString(backendApiPath(`/ai/video_profiles`), query));
  }
}

export interface AiModelVoicesListParams {
  vendorCode?: string;
}

export class AiModelVoicesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model TTS voices */
  async list(modelId: string, params?: AiModelVoicesListParams): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsPage>(appendQueryString(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}/voices`), query));
  }
}

export interface AiVoicesListParams {
  vendorCode?: string;
  regionCode?: string;
  locale?: string;
  catalogKey?: string;
  modelId?: string;
  q?: string;
}

export class AiVoicesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List TTS voices */
  async list(params?: AiVoicesListParams): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'region_code', value: params?.regionCode, style: 'form', explode: true, allowReserved: false },
      { name: 'locale', value: params?.locale, style: 'form', explode: true, allowReserved: false },
      { name: 'catalog_key', value: params?.catalogKey, style: 'form', explode: true, allowReserved: false },
      { name: 'model_id', value: params?.modelId, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsPage>(appendQueryString(backendApiPath(`/ai/voices`), query));
  }
}

export interface AiAiResourcesListParams {
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAiResourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List */
  async list(params?: AiAiResourcesListParams): Promise<AiResourcesPage> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AiResourcesPage>(appendQueryString(backendApiPath(`/ai/resources`), query));
  }

/** Create */
  async create(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/resources`));
  }

/** Update */
  async update(resourceId: string): Promise<NoData> {
    return this.client.put<NoData>(backendApiPath(`/ai/resources/${serializePathParameter(resourceId, { name: 'resourceId', style: 'simple', explode: false })}`));
  }
}

export interface AiAiResourceGroupsResourcesListParams {
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAiResourceGroupsResourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List */
  async list(groupIdOrCode: string, params?: AiAiResourceGroupsResourcesListParams): Promise<AiResourceGroupResourcesPage> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AiResourceGroupResourcesPage>(appendQueryString(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupIdOrCode, { name: 'groupIdOrCode', style: 'simple', explode: false })}/resources`), query));
  }
}

export class AiAiResourceGroupsApi {
  private client: HttpClient;
  public readonly resources: AiAiResourceGroupsResourcesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.resources = new AiAiResourceGroupsResourcesApi(client);
  }


/** List */
  async list(): Promise<NoData> {
    return this.client.get<NoData>(backendApiPath(`/ai/resource_groups`));
  }

/** Create */
  async create(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/resource_groups`));
  }

/** Delete */
  async delete(groupId: string): Promise<NoData> {
    return this.client.delete<NoData>(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupId, { name: 'groupId', style: 'simple', explode: false })}`));
  }

/** Update */
  async update(groupId: string): Promise<NoData> {
    return this.client.patch<NoData>(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupId, { name: 'groupId', style: 'simple', explode: false })}`));
  }
}

export interface AiModelsListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  billingMeter?: string;
  vendorCodes?: string[];
  modalities?: string[];
  capabilities?: string[];
  categories?: string[];
  groups?: string[];
  modelTypes?: string;
}

export class AiModelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List */
  async list(params?: AiModelsListParams): Promise<ModelCatalogPage> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'billing_meter', value: params?.billingMeter, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_codes', value: params?.vendorCodes, style: 'form', explode: false, allowReserved: false },
      { name: 'modalities', value: params?.modalities, style: 'form', explode: false, allowReserved: false },
      { name: 'capabilities', value: params?.capabilities, style: 'form', explode: false, allowReserved: false },
      { name: 'categories', value: params?.categories, style: 'form', explode: false, allowReserved: false },
      { name: 'groups', value: params?.groups, style: 'form', explode: false, allowReserved: false },
      { name: 'model_types', value: params?.modelTypes, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelCatalogPage>(appendQueryString(backendApiPath(`/ai/models`), query));
  }

/** Create */
  async create(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/models`));
  }

/** Refresh */
  async refresh(): Promise<unknown> {
    return this.client.post<unknown>(backendApiPath(`/ai/models/refresh`));
  }

/** Delete */
  async delete(modelId: string): Promise<NoData> {
    return this.client.delete<NoData>(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}`));
  }

/** Update */
  async update(modelId: string): Promise<NoData> {
    return this.client.patch<NoData>(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}`));
  }
}

export class AiModelVendorsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List */
  async list(): Promise<NoData> {
    return this.client.get<NoData>(backendApiPath(`/ai/model_vendors`));
  }

/** Create */
  async create(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/model_vendors`));
  }
}

export interface AiModelRankingsStatusRetrieveParams {
  page?: number;
  pageSize?: number;
  q?: string;
  billingMeter?: string;
  vendorCodes?: string[];
  modalities?: string[];
  capabilities?: string[];
  categories?: string[];
  groups?: string[];
}

export class AiModelRankingsStatusApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve */
  async retrieve(params?: AiModelRankingsStatusRetrieveParams): Promise<ModelCatalogPage> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'billing_meter', value: params?.billingMeter, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_codes', value: params?.vendorCodes, style: 'form', explode: false, allowReserved: false },
      { name: 'modalities', value: params?.modalities, style: 'form', explode: false, allowReserved: false },
      { name: 'capabilities', value: params?.capabilities, style: 'form', explode: false, allowReserved: false },
      { name: 'categories', value: params?.categories, style: 'form', explode: false, allowReserved: false },
      { name: 'groups', value: params?.groups, style: 'form', explode: false, allowReserved: false },
    ]);
    return this.client.get<ModelCatalogPage>(appendQueryString(backendApiPath(`/ai/model_rankings/status`), query));
  }
}

export interface AiModelRankingsJobsListParams {
  rankScope?: string;
  pageSize?: number;
}

export class AiModelRankingsJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List */
  async list(params?: AiModelRankingsJobsListParams): Promise<ModelRankingRefreshJobHistoryPage> {
    const query = buildQueryString([
      { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingRefreshJobHistoryPage>(appendQueryString(backendApiPath(`/ai/model_rankings/jobs`), query));
  }
}

export interface AiModelRankingsListParams {
  rankScope?: string;
  vendorCode?: string;
  modality?: string;
  q?: string;
  pageSize?: number;
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


/** List */
  async list(params?: AiModelRankingsListParams): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'modality', value: params?.modality, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<ModelRankingsPage>(appendQueryString(backendApiPath(`/ai/model_rankings`), query));
  }

/** Refresh */
  async refresh(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/model_rankings/refresh`));
  }
}

export class AiModelMappingsResolveApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create */
  async create(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/model_mappings/resolve`));
  }
}

export class AiModelMappingsApi {
  private client: HttpClient;
  public readonly resolve: AiModelMappingsResolveApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.resolve = new AiModelMappingsResolveApi(client);
  }


/** List */
  async list(): Promise<NoData> {
    return this.client.get<NoData>(backendApiPath(`/ai/model_mappings`));
  }

/** Create */
  async create(): Promise<NoData> {
    return this.client.post<NoData>(backendApiPath(`/ai/model_mappings`));
  }

/** Replace */
  async replace(): Promise<NoData> {
    return this.client.put<NoData>(backendApiPath(`/ai/model_mappings`));
  }

/** Delete */
  async delete(mappingId: string): Promise<NoData> {
    return this.client.delete<NoData>(backendApiPath(`/ai/model_mappings/${serializePathParameter(mappingId, { name: 'mappingId', style: 'simple', explode: false })}`));
  }

/** Update */
  async update(mappingId: string): Promise<NoData> {
    return this.client.patch<NoData>(backendApiPath(`/ai/model_mappings/${serializePathParameter(mappingId, { name: 'mappingId', style: 'simple', explode: false })}`));
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
  public readonly voices: AiVoicesApi;
  public readonly modelVoices: AiModelVoicesApi;
  public readonly videoProfiles: AiVideoProfilesApi;
  public readonly modelVideoProfiles: AiModelVideoProfilesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.modelMappings = new AiModelMappingsApi(client);
    this.modelRankings = new AiModelRankingsApi(client);
    this.modelVendors = new AiModelVendorsApi(client);
    this.models = new AiModelsApi(client);
    this.aiResourceGroups = new AiAiResourceGroupsApi(client);
    this.aiResources = new AiAiResourcesApi(client);
    this.voices = new AiVoicesApi(client);
    this.modelVoices = new AiModelVoicesApi(client);
    this.videoProfiles = new AiVideoProfilesApi(client);
    this.modelVideoProfiles = new AiModelVideoProfilesApi(client);
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
