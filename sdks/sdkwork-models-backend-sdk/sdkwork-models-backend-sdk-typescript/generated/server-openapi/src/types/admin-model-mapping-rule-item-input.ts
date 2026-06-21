/** Admin model mapping rule item input schema exposed by Claw Router. */
export interface AdminModelMappingRuleItemInput {
  /** Enabled field on admin model mapping rule item input. */
  enabled?: boolean;
  /** Id field on admin model mapping rule item input. */
  id?: string | null;
  /** Source catalog key field on admin model mapping rule item input. */
  sourceCatalogKey?: string | null;
  /** Source model field on admin model mapping rule item input. */
  sourceModel: string;
  /** Target catalog key field on admin model mapping rule item input. */
  targetCatalogKey?: string | null;
  /** Target model field on admin model mapping rule item input. */
  targetModel: string;
  /** Target provider model field on admin model mapping rule item input. */
  targetProviderModel?: string | null;
  /** Target provider native model field on admin model mapping rule item input. */
  targetProviderNativeModel?: string | null;
}
