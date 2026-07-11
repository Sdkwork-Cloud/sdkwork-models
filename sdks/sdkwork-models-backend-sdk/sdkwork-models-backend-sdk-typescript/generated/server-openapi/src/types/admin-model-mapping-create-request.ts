import type { AdminModelMappingRuleBindingInput } from './admin-model-mapping-rule-binding-input';
import type { AdminModelMappingRuleItemInput } from './admin-model-mapping-rule-item-input';

/** Request body for creating a model mapping rule. */
export interface AdminModelMappingCreateRequest {
  sourceVendorId?: string | null;
  sourceVendorCode: string;
  targetVendorId?: string | null;
  targetVendorCode: string;
  mappingMode?: 'alias' | null;
  matchType?: 'exact' | null;
  enabled?: boolean;
  bindings: AdminModelMappingRuleBindingInput[];
  mappingItems: AdminModelMappingRuleItemInput[];
}
