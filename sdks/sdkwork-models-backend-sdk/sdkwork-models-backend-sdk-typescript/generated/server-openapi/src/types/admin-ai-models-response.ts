import type { AdminAiModelItem } from './admin-ai-model-item';

/** Admin ai models response schema exposed by Claw Router. */
export interface AdminAiModelsResponse {
  /** AI model catalog snapshots returned by the backend. */
  items: AdminAiModelItem[];
  /** Total count field on admin ai models response. */
  totalCount?: string;
}
