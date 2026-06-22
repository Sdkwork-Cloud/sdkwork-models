import type { AdminAccountModelMappingInput } from './admin-account-model-mapping-input';

/** Admin account model mappings replace request schema exposed by Claw Router. */
export interface AdminAccountModelMappingsReplaceRequest {
  /** Account vendor code field on admin account model mappings replace request. */
  accountVendorCode: string;
  /** Channel id field on admin account model mappings replace request. */
  channelId: string;
  /** Channel name field on admin account model mappings replace request. */
  channelName?: string | null;
  /** Mappings field on admin account model mappings replace request. */
  mappings: AdminAccountModelMappingInput[];
}
