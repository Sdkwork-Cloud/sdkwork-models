import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { AppModelAccessChannelItem, AppModelAccessChannelPresetsPage, AppModelAccessChannelsPage, AppModelAccessChannelUpsertRequest, AppModelCatalogPage, AppModelVendorCatalogResponse, ModelRankingsPage, PageInfo } from '../types';


export class AiModelAccessChannelPresetsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List official model access channel presets */
  async list(requestOptions?: ApiRequestOptions): Promise<AppModelAccessChannelPresetsPage> {
    return this.client.request<AppModelAccessChannelPresetsPage>(appApiPath(`/ai/model_access_channel_presets`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'data' });
  }
}

export interface AiModelAccessChannelsListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  kind?: 'official' | 'relay' | 'custom';
  vendorCode?: string;
  agentProviderId?: string;
}

export class AiModelAccessChannelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model access channels */
  async list(params?: AiModelAccessChannelsListParams, requestOptions?: ApiRequestOptions): Promise<AppModelAccessChannelsPage> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'kind', value: params?.kind, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'agent_provider_id', value: params?.agentProviderId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<AppModelAccessChannelsPage>(appendQueryString(appApiPath(`/ai/model_access_channels`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
  }

/** Create or update a model access channel */
  async upsert(channelCode: string, body: AppModelAccessChannelUpsertRequest, requestOptions?: ApiRequestOptions): Promise<AppModelAccessChannelItem> {
    return this.client.request<AppModelAccessChannelItem>(appApiPath(`/ai/model_access_channels/${serializePathParameter(channelCode, { name: 'channelCode', style: 'simple', explode: false })}`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'PUT' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export interface AiModelVideoProfilesListParams {
  vendorCode?: string;
}

export class AiModelVideoProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model video generation profiles */
  async list(modelId: string, params?: AiModelVideoProfilesListParams, requestOptions?: ApiRequestOptions): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<ModelRankingsPage>(appendQueryString(appApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}/video_profiles`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
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
  async list(params?: AiVideoProfilesListParams, requestOptions?: ApiRequestOptions): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'region_code', value: params?.regionCode, style: 'form', explode: true, allowReserved: false },
      { name: 'catalog_key', value: params?.catalogKey, style: 'form', explode: true, allowReserved: false },
      { name: 'model_id', value: params?.modelId, style: 'form', explode: true, allowReserved: false },
      { name: 'generation_mode', value: params?.generationMode, style: 'form', explode: true, allowReserved: false },
      { name: 'duration_tier_code', value: params?.durationTierCode, style: 'form', explode: true, allowReserved: false },
      { name: 'resolution', value: params?.resolution, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<ModelRankingsPage>(appendQueryString(appApiPath(`/ai/video_profiles`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
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
  async list(modelId: string, params?: AiModelVoicesListParams, requestOptions?: ApiRequestOptions): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<ModelRankingsPage>(appendQueryString(appApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}/voices`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
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
  async list(params?: AiVoicesListParams, requestOptions?: ApiRequestOptions): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'region_code', value: params?.regionCode, style: 'form', explode: true, allowReserved: false },
      { name: 'locale', value: params?.locale, style: 'form', explode: true, allowReserved: false },
      { name: 'catalog_key', value: params?.catalogKey, style: 'form', explode: true, allowReserved: false },
      { name: 'model_id', value: params?.modelId, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<ModelRankingsPage>(appendQueryString(appApiPath(`/ai/voices`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
  }
}

export interface AiModelsListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  billingMeter?: string;
  vendorCode?: string;
  vendorCodes?: string[];
  modalities?: string[];
  capabilities?: string[];
  categories?: string[];
  groups?: string[];
}

export class AiModelsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List models */
  async list(params?: AiModelsListParams, requestOptions?: ApiRequestOptions): Promise<AppModelCatalogPage> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'billing_meter', value: params?.billingMeter, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_codes', value: params?.vendorCodes, style: 'form', explode: false, allowReserved: false },
      { name: 'modalities', value: params?.modalities, style: 'form', explode: false, allowReserved: false },
      { name: 'capabilities', value: params?.capabilities, style: 'form', explode: false, allowReserved: false },
      { name: 'categories', value: params?.categories, style: 'form', explode: false, allowReserved: false },
      { name: 'groups', value: params?.groups, style: 'form', explode: false, allowReserved: false },
    ]);
    return this.client.request<AppModelCatalogPage>(appendQueryString(appApiPath(`/ai/models`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
  }
}

export class AiModelVendorsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model vendors */
  async list(requestOptions?: ApiRequestOptions): Promise<{ items: AppModelVendorCatalogResponse[]; pageInfo: PageInfo; }> {
    return this.client.request<{ items: AppModelVendorCatalogResponse[]; pageInfo: PageInfo; }>(appApiPath(`/ai/model_vendors`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
  }
}

export interface AiModelRankingsListParams {
  rankScope?: string;
  vendorCode?: string;
  modality?: string;
  q?: string;
  page?: number;
  pageSize?: number;
}

export class AiModelRankingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List model rankings */
  async list(params?: AiModelRankingsListParams, requestOptions?: ApiRequestOptions): Promise<ModelRankingsPage> {
    const query = buildQueryString([
      { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
      { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
      { name: 'modality', value: params?.modality, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.request<ModelRankingsPage>(appendQueryString(appApiPath(`/ai/model_rankings`), query), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, skipAuth: true, sdkworkUnwrapKind: 'page' });
  }
}

export class AiApi {
  private client: HttpClient;
  public readonly modelRankings: AiModelRankingsApi;
  public readonly modelVendors: AiModelVendorsApi;
  public readonly models: AiModelsApi;
  public readonly voices: AiVoicesApi;
  public readonly modelVoices: AiModelVoicesApi;
  public readonly videoProfiles: AiVideoProfilesApi;
  public readonly modelVideoProfiles: AiModelVideoProfilesApi;
  public readonly modelAccessChannels: AiModelAccessChannelsApi;
  public readonly modelAccessChannelPresets: AiModelAccessChannelPresetsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.modelRankings = new AiModelRankingsApi(client);
    this.modelVendors = new AiModelVendorsApi(client);
    this.models = new AiModelsApi(client);
    this.voices = new AiVoicesApi(client);
    this.modelVoices = new AiModelVoicesApi(client);
    this.videoProfiles = new AiVideoProfilesApi(client);
    this.modelVideoProfiles = new AiModelVideoProfilesApi(client);
    this.modelAccessChannels = new AiModelAccessChannelsApi(client);
    this.modelAccessChannelPresets = new AiModelAccessChannelPresetsApi(client);
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
