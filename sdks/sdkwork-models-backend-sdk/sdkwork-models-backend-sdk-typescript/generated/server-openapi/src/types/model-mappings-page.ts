import type { PageInfo } from './page-info';

/** Paginated model mapping rules returned by modelMappings.list. */
export interface ModelMappingsPage {
  /** Model mapping rule items. */
  items: Record<string, unknown>[];
  /** Offset pagination metadata for model mappings. */
  pageInfo: PageInfo;
}
