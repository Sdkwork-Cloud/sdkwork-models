import type { AdminModelCatalogSyncResponse } from './admin-model-catalog-sync-response';

/** Models refresh result schema exposed by Claw Router. */
export interface ModelsRefreshResult {
  /** Business response code. */
  code: string;
  /** Data field on models refresh result. */
  data?: AdminModelCatalogSyncResponse;
  /** Human-readable response message. */
  msg?: string;
}
