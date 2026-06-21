import type { AdminModelVendorItem } from './admin-model-vendor-item';

/** Admin model vendors response schema exposed by Claw Router. */
export interface AdminModelVendorsResponse {
  /** Model vendor snapshots returned by the backend. */
  items: AdminModelVendorItem[];
}
