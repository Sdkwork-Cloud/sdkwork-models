import type { AiResourceGroupsCreateResult } from './ai-resource-groups-create-result';

export interface AiResourceGroupsCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
