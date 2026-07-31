import type { AppModelCatalogGroup } from './app-model-catalog-group';
import type { AppModelCatalogItem } from './app-model-catalog-item';
import type { PageInfo } from './page-info';

/** App model catalog page schema exposed by Claw Router. */
export interface AppModelCatalogPage {
  /** Groups field on app model catalog page. */
  groups: AppModelCatalogGroup[];
  /** Items field on app model catalog page. */
  items: AppModelCatalogItem[];
  /** Page info field on app model catalog page. */
  pageInfo: PageInfo;
}
