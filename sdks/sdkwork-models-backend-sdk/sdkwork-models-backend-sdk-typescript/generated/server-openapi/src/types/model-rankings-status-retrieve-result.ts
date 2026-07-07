import type { ModelCatalogPage } from './model-catalog-page';

/** Model rankings status retrieve result schema exposed by Claw Router. */
export interface ModelRankingsStatusRetrieveResult {
  code: 0;
  data: unknown & ModelCatalogPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
