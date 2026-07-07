import type { ModelCatalogPage } from './model-catalog-page';

/** Models list result schema exposed by Claw Router. */
export interface ModelsListResult {
  code: 0;
  data: unknown & ModelCatalogPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
