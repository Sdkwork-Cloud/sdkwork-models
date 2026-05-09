# Catalog Releases

Each catalog release stores machine-readable metadata for the exact
`catalogVersion` in `sdkwork-models.json`.

Release files must include:

- `catalogVersion`
- index checksum
- source evidence checksums for `sources/vendor-sources.json`,
  `sources/official-model-snapshots.json`, and
  `sources/official-verification-policy.json`
- per `vendorCode/regionCode` official snapshot hashes under
  `sourceEvidenceSha256.officialSnapshotHashes`
- validation summary
- freshness summary
- source audit summary, including `requiredVerifiedRegionCount`,
  `requiredVerifiedRegions`, `officialVerifiedSourceRegionCount`, and
  `officialVerifiedSourceRegions`
- vendor change summary

Release checks must fail when the generated index drifts, validation has
errors, source evidence files drift from the release metadata, source audit has
errors, or error-level source freshness rules are stale without an unexpired
waiver.

`sources/official-verification-policy.json` must satisfy
`schemas/official-verification-policy.schema.json`. Its
`requiredVerifiedVendorRegions` list is a release gate: every listed
`vendorCode/regionCode` must be present in the catalog, declared in
`sources/vendor-sources.json`, marked `official_verified`, and backed by an
independent official snapshot before the release metadata can be considered
current. The gate is bidirectional: every `official_verified` vendor-region in
`sources/vendor-sources.json` must also be present in
`requiredVerifiedVendorRegions`.
