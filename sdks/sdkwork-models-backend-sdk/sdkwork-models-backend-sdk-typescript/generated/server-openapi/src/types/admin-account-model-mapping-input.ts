/** Admin account model mapping input schema exposed by Claw Router. */
export interface AdminAccountModelMappingInput {
  /** Source model field on admin account model mapping input. */
  sourceModel: string;
  /** Target model field on admin account model mapping input. */
  targetModel: string;
  /** Target vendor code field on admin account model mapping input. */
  targetVendorCode?: string | null;
}
