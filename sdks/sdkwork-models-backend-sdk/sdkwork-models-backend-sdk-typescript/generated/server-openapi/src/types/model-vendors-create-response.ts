import type { ModelVendorsCreateResult } from './model-vendors-create-result';

export interface ModelVendorsCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
