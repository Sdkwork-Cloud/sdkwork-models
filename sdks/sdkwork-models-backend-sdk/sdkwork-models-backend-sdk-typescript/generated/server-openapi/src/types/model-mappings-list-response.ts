import type { ModelMappingsPage } from './model-mappings-page';

export interface ModelMappingsListResponse {
  code: 0;
  data: unknown & ModelMappingsPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
