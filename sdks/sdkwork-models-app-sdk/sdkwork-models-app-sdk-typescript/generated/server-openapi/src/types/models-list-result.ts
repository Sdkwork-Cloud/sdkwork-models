import type { AppModelCatalogPage } from './app-model-catalog-page';

/** Models list result schema exposed by Claw Router. */
export interface ModelsListResult {
  code: 0;
  data: unknown & AppModelCatalogPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
