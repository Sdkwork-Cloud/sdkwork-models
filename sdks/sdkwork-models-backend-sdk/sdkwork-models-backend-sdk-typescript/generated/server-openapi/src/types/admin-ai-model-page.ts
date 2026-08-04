import type { AdminAiModelItem } from './admin-ai-model-item';
import type { PageInfo } from './page-info';

/** Admin ai model page schema exposed by Cloud Router. */
export interface AdminAiModelPage {
  /** Items field on admin ai model page. */
  items: AdminAiModelItem[];
  /** Page info field on admin ai model page. */
  pageInfo: PageInfo;
}
