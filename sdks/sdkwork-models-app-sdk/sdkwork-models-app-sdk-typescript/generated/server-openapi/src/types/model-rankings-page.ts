import type { JsonValue } from './json-value';
import type { PageInfo } from './page-info';

/** Model rankings page schema exposed by Claw Router. */
export interface ModelRankingsPage {
  /** History field on model rankings page. */
  history: Record<string, JsonValue>[];
  /** Items field on model rankings page. */
  items: Record<string, JsonValue>[];
  /** Page info field on model rankings page. */
  pageInfo: PageInfo;
  /** Source field on model rankings page. */
  source: Record<string, JsonValue>;
}
