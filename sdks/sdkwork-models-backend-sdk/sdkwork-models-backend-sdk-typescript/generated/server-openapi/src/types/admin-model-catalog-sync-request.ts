/** Request body for refreshing the AI model catalog. */
export interface AdminModelCatalogSyncRequest {
  source?: string | null;
  mode?: 'official_refresh' | 'vendor_refresh' | 'catalog_version_refresh' | 'dry_run';
  vendorCodes?: string[];
  force?: boolean;
  catalogRoot?: string | null;
  catalogVersion?: string | null;
}
