import type { AdminAccountModelMappingsReplaceResponse } from './admin-account-model-mappings-replace-response';

/** Model mappings replace result schema exposed by Claw Router. */
export interface ModelMappingsReplaceResult {
  /** Business response code. */
  code: string;
  /** Data field on model mappings replace result. */
  data?: AdminAccountModelMappingsReplaceResponse;
  /** Human-readable response message. */
  msg?: string;
}
