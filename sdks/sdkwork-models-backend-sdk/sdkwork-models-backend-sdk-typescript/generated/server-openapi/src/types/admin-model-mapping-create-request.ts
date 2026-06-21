import type { AdminModelMappingRuleBindingInput } from './admin-model-mapping-rule-binding-input';
import type { AdminModelMappingRuleItemInput } from './admin-model-mapping-rule-item-input';

/** Admin model mapping create request schema exposed by Claw Router. */
export interface AdminModelMappingCreateRequest {
  /** Bindings field on admin model mapping create request. */
  bindings: AdminModelMappingRuleBindingInput[];
  /** Enabled field on admin model mapping create request. */
  enabled?: boolean;
  /** Mapping items field on admin model mapping create request. */
  mappingItems: AdminModelMappingRuleItemInput[];
  /** Mapping mode field on admin model mapping create request. */
  mappingMode?: 'alias';
  /** Match type field on admin model mapping create request. */
  matchType?: 'exact';
  /** Source vendor code field on admin model mapping create request. */
  sourceVendorCode: string;
  /** Source vendor id field on admin model mapping create request. */
  sourceVendorId?: string | null;
  /** Target vendor code field on admin model mapping create request. */
  targetVendorCode: string;
  /** Target vendor id field on admin model mapping create request. */
  targetVendorId?: string | null;
}
