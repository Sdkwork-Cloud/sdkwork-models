import type { AdminDeleteResponse } from './admin-delete-response';

/** Models delete result schema exposed by Claw Router. */
export interface ModelsDeleteResult {
  /** Business response code. */
  code: string;
  /** Data field on models delete result. */
  data?: AdminDeleteResponse;
  /** Human-readable response message. */
  msg?: string;
}
