import type { AdminModelVendorListResponse } from './admin-model-vendor-list-response';
import type { PageInfo } from './page-info';

/** Model vendors list result schema exposed by Cloud Router. */
export interface ModelVendorsListResult {
  code: 0;
  data: unknown & { items: AdminModelVendorListResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
