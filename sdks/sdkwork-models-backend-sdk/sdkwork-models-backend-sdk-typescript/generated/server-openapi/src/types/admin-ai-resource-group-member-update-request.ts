/** Full request body for assigning or updating one AI resource-group member. */
export interface AdminAiResourceGroupMemberUpdateRequest {
  itemRole?: 'included' | 'optional' | 'fallback' | null;
  sortOrder?: string | null;
}
