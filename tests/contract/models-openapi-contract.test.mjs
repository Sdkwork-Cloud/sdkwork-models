#!/usr/bin/env node
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.cwd();

function readJson(relativePath) {
  return JSON.parse(readFileSync(join(root, relativePath), 'utf8'));
}

const backendOpenApi = readJson('apis/backend-api/intelligence/openapi.json');
const resolveOperation = backendOpenApi.paths?.['/backend/v3/api/ai/model_mappings/resolve']?.post;
const modelMappingsListOperation = backendOpenApi.paths?.['/backend/v3/api/ai/model_mappings']?.get;

const writeOperationRequestBodies = [
  ['modelVendors.create', 'post', '/backend/v3/api/ai/model_vendors', '#/components/schemas/AdminModelVendorCreateRequest'],
  ['models.create', 'post', '/backend/v3/api/ai/models', '#/components/schemas/AdminAiModelCreateRequest'],
  ['models.update', 'patch', '/backend/v3/api/ai/models/{modelId}', '#/components/schemas/AdminAiModelUpdateRequest'],
  ['models.refresh', 'post', '/backend/v3/api/ai/models/refresh', '#/components/schemas/AdminModelCatalogSyncRequest'],
  ['modelMappings.create', 'post', '/backend/v3/api/ai/model_mappings', '#/components/schemas/AdminModelMappingCreateRequest'],
  ['modelMappings.update', 'patch', '/backend/v3/api/ai/model_mappings/{mappingId}', '#/components/schemas/AdminModelMappingUpdateRequest'],
  ['modelMappings.resolve', 'post', '/backend/v3/api/ai/model_mappings/resolve', '#/components/schemas/AdminModelMappingResolveRequest'],
  ['modelRankings.refresh', 'post', '/backend/v3/api/ai/model_rankings/refresh', '#/components/schemas/ModelRankingRefreshTriggerRequest'],
  ['aiResources.create', 'post', '/backend/v3/api/ai/resources', '#/components/schemas/AdminAiResourceCreateRequest'],
  ['aiResources.update', 'put', '/backend/v3/api/ai/resources/{resourceId}', '#/components/schemas/AdminAiResourceUpdateRequest'],
  ['aiResourceGroups.create', 'post', '/backend/v3/api/ai/resource_groups', '#/components/schemas/AdminAiResourceGroupCreateRequest'],
  ['aiResourceGroups.update', 'patch', '/backend/v3/api/ai/resource_groups/{groupId}', '#/components/schemas/AdminAiResourceGroupUpdateRequest'],
];

const expectedGeneratedBodyMethods = [
  ['modelVendors.create', /async create\(\s*body:\s*AdminModelVendorCreateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.post<Record<string, unknown>>\(backendApiPath\(`\/ai\/model_vendors`\), body, undefined, undefined, 'application\/json'\)/],
  ['models.create', /async create\(\s*body:\s*AdminAiModelCreateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.post<Record<string, unknown>>\(backendApiPath\(`\/ai\/models`\), body, undefined, undefined, 'application\/json'\)/],
  ['models.update', /async update\(\s*modelId:\s*string,\s*body:\s*AdminAiModelUpdateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.patch<Record<string, unknown>>\(backendApiPath\(`\/ai\/models\/\$\{serializePathParameter\(modelId,[\s\S]*?\}`\), body, undefined, undefined, 'application\/json'\)/],
  ['models.refresh', /async refresh\(\s*body:\s*AdminModelCatalogSyncRequest\s*\): Promise<ModelCatalogSyncResult>/, /this\.client\.post<ModelCatalogSyncResult>\(backendApiPath\(`\/ai\/models\/refresh`\), body, undefined, undefined, 'application\/json'\)/],
  ['modelMappings.create', /async create\(\s*body:\s*AdminModelMappingCreateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.post<Record<string, unknown>>\(backendApiPath\(`\/ai\/model_mappings`\), body, undefined, undefined, 'application\/json'\)/],
  ['modelMappings.update', /async update\(\s*mappingId:\s*string,\s*body:\s*AdminModelMappingUpdateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.patch<Record<string, unknown>>\(backendApiPath\(`\/ai\/model_mappings\/\$\{serializePathParameter\(mappingId,[\s\S]*?\}`\), body, undefined, undefined, 'application\/json'\)/],
  ['modelMappings.resolve', /async resolve\(\s*body:\s*AdminModelMappingResolveRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.post<Record<string, unknown>>\(backendApiPath\(`\/ai\/model_mappings\/resolve`\), body, undefined, undefined, 'application\/json'\)/],
  ['modelRankings.refresh', /async refresh\(\s*body:\s*ModelRankingRefreshTriggerRequest\s*\): Promise<ModelRankingRefreshTriggerResponse>/, /this\.client\.post<ModelRankingRefreshTriggerResponse>\(backendApiPath\(`\/ai\/model_rankings\/refresh`\), body, undefined, undefined, 'application\/json'\)/],
  ['aiResources.create', /async create\(\s*body:\s*AdminAiResourceCreateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.post<Record<string, unknown>>\(backendApiPath\(`\/ai\/resources`\), body, undefined, undefined, 'application\/json'\)/],
  ['aiResources.update', /async update\(\s*resourceId:\s*string,\s*body:\s*AdminAiResourceUpdateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.put<Record<string, unknown>>\(backendApiPath\(`\/ai\/resources\/\$\{serializePathParameter\(resourceId,[\s\S]*?\}`\), body, undefined, undefined, 'application\/json'\)/],
  ['aiResourceGroups.create', /async create\(\s*body:\s*AdminAiResourceGroupCreateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.post<Record<string, unknown>>\(backendApiPath\(`\/ai\/resource_groups`\), body, undefined, undefined, 'application\/json'\)/],
  ['aiResourceGroups.update', /async update\(\s*groupId:\s*string,\s*body:\s*AdminAiResourceGroupUpdateRequest\s*\): Promise<Record<string, unknown>>/, /this\.client\.patch<Record<string, unknown>>\(backendApiPath\(`\/ai\/resource_groups\/\$\{serializePathParameter\(groupId,[\s\S]*?\}`\), body, undefined, undefined, 'application\/json'\)/],
];

for (const [operationId, method, path, schemaRef] of writeOperationRequestBodies) {
  const operation = backendOpenApi.paths?.[path]?.[method];
  assert.ok(operation, `${operationId} operation must exist`);
  assert.equal(operation.operationId, operationId);
  assert.equal(
    operation.requestBody?.content?.['application/json']?.schema?.$ref,
    schemaRef,
    `${operationId} must accept ${schemaRef} as its JSON request body`,
  );
}

assert.ok(resolveOperation, 'model mapping resolve operation must exist');
assert.equal(resolveOperation.operationId, 'modelMappings.resolve');
assert.equal(
  resolveOperation.requestBody?.content?.['application/json']?.schema?.$ref,
  '#/components/schemas/AdminModelMappingResolveRequest',
  'modelMappings.resolve must accept the runtime resolve request body',
);

const resolveRequest = backendOpenApi.components?.schemas?.AdminModelMappingResolveRequest;
assert.ok(resolveRequest, 'AdminModelMappingResolveRequest schema must be present');
assert.equal(resolveRequest.type, 'object');
assert.equal(resolveRequest.additionalProperties, false);
assert.deepEqual(resolveRequest.required, ['sourceModel']);
assert.ok(resolveRequest.properties?.sourceModel, 'sourceModel property must be present');
assert.ok(resolveRequest.properties?.vendorCode, 'vendorCode property must be present');
assert.ok(resolveRequest.properties?.channelCode, 'channelCode property must be present');
assert.ok(resolveRequest.properties?.providerAccountCode, 'providerAccountCode property must be present');

const syncRequest = backendOpenApi.components?.schemas?.AdminModelCatalogSyncRequest;
assert.ok(syncRequest, 'AdminModelCatalogSyncRequest schema must be present');
assert.deepEqual(
  syncRequest.properties?.mode?.enum,
  ['official_refresh', 'vendor_refresh', 'catalog_version_refresh', 'dry_run'],
  'AdminModelCatalogSyncRequest.mode must match the runtime sync mode validator',
);

assert.ok(modelMappingsListOperation, 'model mappings list operation must exist');
assert.equal(modelMappingsListOperation.operationId, 'modelMappings.list');
const modelMappingQueryParams = new Map(
  (modelMappingsListOperation.parameters ?? []).map((parameter) => [parameter.name, parameter]),
);
for (const name of ['binding_type', 'vendor_code', 'channel_id', 'channel_code', 'q']) {
  assert.ok(modelMappingQueryParams.has(name), `modelMappings.list must expose ${name} query parameter`);
}
assert.equal(
  modelMappingQueryParams.get('channel_id')?.schema?.['x-sdkwork-int64-string'],
  true,
  'modelMappings.list channel_id must be generated as an int64 string query parameter',
);
assert.equal(
  modelMappingsListOperation.responses?.['200']?.content?.['application/json']?.schema?.allOf?.[1]?.properties?.data?.allOf?.[0]?.$ref,
  '#/components/schemas/ModelMappingsPage',
  'modelMappings.list must return the runtime items/pageInfo data shape',
);
const modelMappingsPage = backendOpenApi.components?.schemas?.ModelMappingsPage;
assert.ok(modelMappingsPage, 'ModelMappingsPage schema must be present');
assert.ok(modelMappingsPage.properties?.items, 'ModelMappingsPage.items schema must be present');
assert.ok(modelMappingsPage.properties?.pageInfo, 'ModelMappingsPage.pageInfo schema must be present');

const generatedApiSource = readFileSync(
  join(root, 'sdks/sdkwork-models-backend-sdk/sdkwork-models-backend-sdk-typescript/generated/server-openapi/src/api/ai.ts'),
  'utf8',
);
const catalogServiceSource = readFileSync(
  join(root, 'apps/sdkwork-models-pc/packages/sdkwork-models-pc-admin-catalog/src/modelService.ts'),
  'utf8',
);
const resourceGroupServiceSource = readFileSync(
  join(root, 'apps/sdkwork-models-pc/packages/sdkwork-models-pc-admin-resource/src/resourceGroupService.ts'),
  'utf8',
);
const iamModuleManifest = readJson('specs/iam.module.manifest.json');

for (const [operationId, signaturePattern, transportPattern] of expectedGeneratedBodyMethods) {
  assert.match(
    generatedApiSource,
    signaturePattern,
    `generated TypeScript backend SDK must expose ${operationId}(body)`,
  );
  assert.match(
    generatedApiSource,
    transportPattern,
    `generated TypeScript backend SDK must send ${operationId} body to the backend API`,
  );
}

const modelMappingsListParamsBlock = generatedApiSource.match(
  /export interface AiModelMappingsListParams \{([\s\S]*?)\n\}/,
)?.[1];
assert.ok(
  modelMappingsListParamsBlock,
  'generated TypeScript backend SDK must expose modelMappings.list(params) filters',
);
for (const fieldPattern of [
  /page\?: number;/,
  /pageSize\?: number;/,
  /bindingType\?: 'global' \| 'vendor' \| 'channel_group' \| 'channel' \| 'provider_account' \| 'site' \| 'site_service';/,
  /vendorCode\?: string;/,
  /channelId\?: string;/,
  /channelCode\?: string;/,
  /q\?: string;/,
]) {
  assert.match(
    modelMappingsListParamsBlock,
    fieldPattern,
    'generated TypeScript backend SDK must expose modelMappings.list(params) filters',
  );
}
assert.match(
  generatedApiSource,
  /async list\(params\?: AiModelMappingsListParams\): Promise<ModelMappingsPage>/,
  'generated TypeScript backend SDK must accept params and return ModelMappingsPage for modelMappings.list',
);
for (const [wireName, modelName] of [
  ['binding_type', 'bindingType'],
  ['vendor_code', 'vendorCode'],
  ['channel_id', 'channelId'],
  ['channel_code', 'channelCode'],
  ['q', 'q'],
]) {
  assert.match(
    generatedApiSource,
    new RegExp(`\\{ name: '${wireName}', value: params\\?\\.${modelName}`),
    `generated TypeScript backend SDK must serialize modelMappings.list ${modelName} as ${wireName}`,
  );
}

assert.doesNotMatch(
  catalogServiceSource,
  /modelMappings\.resolve\.create/,
  'catalog service must use the generated modelMappings.resolve(body) method',
);
assert.match(
  catalogServiceSource,
  /modelMappings\.resolve\(input\)/,
  'catalog service must call modelMappings.resolve(input)',
);
assert.doesNotMatch(
  catalogServiceSource,
  /ensureDeleteResult/,
  'catalog service must not expect delete operations to return a JSON confirmation body',
);
assert.doesNotMatch(
  catalogServiceSource,
  /readRequiredRecord\(readApiRecord\(result\)\.data, 'Model mapping delete response is missing data'\)/,
  'model mapping delete must treat 204 No Content as success',
);
assert.doesNotMatch(
  catalogServiceSource,
  /AdminModelCatalogSyncResponse/,
  'catalog service must consume the current ModelCatalogSyncResult generated SDK type',
);
assert.match(
  catalogServiceSource,
  /ModelCatalogSyncResult\['mode'\]/,
  'catalog service sync report must derive mode from ModelCatalogSyncResult',
);
assert.match(
  catalogServiceSource,
  /modelRankings\.list\(\{\s*pageSize: 200\s*\}\)/,
  'catalog service must use generated pageSize params for model rankings list',
);
assert.match(
  catalogServiceSource,
  /modelRankings\.jobs\.list\(\{\s*pageSize: 20\s*\}\)/,
  'catalog service must use generated pageSize params for model ranking jobs',
);
assert.doesNotMatch(
  catalogServiceSource,
  /modelRankings\.(?:jobs\.)?list\(\{\s*limit:/,
  'catalog service must not use legacy limit params for generated list methods',
);
assert.doesNotMatch(
  catalogServiceSource,
  /limit\?: number;\s*\n\s*offset\?: number;/,
  'catalog service model list query must use standard page/pageSize params',
);
assert.doesNotMatch(
  catalogServiceSource,
  /fetchModelsPage\(\{[^}]*\blimit:/,
  'catalog service callers must not use legacy limit params for model pages',
);
assert.match(
  catalogServiceSource,
  /function modelType\(value: Model\['type'\]\): AdminAiModelType/,
  'catalog service must map UI model type labels to backend API model type values',
);
assert.doesNotMatch(
  catalogServiceSource,
  /return value;\s*\n\s*\}\s*\n\s*throw new Error\(value \? `Unsupported model type:/,
  'catalog service must not send UI model type labels directly to the backend SDK',
);
assert.match(
  catalogServiceSource,
  /releaseStage: 1,\s*\n\s*shelfState: 1,\s*\n\s*routingState: 1,/,
  'catalog service default model metadata must use numeric enum values required by the backend SDK',
);
assert.doesNotMatch(
  resourceGroupServiceSource,
  /readBoolean\(readApiRecord\(result\), 'deleted', false\)/,
  'resource group delete must treat 204 No Content as success',
);
assert.match(
  resourceGroupServiceSource,
  /function toSdkListParams\(query: ResourceListQuery\): \{ page\?: number; pageSize\?: number; q\?: string \}/,
  'resource group service must pass numeric pagination params to generated SDK list methods',
);
assert.doesNotMatch(
  resourceGroupServiceSource,
  /page:\s*'1'|pageSize:\s*'200'|String\(query\.page\)|String\(query\.pageSize\)/,
  'resource group service must not pass string pagination values to generated SDK list methods',
);

const backendPermissions = iamModuleManifest.permissions?.openapiAuthorities
  ?.find((authority) => authority.apiAuthority === 'sdkwork-models-backend-api')
  ?.operationPermissions ?? [];
const operationIds = new Set(backendPermissions.map((permission) => permission.operationId));
assert.ok(operationIds.has('modelMappings.resolve'), 'IAM manifest must grant modelMappings.resolve');
assert.ok(!operationIds.has('modelMappings.resolve.create'), 'IAM manifest must not keep stale modelMappings.resolve.create');

process.stdout.write('models-openapi-contract.test.mjs passed\n');
