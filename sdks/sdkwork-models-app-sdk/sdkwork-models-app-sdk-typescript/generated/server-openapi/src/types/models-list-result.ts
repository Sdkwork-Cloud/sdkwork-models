import type { AppModelCatalogResponse } from './app-model-catalog-response';

/** Models list result schema exposed by Claw Router. */
export interface ModelsListResult {
  /** Business response code. */
  code: string;
  /** Data field on models list result. */
  data?: AppModelCatalogResponse;
  /** Human-readable response message. */
  msg?: string;
}
