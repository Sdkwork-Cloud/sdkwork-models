/** Admin model catalog sync request schema exposed by Claw Router. */
export interface AdminModelCatalogSyncRequest {
  /** Optional sdkwork-models project root. Overrides SDKWORK_MODELS_CATALOG_ROOT for this sync. */
  catalogRoot?: string;
  /** Optional catalogVersion pin; sync fails if the loaded catalog differs. */
  catalogVersion?: string;
  /** Whether to force refresh even when the selected catalog version is already installed. */
  force?: boolean;
  /** Refresh mode. dry_run previews without mutating catalog tables. */
  mode?: 'official_refresh' | 'vendor_refresh' | 'catalog_version_refresh' | 'dry_run';
  /** Optional catalog source label; defaults to sdkwork_models. */
  source?: string;
  /** Optional vendor directory codes to refresh. */
  vendorCodes?: string[];
}
