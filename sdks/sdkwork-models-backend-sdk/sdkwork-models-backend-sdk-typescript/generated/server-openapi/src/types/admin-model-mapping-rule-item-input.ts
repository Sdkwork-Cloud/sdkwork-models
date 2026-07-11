/** Model mapping rule item input. */
export interface AdminModelMappingRuleItemInput {
  id?: string | null;
  sourceModel: string | null;
  sourceCatalogKey?: string | null;
  targetModel: string | null;
  targetCatalogKey?: string | null;
  targetProviderModel?: string | null;
  targetProviderNativeModel?: string | null;
  enabled?: boolean;
}
