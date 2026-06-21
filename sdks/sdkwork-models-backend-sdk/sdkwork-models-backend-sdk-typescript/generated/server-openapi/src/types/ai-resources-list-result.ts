import type { AdminAiResourcesResponse } from './admin-ai-resources-response';

/** Ai resources list result schema exposed by Claw Router. */
export interface AiResourcesListResult {
  /** Business response code. */
  code: string;
  /** Data field on ai resources list result. */
  data?: AdminAiResourcesResponse;
  /** Human-readable response message. */
  msg?: string;
}
