/** Admin model vendor item schema exposed by Claw Router. */
export interface AdminModelVendorItem {
  /** Color field on admin model vendor item. */
  color: string;
  /** Description field on admin model vendor item. */
  description: string;
  /** Id field on admin model vendor item. */
  id: string;
  /** Name field on admin model vendor item. */
  name: string;
  /** Status field on admin model vendor item. */
  status: 'active' | 'inactive';
  /** Vendor code field on admin model vendor item. */
  vendorCode: string;
}
