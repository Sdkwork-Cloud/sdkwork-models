import type { AiResourcesPage } from './ai-resources-page';

export interface ResourcesListResponse {
  code: 0;
  data: unknown & AiResourcesPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
