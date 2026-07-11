/** AI resource composition member input. */
export interface AdminAiResourceMemberInput {
  memberResourceCode: string;
  memberRole?: 'included' | 'optional' | 'fallback' | null;
  required?: boolean;
  sortOrder?: string | null;
}
