/** Admin model vendor create request schema exposed by Claw Router. */
export interface AdminModelVendorCreateRequest {
  /** Safe style token used by the admin console. */
  color?: string;
  /** Vendor description shown in the admin console. */
  description?: string;
  /** Human-readable vendor display name. */
  name: string;
  /** Status field on admin model vendor create request. */
  status?: 'active' | 'inactive';
  /** Optional normalized vendor code; generated from name when omitted. */
  vendorCode?: string;
}
