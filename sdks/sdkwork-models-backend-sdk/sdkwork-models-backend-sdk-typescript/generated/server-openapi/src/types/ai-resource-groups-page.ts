import type { PageInfo } from './page-info';

/** Paginated AI resource groups returned by aiResourceGroups.list. */
export interface AiResourceGroupsPage {
  /** Items field on ai resources page. */
  items: Record<string, unknown>[];
  /** Page info field on ai resources page. */
  pageInfo: PageInfo;
}
