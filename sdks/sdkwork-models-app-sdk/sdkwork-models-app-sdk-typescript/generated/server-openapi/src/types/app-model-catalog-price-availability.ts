/** App model catalog price availability schema exposed by Cloud Router. */
export interface AppModelCatalogPriceAvailability {
  /** Reason field on app model catalog price availability. */
  reason: string | null;
  /** Status field on app model catalog price availability. */
  status: 'reference' | 'unavailable';
}
