import type { AiResourcesUpdateResult } from './ai-resources-update-result';

export interface AiResourcesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
