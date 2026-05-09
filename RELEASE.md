# sdkwork-models Release Guide

## Repository Setup

Initialize the standalone catalog repository from this directory:

```powershell
git init
git add .
git commit -m "first commit"
git branch -M main
git remote add origin https://github.com/Sdkwork-Cloud/sdkwork-models.git
git push -u origin main
```

Do not run the push command from restricted automation. Run it from a release
host or maintainer workstation with network access and GitHub credentials.

## ClawRouter Submodule Setup

ClawRouter should mount this repository at:

```text
data/sdkwork-models
```

Add or refresh the submodule from the ClawRouter application root:

```powershell
git submodule add https://github.com/Sdkwork-Cloud/sdkwork-models.git data/sdkwork-models
git submodule update --init --recursive
```

To update ClawRouter to a newer catalog:

```powershell
cd data/sdkwork-models
git fetch origin
git checkout catalog-v2026.05.08.1
cd ..\..
git add data/sdkwork-models
git commit -m "update sdkwork models catalog"
```

Use the actual release tag for the target catalog version.

## Required Local Checks

Run these commands before tagging or publishing:

```powershell
node tools\build-index.mjs --check
node tools\validate-catalog.mjs
node tools\freshness-report.mjs --max-age-policy catalog-freshness-policy.json --as-of 2026-05-08
node tools\catalog-audit.mjs --as-of 2026-05-08
node tools\release-catalog.mjs --check --as-of 2026-05-08
cargo test --manifest-path sdkwork-models-rust\Cargo.toml --offline
```

The source evidence release gate is mandatory. `sources/vendor-sources.json`,
`sources/official-model-snapshots.json`, and
`sources/official-verification-policy.json` must match the release
`schemaVersion` and `catalogVersion`. `sources/official-verification-policy.json`
must satisfy `schemas/official-verification-policy.schema.json`; every
`requiredVerifiedVendorRegions` entry must resolve to an `official_verified`
`vendorCode/regionCode` with an independent official snapshot. The relationship
is bidirectional: every source declaration with `verificationStatus:
"official_verified"` must also be present in `requiredVerifiedVendorRegions`.

From ClawRouter, also run:

```powershell
cargo test -p sdkwork-claw-product --test database_installer --offline
```

## Versioning

Schema versions use semantic versioning:

```text
1.0.0
```

Catalog versions use date release numbering:

```text
YYYY.MM.DD.N
```

Git tags must use:

```text
catalog-vYYYY.MM.DD.N
```

## Release Evidence

Each release must include:

- regenerated `models/index.json`
- regenerated `models/vendors.json`
- release metadata under `releases/<catalogVersion>.json`
- source evidence checksums for `sources/vendor-sources.json`,
  `sources/official-model-snapshots.json`, and
  `sources/official-verification-policy.json`
- per `vendorCode/regionCode` official snapshot hashes under
  `sourceEvidenceSha256.officialSnapshotHashes`
- source audit metadata including `requiredVerifiedRegionCount`,
  `requiredVerifiedRegions`, `officialVerifiedSourceRegionCount`, and
  `officialVerifiedSourceRegions`
- freshness report with no unwaived error-level stale sources
- diff metadata for changed vendors when a previous release is available
- updated `CHANGELOG.md`

