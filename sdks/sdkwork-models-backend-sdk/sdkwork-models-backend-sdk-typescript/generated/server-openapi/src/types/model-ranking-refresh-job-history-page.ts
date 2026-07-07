import type { JsonValue } from './json-value';
import type { PageInfo } from './page-info';

/** Model ranking refresh job history page schema exposed by Claw Router. */
export interface ModelRankingRefreshJobHistoryPage {
  /** Items field on model ranking refresh job history page. */
  items: Record<string, JsonValue>[];
  /** Page info field on model ranking refresh job history page. */
  pageInfo: PageInfo;
}
