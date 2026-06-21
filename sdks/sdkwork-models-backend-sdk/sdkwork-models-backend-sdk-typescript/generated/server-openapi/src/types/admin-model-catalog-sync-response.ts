import type { AdminAiModelItem } from './admin-ai-model-item';
import type { AdminModelVendorItem } from './admin-model-vendor-item';

/** Admin model catalog sync response schema exposed by Claw Router. */
export interface AdminModelCatalogSyncResponse {
  /** Total accepted standard fact count across meters, vendors, families, models, capabilities, prices, and rankings. */
  acceptedCount: string;
  /** Generated model capability fact count considered by the sync. */
  capabilityCount: string;
  /** Catalog root field on admin model catalog sync response. */
  catalogRoot?: string | null;
  /** sdkwork-models catalogVersion loaded for this sync. */
  catalogVersion: string;
  /** Whether this response represents an observation-only sync that did not mutate model catalog facts. */
  dryRun: boolean;
  /** Selected model family fact count considered by the sync. */
  familyCount: string;
  /** Shared sdkwork-models billing meter fact count considered by the sync. */
  meterCount: string;
  /** Normalized sync mode executed by the backend. */
  mode: 'official_refresh' | 'vendor_refresh' | 'catalog_version_refresh' | 'dry_run';
  /** Selected model definition fact count considered by the sync. */
  modelCount: string;
  /** Current ai model snapshots after sync. */
  models: AdminAiModelItem[];
  /** Expanded pricing fact count considered by the sync. */
  priceCount: string;
  /** Selected ranking snapshot item count considered by the sync. */
  rankingCount: string;
  /** Requested catalog version field on admin model catalog sync response. */
  requestedCatalogVersion?: string | null;
  /** Pricing import snapshot identifier created by the sync. */
  snapshotId?: string;
  /** Normalized catalog source label used for the sync. */
  source: string;
  /** Stable SHA-256 hash of the selected sdkwork-models catalog scope, independent of request id, time, or snapshot id. */
  sourceHash: string;
  /** Model catalog sync-run identifier created by the sync. */
  syncRunId?: string;
  /** Whether the catalog snapshot refresh completed. */
  synced: boolean;
  /** Actual vendor scope covered by the loaded catalog snapshot. */
  vendorCodes: string[];
  /** Selected vendor directory count considered by the sync. */
  vendorCount: string;
  /** Current model vendor snapshots after sync. */
  vendors: AdminModelVendorItem[];
}
