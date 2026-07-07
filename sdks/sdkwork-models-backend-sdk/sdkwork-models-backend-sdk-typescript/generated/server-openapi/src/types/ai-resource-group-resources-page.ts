import type { JsonValue } from './json-value';
import type { PageInfo } from './page-info';

/** Ai resource group resources page schema exposed by Claw Router. */
export interface AiResourceGroupResourcesPage {
  /** Items field on ai resource group resources page. */
  items: Record<string, JsonValue>[];
  /** Page info field on ai resource group resources page. */
  pageInfo: PageInfo;
}
