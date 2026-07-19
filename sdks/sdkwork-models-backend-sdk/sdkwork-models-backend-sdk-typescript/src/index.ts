import {
  createClient as createGeneratedBackendClient,
  SdkworkBackendClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkBackendConfig } from '../generated/server-openapi/src/types/common';

export { SdkworkBackendClient, createGeneratedBackendClient };
export type { SdkworkBackendConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';
export type { AiModelMappingsListParams } from '../generated/server-openapi/src/api/ai';

export function createClient(config: SdkworkBackendConfig): SdkworkBackendClient {
  return createGeneratedBackendClient(config);
}
