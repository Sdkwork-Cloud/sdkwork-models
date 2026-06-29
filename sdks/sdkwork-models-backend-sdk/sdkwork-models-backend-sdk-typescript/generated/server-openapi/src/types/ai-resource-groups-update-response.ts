import type { AiResourceGroupsUpdateResult } from './ai-resource-groups-update-result';

export interface AiResourceGroupsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
