/** AI resource composition member returned by the management API. */
export interface AdminAiResourceMemberItem {
  parentResourceCode: string;
  memberResourceCode: string;
  memberRole: 'included' | 'optional' | 'fallback';
  required: boolean;
  sortOrder?: string | null;
}
