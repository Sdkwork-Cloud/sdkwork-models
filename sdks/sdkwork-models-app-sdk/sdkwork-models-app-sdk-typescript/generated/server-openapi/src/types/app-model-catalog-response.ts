import type { AppModelCatalogGroupOption } from './app-model-catalog-group-option';
import type { AppModelCatalogItem } from './app-model-catalog-item';

/** App model catalog response schema exposed by Claw Router. */
export interface AppModelCatalogResponse {
  /** Complete admin-maintained channel group catalog for the model library sidebar. Groups are returned even when the current model filter result contains no matching model. */
  groups: AppModelCatalogGroupOption[];
  /** Items field on app model catalog response. */
  items: AppModelCatalogItem[];
}
