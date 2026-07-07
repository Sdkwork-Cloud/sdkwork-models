import type { JsonValue } from './json-value';
import type { PageInfo } from './page-info';

/** Model catalog page schema exposed by Claw Router. */
export interface ModelCatalogPage {
  /** Groups field on model catalog page. */
  groups: Record<string, unknown>[];
  /** Items field on model catalog page. */
  items: Record<string, JsonValue>[];
  /** Page info field on model catalog page. */
  pageInfo: PageInfo;
}
