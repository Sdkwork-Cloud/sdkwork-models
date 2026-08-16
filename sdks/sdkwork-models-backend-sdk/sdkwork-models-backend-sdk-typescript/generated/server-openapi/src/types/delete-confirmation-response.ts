/** Fail-closed delete confirmation response. */
export interface DeleteConfirmationResponse {
  code: 0;
  data: unknown & { deleted: boolean; };
  /** Server-owned request correlation id. */
  traceId: string;
}
