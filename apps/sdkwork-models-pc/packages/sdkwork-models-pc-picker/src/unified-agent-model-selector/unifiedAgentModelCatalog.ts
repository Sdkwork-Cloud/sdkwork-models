import type { UnifiedAgentModelOption } from './unifiedAgentModelSelectorTypes';

export interface MainstreamAgentModelCatalogEntry {
  catalogKey: string;
  catalogVersion: string;
  contextTokens?: number;
  description: string;
  displayName: string;
  inputModalities: readonly string[];
  lifecycle: 'active' | 'preview';
  maxOutputTokens?: number;
  modelId: string;
  modalities?: readonly string[];
  outputModalities: readonly string[];
  rankScore: number;
  releaseStage: string;
  searchTerms: readonly string[];
  sortOrder: number;
  sourceObservedAt: string;
  supportedProviderIds: readonly string[];
  supportsTools: boolean;
  toolCallRounds?: number;
  vendorCode: string;
  vendorName: string;
}

function compareText(left: string, right: string): number {
  const normalizedLeft = left.trim().toLowerCase();
  const normalizedRight = right.trim().toLowerCase();
  return normalizedLeft < normalizedRight ? -1 : normalizedLeft > normalizedRight ? 1 : 0;
}

function optionKindOrder(option: UnifiedAgentModelOption): number {
  return option.kind === 'built-in' ? 0 : 1;
}

function releaseStageOrder(option: UnifiedAgentModelOption): number {
  return option.releaseStage === 'active' ? 0 : option.releaseStage === 'preview' ? 1 : 2;
}

export function compareUnifiedAgentModelOptions(
  left: UnifiedAgentModelOption,
  right: UnifiedAgentModelOption,
): number {
  return optionKindOrder(left) - optionKindOrder(right)
    || (left.sortOrder ?? Number.MAX_SAFE_INTEGER)
      - (right.sortOrder ?? Number.MAX_SAFE_INTEGER)
    || compareText(left.vendorCode ?? '', right.vendorCode ?? '')
    || releaseStageOrder(left) - releaseStageOrder(right)
    || (right.rankScore ?? 0) - (left.rankScore ?? 0)
    || compareText(left.label, right.label)
    || compareText(left.modelId, right.modelId)
    || compareText(left.id, right.id);
}

export function sortUnifiedAgentModelOptions(
  options: readonly UnifiedAgentModelOption[],
): UnifiedAgentModelOption[] {
  return [...options].sort(compareUnifiedAgentModelOptions);
}

export function normalizeUnifiedAgentModelSearchQuery(query: string): string[] {
  return query
    .trim()
    .toLowerCase()
    .split(/\s+/u)
    .filter(Boolean);
}

export function unifiedAgentModelOptionMatchesQuery(
  option: UnifiedAgentModelOption,
  query: string,
): boolean {
  const queryTerms = normalizeUnifiedAgentModelSearchQuery(query);
  if (queryTerms.length === 0) {
    return true;
  }
  const haystack = [
    option.label,
    option.modelId,
    option.description,
    option.vendorCode,
    option.vendorName,
    option.metadataLabel,
    option.releaseStage,
    ...(option.searchTerms ?? []),
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
  return queryTerms.every((term) => haystack.includes(term));
}

export function filterUnifiedAgentModelOptions(
  options: readonly UnifiedAgentModelOption[],
  query: string,
): UnifiedAgentModelOption[] {
  return options.filter((option) => unifiedAgentModelOptionMatchesQuery(option, query));
}
