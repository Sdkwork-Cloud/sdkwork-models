import type { ModelRankingsSnapshot } from './model-rankings-snapshot';

/** Model rankings list result schema exposed by Claw Router. */
export interface ModelRankingsListResult {
  /** Business response code. */
  code: string;
  /** Data field on model rankings list result. */
  data?: ModelRankingsSnapshot;
  /** Human-readable response message. */
  msg?: string;
}
