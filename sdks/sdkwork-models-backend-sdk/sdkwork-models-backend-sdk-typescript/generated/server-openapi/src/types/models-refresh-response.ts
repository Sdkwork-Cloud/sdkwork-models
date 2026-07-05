import type { ModelCatalogSyncResult } from './model-catalog-sync-result';

export interface ModelsRefreshResponse {
  code: 0;
  data: unknown & ModelCatalogSyncResult;
  /** Server-owned request correlation id. */
  traceId: string;
}
