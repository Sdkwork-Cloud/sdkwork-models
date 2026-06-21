import type { AdminModelMappingRuleBinding } from './admin-model-mapping-rule-binding';
import type { AdminModelMappingRuleItem } from './admin-model-mapping-rule-item';

/** Admin model mapping rule schema exposed by Claw Router. */
export interface AdminModelMappingRule {
  /** Binding type field on admin model mapping rule. */
  bindingType: 'global' | 'vendor' | 'channel_group' | 'channel' | 'provider_account' | 'site' | 'site_service';
  /** Bindings field on admin model mapping rule. */
  bindings: AdminModelMappingRuleBinding[];
  /** Created at field on admin model mapping rule. */
  createdAt?: string | null;
  /** Enabled field on admin model mapping rule. */
  enabled: boolean;
  /** Id field on admin model mapping rule. */
  id: string;
  /** Mapping items field on admin model mapping rule. */
  mappingItems: AdminModelMappingRuleItem[];
  /** Mapping mode field on admin model mapping rule. */
  mappingMode: 'alias';
  /** Match type field on admin model mapping rule. */
  matchType: 'exact';
  /** Source vendor code field on admin model mapping rule. */
  sourceVendorCode: string;
  /** Source vendor id field on admin model mapping rule. */
  sourceVendorId?: string | null;
  /** Target vendor code field on admin model mapping rule. */
  targetVendorCode: string;
  /** Target vendor id field on admin model mapping rule. */
  targetVendorId?: string | null;
  /** Updated at field on admin model mapping rule. */
  updatedAt?: string | null;
}
