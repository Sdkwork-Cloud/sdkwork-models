/** Admin ai resource group member input schema exposed by Claw Router. */
export interface AdminAiResourceGroupMemberInput {
  /** Item role field on admin ai resource group member input. */
  itemRole?: 'included' | 'optional' | 'fallback';
  /** Resource code field on admin ai resource group member input. */
  resourceCode: string;
  /** Sort order field on admin ai resource group member input. */
  sortOrder?: string | null;
}
