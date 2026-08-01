import type { AdminAiResourceGroupItem } from './admin-ai-resource-group-item';
import type { OffsetPageInfo } from './offset-page-info';

/** Paginated AI resource groups returned by resourceGroups.list. */
export interface AiResourceGroupsPage {
  items: AdminAiResourceGroupItem[];
  /** Offset pagination metadata. */
  pageInfo: OffsetPageInfo;
}
