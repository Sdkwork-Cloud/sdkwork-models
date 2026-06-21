/** Field-level validation problem detail. */
export interface FieldError {
  /** Machine-readable field validation code. */
  code?: string;
  /** Problem field path. */
  field?: string;
  /** Human-readable field validation message. */
  message?: string;
}
