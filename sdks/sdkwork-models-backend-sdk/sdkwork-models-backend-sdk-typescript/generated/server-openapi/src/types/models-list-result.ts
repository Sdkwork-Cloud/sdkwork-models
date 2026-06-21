import type { AdminAiModelsResponse } from './admin-ai-models-response';

/** Models list result schema exposed by Claw Router. */
export interface ModelsListResult {
  /** Business response code. */
  code: string;
  /** Data field on models list result. */
  data?: AdminAiModelsResponse;
  /** Human-readable response message. */
  msg?: string;
}
