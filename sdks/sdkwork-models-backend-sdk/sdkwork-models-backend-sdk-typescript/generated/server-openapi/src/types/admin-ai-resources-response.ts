import type { AdminAiResourceItem } from './admin-ai-resource-item';

/** Admin ai resources response schema exposed by Claw Router. */
export interface AdminAiResourcesResponse {
  /** Items field on admin ai resources response. */
  items: AdminAiResourceItem[];
}
