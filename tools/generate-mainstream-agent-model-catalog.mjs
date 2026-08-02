import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const catalogRoot = path.join(repositoryRoot, 'models');
const mainstreamCatalogOutputFile = path.join(
  repositoryRoot,
  'apps',
  'sdkwork-models-pc',
  'packages',
  'sdkwork-models-pc-picker',
  'src',
  'unified-agent-model-selector',
  'mainstreamAgentModelCatalog.generated.ts',
);
const officialVendorPresetsOutputFile = path.join(
  repositoryRoot,
  'apps',
  'sdkwork-models-pc',
  'packages',
  'sdkwork-models-pc-picker',
  'src',
  'agent-model-access-selector',
  'officialModelVendorPresets.generated.ts',
);
const clawRouterProvidersFile = path.join(
  repositoryRoot,
  'overlays',
  'clawrouter',
  'providers.json',
);

const providerIdByClientApiCode = {
  claude_code: 'claude-code',
  codex: 'codex',
  gemini_cli: 'gemini',
};

async function readJson(file) {
  return JSON.parse(await readFile(file, 'utf8'));
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function lifecycleOrder(value) {
  return value === 'active' ? 0 : value === 'preview' ? 1 : 2;
}

function selectPreferredRegion(current, candidate) {
  if (!current || candidate.regionCode === 'global') {
    return candidate;
  }
  return current;
}

function listSupportedProviderIds(vendor) {
  return Object.entries(vendor.clientApiCompatibility ?? {})
    .filter(([, compatibility]) => (
      compatibility.supportStatus === 'supported'
      || compatibility.supportStatus === 'convert'
      || compatibility.supportStatus === 'partial'
    ))
    .map(([clientApiCode]) => providerIdByClientApiCode[clientApiCode])
    .filter(Boolean)
    .sort(compareText);
}

async function generateCatalogSource() {
  const index = await readJson(path.join(catalogRoot, 'index.json'));
  // Every catalog vendor that publishes chat models is an option, not only a
  // curated mainstream subset, so vendor lists stay complete for relay and
  // custom channels regardless of the database availability.
  const vendorEntriesByCode = new Map();
  for (const vendorEntry of index.vendors) {
    vendorEntriesByCode.set(
      vendorEntry.vendorCode,
      selectPreferredRegion(vendorEntriesByCode.get(vendorEntry.vendorCode), vendorEntry),
    );
  }

  const vendorsByCode = new Map();
  for (const [vendorCode, vendorEntry] of vendorEntriesByCode) {
    vendorsByCode.set(
      vendorCode,
      await readJson(path.join(catalogRoot, vendorEntry.path)),
    );
  }

  const modelsByCatalogKey = new Map();
  for (const vendorEntry of index.vendors) {
    for (const modelFile of vendorEntry.modelFiles) {
      const model = await readJson(path.join(catalogRoot, modelFile));
      if (
        model.primaryCapability !== 'chat'
        || model.shelfState !== 'listed'
        || model.routingState !== 'enabled'
        || !['active', 'preview'].includes(model.lifecycle)
      ) {
        continue;
      }
      const candidate = { ...model, regionCode: vendorEntry.regionCode };
      modelsByCatalogKey.set(
        model.catalogKey,
        selectPreferredRegion(modelsByCatalogKey.get(model.catalogKey), candidate),
      );
    }
  }

  const models = [...modelsByCatalogKey.values()].sort((left, right) => {
    const leftVendor = vendorsByCode.get(left.vendorCode);
    const rightVendor = vendorsByCode.get(right.vendorCode);
    return (leftVendor?.sortOrder ?? Number.MAX_SAFE_INTEGER)
      - (rightVendor?.sortOrder ?? Number.MAX_SAFE_INTEGER)
      || lifecycleOrder(left.lifecycle) - lifecycleOrder(right.lifecycle)
      || Number(right.rankScore ?? 0) - Number(left.rankScore ?? 0)
      || compareText(left.modelId.toLowerCase(), right.modelId.toLowerCase());
  });

  const entries = models.map((model, sortOrder) => {
    const vendor = vendorsByCode.get(model.vendorCode);
    return {
      catalogKey: model.catalogKey,
      catalogVersion: index.catalogVersion,
      contextTokens: model.contextTokens,
      description: model.description,
      displayName: model.displayName,
      inputModalities: model.inputModalities,
      lifecycle: model.lifecycle,
      maxOutputTokens: model.maxOutputTokens,
      modelId: model.modelId,
      modalities: model.modalities,
      outputModalities: model.outputModalities,
      rankScore: Number(model.rankScore ?? 0),
      releaseStage: model.releaseStage,
      searchTerms: [
        model.familyCode,
        ...(model.strengths ?? []),
      ].filter(Boolean),
      sortOrder,
      sourceObservedAt: model.source?.observedAt ?? index.generatedAt,
      supportedProviderIds: listSupportedProviderIds(vendor ?? {}),
      supportsTools: model.supportsTools,
      toolCallRounds: model.toolCallRounds,
      vendorCode: model.vendorCode,
      vendorName: model.vendorName ?? vendor?.displayName ?? model.vendorCode,
    };
  });

  return [
    '/* This file is generated by tools/generate-mainstream-agent-model-catalog.mjs. */',
    "import type { MainstreamAgentModelCatalogEntry } from './unifiedAgentModelCatalog';",
    '',
    `export const SDKWORK_MODELS_CATALOG_VERSION = ${JSON.stringify(index.catalogVersion)};`,
    `export const SDKWORK_MODELS_CATALOG_GENERATED_AT = ${JSON.stringify(index.generatedAt)};`,
    '',
    `export const SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG: readonly MainstreamAgentModelCatalogEntry[] = ${JSON.stringify(entries, null, 2)};`,
    '',
  ].join('\n');
}

async function generateOfficialVendorPresetsSource() {
  const [index, providerOverlay] = await Promise.all([
    readJson(path.join(catalogRoot, 'index.json')),
    readJson(clawRouterProvidersFile),
  ]);
  const directProviders = providerOverlay.providers.filter((provider) => (
    provider.providerCode.endsWith('_direct')
  ));
  const duplicateVendorCodes = directProviders
    .map((provider) => provider.vendorCode)
    .filter((vendorCode, index, vendorCodes) => vendorCodes.indexOf(vendorCode) !== index);
  if (duplicateVendorCodes.length > 0) {
    throw new Error(
      `Each official vendor must have exactly one direct provider: ${duplicateVendorCodes.join(', ')}`,
    );
  }

  // Every catalog vendor that publishes chat models is an official channel
  // option, not only the mainstream subset.
  const vendorEntriesByCode = new Map();
  for (const vendorEntry of index.vendors) {
    vendorEntriesByCode.set(
      vendorEntry.vendorCode,
      selectPreferredRegion(vendorEntriesByCode.get(vendorEntry.vendorCode), vendorEntry),
    );
  }

  async function listChatModels(vendorEntry) {
    const models = [];
    for (const [sortOrder, modelFile] of vendorEntry.modelFiles.entries()) {
      const model = await readJson(path.join(catalogRoot, modelFile));
      if (
        model.primaryCapability !== 'chat'
        || model.shelfState !== 'listed'
        || model.routingState !== 'enabled'
      ) {
        continue;
      }
      models.push({
        catalogKey: model.catalogKey,
        model: model.modelId,
        displayName: model.displayName,
        sortOrder,
      });
    }
    return models;
  }

  const directSortOrder = new Map(
    directProviders.map((provider, index) => [provider.vendorCode, index]),
  );
  const entries = [];
  for (const [vendorCode, vendorEntry] of vendorEntriesByCode) {
    const vendor = await readJson(path.join(catalogRoot, vendorEntry.path));
    const models = await listChatModels(vendorEntry);
    if (models.length === 0) {
      continue;
    }
    const directProvider = directProviders.find((provider) => (
      provider.vendorCode === vendorCode
    ));
    entries.push({
      baseUrl: directProvider?.baseUrl ?? '',
      channelName: vendor.displayName,
      protocol: directProvider?.protocol ?? '',
      providerCode: directProvider?.providerCode ?? `${vendorCode}.direct`,
      providerDisplayName: directProvider?.displayName ?? vendor.displayName,
      models,
      sortOrder: directSortOrder.has(vendorCode)
        ? directSortOrder.get(vendorCode)
        : directProviders.length + entries.length,
      vendorCode,
      vendorName: vendor.displayName,
    });
  }
  entries.sort((left, right) => left.sortOrder - right.sortOrder);

  return [
    '/* This file is generated by tools/generate-mainstream-agent-model-catalog.mjs. */',
    "import type { OfficialModelVendorPreset } from './agentModelAccessSelectorTypes';",
    '',
    `export const SDKWORK_OFFICIAL_MODEL_VENDOR_PRESETS = ${JSON.stringify(entries, null, 2)} as const satisfies readonly OfficialModelVendorPreset[];`,
    '',
  ].join('\n');
}

const generatedOutputs = [
  [mainstreamCatalogOutputFile, await generateCatalogSource()],
  [officialVendorPresetsOutputFile, await generateOfficialVendorPresetsSource()],
];
if (process.argv.includes('--check')) {
  for (const [outputFile, source] of generatedOutputs) {
    const current = await readFile(outputFile, 'utf8').catch(() => '');
    if (current !== source) {
      throw new Error(
        `Generated picker catalog ${path.relative(repositoryRoot, outputFile)} is stale. `
        + 'Run node tools/generate-mainstream-agent-model-catalog.mjs.',
      );
    }
  }
} else {
  await Promise.all(generatedOutputs.map(([outputFile, source]) => (
    writeFile(outputFile, source, 'utf8')
  )));
}
