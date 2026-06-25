/** App model catalog reference price schema exposed by Claw Router. */
export interface AppModelCatalogReferencePrice {
  /** Billing meter field on app model catalog reference price. */
  billingMeter: string;
  /** Currency field on app model catalog reference price. */
  currency: string;
  /** Deployment or pricing region for this public reference price. Region is never encoded in catalogKey. */
  regionCode: string;
  /** Decimal unit price in the native official reference currency. */
  unitPrice: string;
}
