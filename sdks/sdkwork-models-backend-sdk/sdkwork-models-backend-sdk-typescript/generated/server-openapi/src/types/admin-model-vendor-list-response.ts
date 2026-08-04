import type { AdminModelVendorItem } from './admin-model-vendor-item';

/** Admin model vendor list response schema exposed by Cloud Router. */
export interface AdminModelVendorListResponse {
  /** Items field on admin model vendor list response. */
  items: AdminModelVendorItem[];
}
