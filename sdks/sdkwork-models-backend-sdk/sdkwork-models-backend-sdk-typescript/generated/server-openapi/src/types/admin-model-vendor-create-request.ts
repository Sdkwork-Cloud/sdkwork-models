/** Request body for creating an AI model vendor. */
export interface AdminModelVendorCreateRequest {
  /** Optional vendor code. The backend generates a code when omitted. */
  vendorCode?: string | null;
  /** Vendor display name. */
  name: string;
  status?: 'active' | 'disabled' | 'inactive' | null;
  color?: string | null;
  description?: string | null;
}
