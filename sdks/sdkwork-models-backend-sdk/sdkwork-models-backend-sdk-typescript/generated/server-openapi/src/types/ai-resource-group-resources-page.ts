import type { AdminAiResourceGroupResourceItem } from './admin-ai-resource-group-resource-item';
import type { OffsetPageInfo } from './offset-page-info';

/** Paginated resources returned by resourceGroups.resources.list. */
export interface AiResourceGroupResourcesPage {
  items: AdminAiResourceGroupResourceItem[];
  /** Offset pagination metadata. */
  pageInfo: OffsetPageInfo;
}
