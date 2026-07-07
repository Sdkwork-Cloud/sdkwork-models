import type { JsonValue } from './json-value';
import type { PageInfo } from './page-info';

/** Ai resources page schema exposed by Claw Router. */
export interface AiResourcesPage {
  /** Items field on ai resources page. */
  items: Record<string, JsonValue>[];
  /** Page info field on ai resources page. */
  pageInfo: PageInfo;
}
