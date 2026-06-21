'use strict';

var sdkCommon = require('@sdkwork/sdk-common');

class HttpClient extends sdkCommon.BaseHttpClient {
    constructor(config) {
        super(config);
    }
    getInternalAuthConfig() {
        const self = this;
        self.authConfig = self.authConfig || {};
        return self.authConfig;
    }
    getInternalHeaders() {
        const self = this;
        self.config = self.config || {};
        self.config.headers = self.config.headers || {};
        return self.config.headers;
    }
    buildRequestHeaders(headers, contentType) {
        const mergedHeaders = {
            ...(headers ?? {}),
        };
        if (contentType && contentType.toLowerCase() !== 'multipart/form-data') {
            mergedHeaders['Content-Type'] = contentType;
        }
        return Object.keys(mergedHeaders).length > 0 ? mergedHeaders : undefined;
    }
    buildHeaders(config, skipAuth = false) {
        const headers = super.buildHeaders(config, skipAuth);
        if (!skipAuth && !config?.skipAuth) {
            return headers;
        }
        [
            HttpClient.ACCESS_TOKEN_HEADER,
            'Authorization',
            'Access-Token',
            ['X', 'API', 'Key'].join('-'),
            'X-Tenant-Id',
            'X-Organization-Id',
            'X-Platform',
            'X-User-Id',
            'X-Sdkwork-Tenant-Id',
            'X-Sdkwork-Organization-Id',
            'X-Sdkwork-User-Id',
        ].forEach((key) => {
            delete headers[key];
        });
        return headers;
    }
    buildRequestBody(body, contentType) {
        if (body == null) {
            return body;
        }
        const normalizedContentType = (contentType ?? '').toLowerCase();
        if (normalizedContentType === 'application/x-www-form-urlencoded') {
            return this.encodeFormBody(body);
        }
        if (normalizedContentType === 'multipart/form-data') {
            return this.encodeMultipartBody(body);
        }
        return body;
    }
    encodeMultipartBody(body) {
        if (body instanceof FormData) {
            return body;
        }
        const formData = new FormData();
        if (body instanceof Map) {
            for (const [key, value] of body.entries()) {
                this.appendMultipartValue(formData, String(key), value);
            }
            return formData;
        }
        if (typeof body === 'object') {
            const record = body;
            for (const [key, value] of Object.entries(record)) {
                if (this.isMultipartMetadataField(key)) {
                    continue;
                }
                this.appendMultipartValue(formData, key, value, this.resolveMultipartFileName(record, key));
            }
            return formData;
        }
        this.appendMultipartValue(formData, 'value', body);
        return formData;
    }
    appendMultipartValue(formData, key, value, fileName) {
        if (value == null) {
            return;
        }
        if (Array.isArray(value)) {
            value.forEach((item) => this.appendMultipartValue(formData, key, item, fileName));
            return;
        }
        if (value instanceof Blob) {
            if (fileName) {
                formData.append(key, value, fileName);
                return;
            }
            formData.append(key, value);
            return;
        }
        if (value instanceof Date) {
            formData.append(key, value.toISOString());
            return;
        }
        if (typeof value === 'object') {
            formData.append(key, JSON.stringify(value));
            return;
        }
        formData.append(key, String(value));
    }
    resolveMultipartFileName(record, key) {
        const fieldSpecificName = record[`${key}FileName`];
        if (typeof fieldSpecificName === 'string' && fieldSpecificName.trim()) {
            return fieldSpecificName.trim();
        }
        const genericName = record.fileName;
        if (key === 'file' && typeof genericName === 'string' && genericName.trim()) {
            return genericName.trim();
        }
        return undefined;
    }
    isMultipartMetadataField(key) {
        return key === 'fileName' || key.endsWith('FileName');
    }
    encodeFormBody(body) {
        if (body instanceof URLSearchParams) {
            return body.toString();
        }
        if (typeof body === 'string') {
            return body;
        }
        const params = new URLSearchParams();
        if (body instanceof Map) {
            for (const [key, value] of body.entries()) {
                this.appendFormValue(params, String(key), value);
            }
            return params.toString();
        }
        if (typeof body === 'object') {
            for (const [key, value] of Object.entries(body)) {
                this.appendFormValue(params, key, value);
            }
            return params.toString();
        }
        params.append('value', String(body));
        return params.toString();
    }
    appendFormValue(params, key, value) {
        if (value == null) {
            return;
        }
        if (Array.isArray(value)) {
            value.forEach((item) => this.appendFormValue(params, key, item));
            return;
        }
        if (value instanceof Date) {
            params.append(key, value.toISOString());
            return;
        }
        if (typeof value === 'object') {
            params.append(key, JSON.stringify(value));
            return;
        }
        params.append(key, String(value));
    }
    setAuthToken(token) {
        super.setAuthToken(token);
    }
    setAccessToken(token) {
        const headers = this.getInternalHeaders();
        headers[HttpClient.ACCESS_TOKEN_HEADER] = token;
        super.setAccessToken(token);
    }
    setTokenManager(manager) {
        const baseProto = Object.getPrototypeOf(HttpClient.prototype);
        if (typeof baseProto.setTokenManager === 'function') {
            baseProto.setTokenManager.call(this, manager);
            return;
        }
        this.getInternalAuthConfig().tokenManager = manager;
    }
    applySdkworkAuthHeaders(headers) {
        const authConfig = this.getInternalAuthConfig();
        const tokenManager = authConfig.tokenManager;
        const accessToken = tokenManager?.getAccessToken?.();
        if (!accessToken) {
            return headers;
        }
        return {
            ...(headers ?? {}),
            [HttpClient.ACCESS_TOKEN_HEADER]: accessToken,
        };
    }
    async request(path, options = {}) {
        const execute = this.execute;
        if (typeof execute !== 'function') {
            throw new Error('BaseHttpClient execute method is not available');
        }
        const { body, headers, contentType, method = 'GET', skipAuth, ...rest } = options;
        const requestHeaders = skipAuth ? headers : this.applySdkworkAuthHeaders(headers);
        return sdkCommon.withRetry(() => execute.call(this, {
            url: path,
            method,
            ...rest,
            skipAuth,
            body: this.buildRequestBody(body, contentType),
            headers: this.buildRequestHeaders(requestHeaders, body == null ? undefined : contentType),
        }), { maxRetries: 3 });
    }
    async *streamJson(path, options = {}) {
        const stream = sdkCommon.BaseHttpClient.prototype.stream;
        if (typeof stream !== 'function') {
            throw new Error('BaseHttpClient stream method is not available');
        }
        const { body, headers, contentType, method = 'GET', skipAuth, ...rest } = options;
        const authHeaders = skipAuth ? headers : this.applySdkworkAuthHeaders(headers);
        const requestHeaders = this.buildRequestHeaders({ Accept: 'text/event-stream', ...(authHeaders ?? {}) }, body == null ? undefined : contentType);
        for await (const data of stream.call(this, path, {
            method,
            ...rest,
            skipAuth,
            body: this.buildRequestBody(body, contentType),
            headers: requestHeaders,
        })) {
            if (data === '[DONE]') {
                return;
            }
            if (typeof data !== 'string' || data.trim().length === 0) {
                continue;
            }
            yield JSON.parse(data);
        }
    }
    async get(path, params, headers) {
        return this.request(path, { method: 'GET', params, headers });
    }
    async post(path, body, params, headers, contentType) {
        return this.request(path, { method: 'POST', body, params, headers, contentType });
    }
    async put(path, body, params, headers, contentType) {
        return this.request(path, { method: 'PUT', body, params, headers, contentType });
    }
    async delete(path, params, headers) {
        return this.request(path, { method: 'DELETE', params, headers });
    }
    async patch(path, body, params, headers, contentType) {
        return this.request(path, { method: 'PATCH', body, params, headers, contentType });
    }
}
HttpClient.ACCESS_TOKEN_HEADER = 'Access-Token';
function createHttpClient(config) {
    return new HttpClient(config);
}

const BACKEND_API_PREFIX = '/backend/v3/api';
function backendApiPath(path) {
    if (!path) {
        return BACKEND_API_PREFIX;
    }
    if (/^https?:\/\//i.test(path)) {
        return path;
    }
    const normalizedPrefixRaw = (BACKEND_API_PREFIX).trim();
    const normalizedPrefix = normalizedPrefixRaw
        ? `/${normalizedPrefixRaw.replace(/^\/+|\/+$/g, '')}`
        : '';
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    if (!normalizedPrefix || normalizedPrefix === '/') {
        return normalizedPath;
    }
    if (normalizedPath === normalizedPrefix || normalizedPath.startsWith(`${normalizedPrefix}/`)) {
        return normalizedPath;
    }
    return `${normalizedPrefix}${normalizedPath}`;
}

class AiAiResourcesApi {
    constructor(client) {
        this.client = client;
    }
    /** List ai resources */
    async list() {
        return this.client.get(backendApiPath(`/ai/resources`));
    }
    /** Create ai resource */
    async create(body) {
        return this.client.post(backendApiPath(`/ai/resources`), body, undefined, undefined, 'application/json');
    }
    /** Update ai resource */
    async update(resourceId, body) {
        return this.client.put(backendApiPath(`/ai/resources/${serializePathParameter(resourceId, { name: 'resourceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
}
class AiAiResourceGroupsResourcesApi {
    constructor(client) {
        this.client = client;
    }
    /** List resource group resources */
    async list(groupIdOrCode) {
        return this.client.get(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupIdOrCode, { name: 'groupIdOrCode', style: 'simple', explode: false })}/resources`));
    }
}
class AiAiResourceGroupsApi {
    constructor(client) {
        this.client = client;
        this.resources = new AiAiResourceGroupsResourcesApi(client);
    }
    /** List resource groups */
    async list() {
        return this.client.get(backendApiPath(`/ai/resource_groups`));
    }
    /** Create resource group */
    async create(body) {
        return this.client.post(backendApiPath(`/ai/resource_groups`), body, undefined, undefined, 'application/json');
    }
    /** Delete resource group */
    async delete(groupId) {
        return this.client.delete(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupId, { name: 'groupId', style: 'simple', explode: false })}`));
    }
    /** Update resource group */
    async update(groupId, body) {
        return this.client.patch(backendApiPath(`/ai/resource_groups/${serializePathParameter(groupId, { name: 'groupId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
}
class AiModelsApi {
    constructor(client) {
        this.client = client;
    }
    /** List models */
    async list() {
        return this.client.get(backendApiPath(`/ai/models`));
    }
    /** Create model */
    async create(body) {
        return this.client.post(backendApiPath(`/ai/models`), body, undefined, undefined, 'application/json');
    }
    /** Sync vendors and models */
    async refresh(body) {
        return this.client.post(backendApiPath(`/ai/models/refresh`), body, undefined, undefined, 'application/json');
    }
    /** Delete model */
    async delete(modelId) {
        return this.client.delete(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}`));
    }
    /** Update model */
    async update(modelId, body) {
        return this.client.patch(backendApiPath(`/ai/models/${serializePathParameter(modelId, { name: 'modelId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
}
class AiModelVendorsApi {
    constructor(client) {
        this.client = client;
    }
    /** List vendors */
    async list() {
        return this.client.get(backendApiPath(`/ai/model_vendors`));
    }
    /** Create vendor */
    async create(body) {
        return this.client.post(backendApiPath(`/ai/model_vendors`), body, undefined, undefined, 'application/json');
    }
}
class AiModelRankingsStatusApi {
    constructor(client) {
        this.client = client;
    }
    /** List model ranking refresh status */
    async retrieve(params) {
        const query = buildQueryString([
            { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(backendApiPath(`/ai/model_rankings/status`), query));
    }
}
class AiModelRankingsJobsApi {
    constructor(client) {
        this.client = client;
    }
    /** List model ranking refresh jobs */
    async list(params) {
        const query = buildQueryString([
            { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
            { name: 'limit', value: params?.limit, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(backendApiPath(`/ai/model_rankings/jobs`), query));
    }
}
class AiModelRankingsApi {
    constructor(client) {
        this.client = client;
        this.jobs = new AiModelRankingsJobsApi(client);
        this.status = new AiModelRankingsStatusApi(client);
    }
    /** List model rankings */
    async list(params) {
        const query = buildQueryString([
            { name: 'rank_scope', value: params?.rankScope, style: 'form', explode: true, allowReserved: false },
            { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
            { name: 'modality', value: params?.modality, style: 'form', explode: true, allowReserved: false },
            { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
            { name: 'limit', value: params?.limit, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(backendApiPath(`/ai/model_rankings`), query));
    }
    /** Trigger model ranking refresh */
    async refresh(body) {
        return this.client.post(backendApiPath(`/ai/model_rankings/refresh`), body, undefined, undefined, 'application/json');
    }
}
class AiModelMappingsResolveApi {
    constructor(client) {
        this.client = client;
    }
    /** Resolve model mapping */
    async create(body) {
        return this.client.post(backendApiPath(`/ai/model_mappings/resolve`), body, undefined, undefined, 'application/json');
    }
}
class AiModelMappingsApi {
    constructor(client) {
        this.client = client;
        this.resolve = new AiModelMappingsResolveApi(client);
    }
    /** List model mappings */
    async list(params) {
        const query = buildQueryString([
            { name: 'binding_type', value: params?.bindingType, style: 'form', explode: true, allowReserved: false },
            { name: 'vendor_code', value: params?.vendorCode, style: 'form', explode: true, allowReserved: false },
            { name: 'channel_id', value: params?.channelId, style: 'form', explode: true, allowReserved: false },
            { name: 'channel_code', value: params?.channelCode, style: 'form', explode: true, allowReserved: false },
            { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(backendApiPath(`/ai/model_mappings`), query));
    }
    /** Create model mapping */
    async create(body) {
        return this.client.post(backendApiPath(`/ai/model_mappings`), body, undefined, undefined, 'application/json');
    }
    /** Delete model mapping */
    async delete(mappingId) {
        return this.client.delete(backendApiPath(`/ai/model_mappings/${serializePathParameter(mappingId, { name: 'mappingId', style: 'simple', explode: false })}`));
    }
    /** Update model mapping */
    async update(mappingId, body) {
        return this.client.patch(backendApiPath(`/ai/model_mappings/${serializePathParameter(mappingId, { name: 'mappingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
}
class AiApi {
    constructor(client) {
        this.client = client;
        this.modelMappings = new AiModelMappingsApi(client);
        this.modelRankings = new AiModelRankingsApi(client);
        this.modelVendors = new AiModelVendorsApi(client);
        this.models = new AiModelsApi(client);
        this.aiResourceGroups = new AiAiResourceGroupsApi(client);
        this.aiResources = new AiAiResourcesApi(client);
    }
}
function createAiApi(client) {
    return new AiApi(client);
}
function appendQueryString(path, rawQueryString) {
    const query = rawQueryString.replace(/^\?+/, '');
    if (!query) {
        return path;
    }
    return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
function serializePathParameter(value, spec) {
    if (value === undefined || value === null) {
        return '';
    }
    const style = spec.style || 'simple';
    if (Array.isArray(value)) {
        return serializePathArray(spec.name, value, style, spec.explode);
    }
    if (typeof value === 'object') {
        return serializePathObject(spec.name, value, style, spec.explode);
    }
    return pathPrefix(spec.name, style) + encodePathValue(serializePathPrimitive(value));
}
function serializePathArray(name, values, style, explode) {
    const serialized = values
        .filter((item) => item !== undefined && item !== null)
        .map((item) => encodePathValue(serializePathPrimitive(item)));
    if (serialized.length === 0) {
        return pathPrefix(name, style);
    }
    if (style === 'matrix') {
        return explode
            ? serialized.map((item) => `;${name}=${item}`).join('')
            : `;${name}=${serialized.join(',')}`;
    }
    return pathPrefix(name, style) + serialized.join(explode ? '.' : ',');
}
function serializePathObject(name, value, style, explode) {
    const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
    if (entries.length === 0) {
        return pathPrefix(name, style);
    }
    if (style === 'matrix') {
        return explode
            ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
            : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
    }
    const serialized = explode
        ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
        : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
    return pathPrefix(name, style) + serialized;
}
function pathPrefix(name, style, _objectValue) {
    if (style === 'label')
        return '.';
    if (style === 'matrix')
        return `;${name}`;
    return '';
}
function encodePathValue(value) {
    return encodeURIComponent(value);
}
function serializePathPrimitive(value) {
    if (value instanceof Date) {
        return value.toISOString();
    }
    if (typeof value === 'object') {
        return JSON.stringify(value);
    }
    return String(value);
}
function buildQueryString(parameters) {
    const pairs = [];
    for (const parameter of parameters) {
        appendSerializedParameter(pairs, parameter);
    }
    return pairs.join('&');
}
function appendSerializedParameter(pairs, parameter) {
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
        appendObjectParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
        return;
    }
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}
function appendArrayParameter(pairs, name, value, style, explode, allowReserved) {
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
function appendObjectParameter(pairs, name, value, style, explode, allowReserved) {
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
function appendDeepObjectParameter(pairs, name, value, allowReserved) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
        return;
    }
    for (const [key, entryValue] of Object.entries(value)) {
        if (entryValue === undefined || entryValue === null) {
            continue;
        }
        pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
}
function serializePrimitive(value) {
    if (value instanceof Date) {
        return value.toISOString();
    }
    if (typeof value === 'object') {
        return JSON.stringify(value);
    }
    return String(value);
}
function encodeQueryComponent(value) {
    return encodeURIComponent(value);
}
function encodeQueryValue(value, allowReserved) {
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

class SdkworkBackendClient {
    constructor(config) {
        this.httpClient = createHttpClient(config);
        this.ai = createAiApi(this.httpClient);
    }
    setAuthToken(token) {
        this.httpClient.setAuthToken(token);
        return this;
    }
    setAccessToken(token) {
        this.httpClient.setAccessToken(token);
        return this;
    }
    setTokenManager(manager) {
        this.httpClient.setTokenManager(manager);
        return this;
    }
    get http() {
        return this.httpClient;
    }
}
function createClient(config) {
    return new SdkworkBackendClient(config);
}

class BaseApi {
    constructor(http, basePath) {
        this.http = http;
        this.basePath = basePath;
    }
    async get(path, params, headers) {
        return this.http.get(`${this.basePath}${path}`, params, headers);
    }
    async post(path, body, params, headers, contentType) {
        return this.http.post(`${this.basePath}${path}`, body, params, headers, contentType);
    }
    async put(path, body, params, headers, contentType) {
        return this.http.put(`${this.basePath}${path}`, body, params, headers, contentType);
    }
    async delete(path, params, headers) {
        return this.http.delete(`${this.basePath}${path}`, params, headers);
    }
    async patch(path, body, params, headers, contentType) {
        return this.http.patch(`${this.basePath}${path}`, body, params, headers, contentType);
    }
    async request(method, path, body, params, headers, contentType) {
        return this.http.request(`${this.basePath}${path}`, { method: method, body, params, headers, contentType });
    }
}

Object.defineProperty(exports, "DEFAULT_TIMEOUT", {
    enumerable: true,
    get: function () { return sdkCommon.DEFAULT_TIMEOUT; }
});
Object.defineProperty(exports, "DefaultAuthTokenManager", {
    enumerable: true,
    get: function () { return sdkCommon.DefaultAuthTokenManager; }
});
Object.defineProperty(exports, "SUCCESS_CODES", {
    enumerable: true,
    get: function () { return sdkCommon.SUCCESS_CODES; }
});
Object.defineProperty(exports, "createTokenManager", {
    enumerable: true,
    get: function () { return sdkCommon.createTokenManager; }
});
exports.AiApi = AiApi;
exports.BaseApi = BaseApi;
exports.HttpClient = HttpClient;
exports.SdkworkBackendClient = SdkworkBackendClient;
exports.backendApiPath = backendApiPath;
exports.createAiApi = createAiApi;
exports.createClient = createClient;
exports.createHttpClient = createHttpClient;
