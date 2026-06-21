import type { AdminModelVendorMutationResponse } from './admin-model-vendor-mutation-response';

/** Model vendors create result schema exposed by Claw Router. */
export interface ModelVendorsCreateResult {
  /** Business response code. */
  code: string;
  /** Data field on model vendors create result. */
  data?: AdminModelVendorMutationResponse;
  /** Human-readable response message. */
  msg?: string;
}
