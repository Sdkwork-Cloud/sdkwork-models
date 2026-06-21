import type { AdminModelMappingRuleBindingInput } from './admin-model-mapping-rule-binding-input';
import type { AdminModelMappingRuleItemInput } from './admin-model-mapping-rule-item-input';

/** Admin model mapping update request schema exposed by Claw Router. */
export interface AdminModelMappingUpdateRequest {
  /** Bindings field on admin model mapping update request. */
  bindings?: AdminModelMappingRuleBindingInput[];
  /** Enabled field on admin model mapping update request. */
  enabled?: boolean;
  /** Mapping items field on admin model mapping update request. */
  mappingItems?: AdminModelMappingRuleItemInput[];
  /** Mapping mode field on admin model mapping update request. */
  mappingMode?: 'alias';
  /** Match type field on admin model mapping update request. */
  matchType?: 'exact';
  /** Source vendor code field on admin model mapping update request. */
  sourceVendorCode?: string;
  /** Source vendor id field on admin model mapping update request. */
  sourceVendorId?: string | null;
  /** Target vendor code field on admin model mapping update request. */
  targetVendorCode?: string;
  /** Target vendor id field on admin model mapping update request. */
  targetVendorId?: string | null;
}
