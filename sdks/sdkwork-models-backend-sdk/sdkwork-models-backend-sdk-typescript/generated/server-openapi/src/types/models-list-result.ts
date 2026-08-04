import type { AdminAiModelPage } from './admin-ai-model-page';

/** Models list result schema exposed by Cloud Router. */
export interface ModelsListResult {
  code: 0;
  data: unknown & AdminAiModelPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
