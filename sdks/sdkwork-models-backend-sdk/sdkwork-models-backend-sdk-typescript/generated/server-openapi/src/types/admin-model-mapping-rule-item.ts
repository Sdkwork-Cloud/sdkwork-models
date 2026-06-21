/** Admin model mapping rule item schema exposed by Claw Router. */
export interface AdminModelMappingRuleItem {
  /** Created at field on admin model mapping rule item. */
  createdAt?: string | null;
  /** Enabled field on admin model mapping rule item. */
  enabled: boolean;
  /** Id field on admin model mapping rule item. */
  id: string;
  /** Sort order field on admin model mapping rule item. */
  sortOrder: string;
  /** Source catalog key field on admin model mapping rule item. */
  sourceCatalogKey?: string | null;
  /** Source model field on admin model mapping rule item. */
  sourceModel: string;
  /** Target catalog key field on admin model mapping rule item. */
  targetCatalogKey?: string | null;
  /** Target model field on admin model mapping rule item. */
  targetModel: string;
  /** Target provider model field on admin model mapping rule item. */
  targetProviderModel?: string | null;
  /** Target provider native model field on admin model mapping rule item. */
  targetProviderNativeModel?: string | null;
  /** Updated at field on admin model mapping rule item. */
  updatedAt?: string | null;
}
