/** Required metadata for an offset-paginated SDKWork list response. */
export interface OffsetPageInfo {
  mode: 'offset';
  page: number;
  pageSize: number;
  totalItems: string;
  totalPages: number;
  hasMore: boolean;
}
