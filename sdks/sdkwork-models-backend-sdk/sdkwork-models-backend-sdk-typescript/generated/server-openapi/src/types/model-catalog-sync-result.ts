/** Catalog sync result returned by models.refresh. */
export interface ModelCatalogSyncResult {
  synced: boolean;
  source: string;
  mode: string;
  dryRun: boolean;
  catalogVersion: string;
  requestedCatalogVersion?: string | null;
  catalogRoot?: string | null;
  vendorCodes: string[];
  sourceHash: string;
  meterCount: number;
  vendorCount: number;
  familyCount: number;
  modelCount: number;
  capabilityCount: number;
  priceCount: number;
  rankingCount: number;
  voiceCount: number;
  voiceBindingCount: number;
  videoProfileCount: number;
  acceptedCount: number;
  snapshotId?: string | null;
  syncRunId?: string | null;
  vendors: Record<string, unknown>[];
  models: Record<string, unknown>[];
}
