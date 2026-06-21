/** Admin ai resource member input schema exposed by Claw Router. */
export interface AdminAiResourceMemberInput {
  /** Member resource code field on admin ai resource member input. */
  memberResourceCode: string;
  /** Member role field on admin ai resource member input. */
  memberRole?: 'included' | 'optional' | 'fallback';
  /** Required field on admin ai resource member input. */
  required?: boolean;
  /** Sort order field on admin ai resource member input. */
  sortOrder?: string | null;
}
