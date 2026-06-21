/** Admin ai resource member item schema exposed by Claw Router. */
export interface AdminAiResourceMemberItem {
  /** Member resource code field on admin ai resource member item. */
  memberResourceCode: string;
  /** Member role field on admin ai resource member item. */
  memberRole: 'included' | 'optional' | 'fallback';
  /** Parent resource code field on admin ai resource member item. */
  parentResourceCode: string;
  /** Required field on admin ai resource member item. */
  required: boolean;
  /** Sort order field on admin ai resource member item. */
  sortOrder?: string;
}
