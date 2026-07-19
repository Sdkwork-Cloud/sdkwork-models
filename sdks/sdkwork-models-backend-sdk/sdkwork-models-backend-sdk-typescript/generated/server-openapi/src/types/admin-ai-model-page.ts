import type { JsonValue } from './json-value';
import type { PageInfo } from './page-info';

/** Admin ai model page schema exposed by Claw Router. */
export interface AdminAiModelPage {
  /** Items field on admin ai model page. */
  items: Record<string, JsonValue>[];
  /** Page info field on admin ai model page. */
  pageInfo: PageInfo;
}
