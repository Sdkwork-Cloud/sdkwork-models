import type { AdminModelMappingRule } from './admin-model-mapping-rule';
import type { JsonNull } from './json-null';

/** Admin model mapping resolve response schema exposed by Claw Router. */
export interface AdminModelMappingResolveResponse {
  /** Matched field on admin model mapping resolve response. */
  matched: boolean;
  /** Matched binding type field on admin model mapping resolve response. */
  matchedBindingType?: 'global' | 'vendor' | 'channel_group' | 'channel' | 'provider_account' | 'site' | 'site_service' | null;
  /** Rule field on admin model mapping resolve response. */
  rule?: AdminModelMappingRule | JsonNull;
  /** Source model field on admin model mapping resolve response. */
  sourceModel: string;
  /** Target catalog key field on admin model mapping resolve response. */
  targetCatalogKey?: string | null;
  /** Target model field on admin model mapping resolve response. */
  targetModel: string;
  /** Target provider model field on admin model mapping resolve response. */
  targetProviderModel?: string | null;
  /** Target provider native model field on admin model mapping resolve response. */
  targetProviderNativeModel?: string | null;
  /** Target vendor code field on admin model mapping resolve response. */
  targetVendorCode?: string | null;
}
