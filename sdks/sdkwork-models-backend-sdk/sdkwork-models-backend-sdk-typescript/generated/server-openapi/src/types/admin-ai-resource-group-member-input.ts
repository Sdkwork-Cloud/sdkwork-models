/** AI resource group member input. */
export interface AdminAiResourceGroupMemberInput {
  resourceCode: string;
  itemRole?: 'included' | 'optional' | 'fallback' | null;
  sortOrder?: string | null;
}
