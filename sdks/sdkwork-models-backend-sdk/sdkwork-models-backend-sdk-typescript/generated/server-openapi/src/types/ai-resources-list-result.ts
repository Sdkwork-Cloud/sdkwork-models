import type { AiResourcesPage } from './ai-resources-page';

/** Ai resources list result schema exposed by Claw Router. */
export interface AiResourcesListResult {
  code: 0;
  data: unknown & AiResourcesPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
