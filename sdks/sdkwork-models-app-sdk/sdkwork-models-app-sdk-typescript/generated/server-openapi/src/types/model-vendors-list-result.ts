import type { AppModelVendorCatalogResponse } from './app-model-vendor-catalog-response';
import type { PageInfo } from './page-info';

/** Model vendors list result schema exposed by Cloud Router. */
export interface ModelVendorsListResult {
  code: 0;
  data: unknown & { items: AppModelVendorCatalogResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
