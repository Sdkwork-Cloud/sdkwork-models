import type { AdminAiResourceItem } from './admin-ai-resource-item';
import type { OffsetPageInfo } from './offset-page-info';

/** Paginated AI resources returned by resources.list. */
export interface AiResourcesPage {
  items: AdminAiResourceItem[];
  /** Offset pagination metadata. */
  pageInfo: OffsetPageInfo;
}
