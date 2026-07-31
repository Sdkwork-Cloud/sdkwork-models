import type { AppModelVendorOption } from './app-model-vendor-option';

/** App model vendor catalog response schema exposed by Claw Router. */
export interface AppModelVendorCatalogResponse {
  /** Items field on app model vendor catalog response. */
  items: AppModelVendorOption[];
}
