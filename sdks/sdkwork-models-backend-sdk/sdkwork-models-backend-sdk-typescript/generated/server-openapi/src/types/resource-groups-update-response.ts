import type { AdminAiResourceGroupResourceItem } from './admin-ai-resource-group-resource-item';

export interface ResourceGroupsUpdateResponse {
  code: 0;
  data: unknown & { item: AdminAiResourceGroupResourceItem; };
  /** Server-owned request correlation id. */
  traceId: string;
}
