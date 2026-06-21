# Tests

Cross-package verification for `sdkwork-models`.

## Current Coverage

- Root `pnpm run check` validates catalog JSON contracts, source freshness, audit gates, and release metadata.
- `sdks/sdkwork-models-sdk/sdkwork-models-sdk-typescript/` owns colocated Node tests for `@sdkwork/models`.
- `sdks/sdkwork-models-sdk/sdkwork-models-sdk-rust/tests/` owns Rust catalog contract tests.
- `apps/sdkwork-models-pc/` owns the PC catalog explorer typecheck/build surface.

## Adding Tests

Place repository-wide fixtures and contract tests here when they span multiple language packages. Keep package-local unit tests beside the owning package.

Verification entrypoint:

```powershell
pnpm run verify
```
