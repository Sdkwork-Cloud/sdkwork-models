import type { AiResourceGroupResourcesPage } from './ai-resource-group-resources-page';

/** Ai resource groups resources list result schema exposed by Claw Router. */
export interface AiResourceGroupsResourcesListResult {
  code: 0;
  data: unknown & AiResourceGroupResourcesPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
