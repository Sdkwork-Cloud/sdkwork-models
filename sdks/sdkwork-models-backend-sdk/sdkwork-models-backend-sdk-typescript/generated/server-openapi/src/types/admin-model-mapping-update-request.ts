import type { AdminModelMappingRuleBindingInput } from './admin-model-mapping-rule-binding-input';
import type { AdminModelMappingRuleItemInput } from './admin-model-mapping-rule-item-input';

/** Request body for updating a model mapping rule. */
export interface AdminModelMappingUpdateRequest {
  sourceVendorId?: string | null;
  sourceVendorCode?: string | null;
  targetVendorId?: string | null;
  targetVendorCode?: string | null;
  mappingMode?: 'alias' | null;
  matchType?: 'exact' | null;
  enabled?: boolean;
  bindings?: AdminModelMappingRuleBindingInput[];
  mappingItems?: AdminModelMappingRuleItemInput[];
}
