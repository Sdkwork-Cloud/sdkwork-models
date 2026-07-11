/** Request body for resolving a model mapping rule without mutating catalog state. */
export interface AdminModelMappingResolveRequest {
  /** Source model identifier to resolve. */
  sourceModel: string;
  /** Optional source vendor code scope. */
  vendorCode?: string | null;
  /** Optional channel id scope encoded as int64 string. */
  channelId?: string | null;
  /** Optional channel code scope. */
  channelCode?: string | null;
  /** Optional provider account id scope encoded as int64 string. */
  providerAccountId?: string | null;
  /** Optional provider account binding code scope. */
  providerAccountCode?: string | null;
}
