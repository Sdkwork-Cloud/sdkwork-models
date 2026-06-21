import type { AdminModelMappingRule } from './admin-model-mapping-rule';

/** Admin model mappings response schema exposed by Claw Router. */
export interface AdminModelMappingsResponse {
  /** Items field on admin model mappings response. */
  items: AdminModelMappingRule[];
}
