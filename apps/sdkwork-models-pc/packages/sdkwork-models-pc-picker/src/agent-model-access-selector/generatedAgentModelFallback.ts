import { SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG } from '../unified-agent-model-selector/mainstreamAgentModelCatalog.generated.ts';
import type { MainstreamAgentModelCatalogEntry } from '../unified-agent-model-selector/unifiedAgentModelCatalog';
import { resolveAuthoritativeAgentModelCatalog } from './agentModelAccessCatalog.ts';
import type { AgentModelCatalogOption } from './agentModelAccessSelectorTypes';

export const GENERATED_MAINSTREAM_AGENT_MODEL_FALLBACK: readonly AgentModelCatalogOption[] =
  SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG.map((entry: MainstreamAgentModelCatalogEntry) => ({
    id: `catalog.${entry.catalogKey}`,
    catalogKey: entry.catalogKey,
    catalogVersion: entry.catalogVersion,
    contextTokens: entry.contextTokens,
    description: entry.description,
    inputModalities: entry.inputModalities,
    kind: 'built-in',
    label: entry.displayName,
    modelId: entry.modelId,
    maxOutputTokens: entry.maxOutputTokens,
    modalities: entry.modalities,
    outputModalities: entry.outputModalities,
    rankScore: entry.rankScore,
    releaseStage: entry.releaseStage,
    searchTerms: entry.searchTerms,
    sortOrder: entry.sortOrder,
    source: 'fallback',
    sourceObservedAt: entry.sourceObservedAt,
    supportedAgentProviderIds: entry.supportedProviderIds,
    supportsTools: entry.supportsTools,
    toolCallRounds: entry.toolCallRounds,
    vendorCode: entry.vendorCode,
    vendorName: entry.vendorName,
  }));

/** A non-empty database result replaces the fallback catalog without merging. */
export function resolveAgentModelCatalog(
  databaseModels: readonly AgentModelCatalogOption[],
  fallbackModels: readonly AgentModelCatalogOption[] = GENERATED_MAINSTREAM_AGENT_MODEL_FALLBACK,
): AgentModelCatalogOption[] {
  return resolveAuthoritativeAgentModelCatalog(databaseModels, fallbackModels);
}
