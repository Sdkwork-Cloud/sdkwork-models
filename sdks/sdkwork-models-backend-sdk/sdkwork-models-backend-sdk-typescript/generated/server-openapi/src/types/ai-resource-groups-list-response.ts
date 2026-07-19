import type { AiResourceGroupsPage } from './ai-resource-groups-page';

export interface AiResourceGroupsListResponse {
  code: 0;
  data: unknown & AiResourceGroupsPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
