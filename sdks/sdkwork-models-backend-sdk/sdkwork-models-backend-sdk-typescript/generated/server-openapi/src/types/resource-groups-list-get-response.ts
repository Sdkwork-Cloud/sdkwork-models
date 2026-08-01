import type { AiResourceGroupResourcesPage } from './ai-resource-group-resources-page';

export interface ResourceGroupsListGetResponse {
  code: 0;
  data: unknown & AiResourceGroupResourcesPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
