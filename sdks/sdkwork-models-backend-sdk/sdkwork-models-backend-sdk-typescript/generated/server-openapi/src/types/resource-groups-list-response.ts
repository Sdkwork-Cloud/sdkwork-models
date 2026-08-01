import type { AiResourceGroupsPage } from './ai-resource-groups-page';

export interface ResourceGroupsListResponse {
  code: 0;
  data: unknown & AiResourceGroupsPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
