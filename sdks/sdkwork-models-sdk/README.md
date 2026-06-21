# SDKWork Models SDK Workspace

`sdkwork-models-sdk` is the provider-standard file catalog SDK workspace for SDKWork.

It is not an OpenAPI-generated HTTP SDK family. It owns cross-language catalog loading, validation,
and query behavior over versioned JSON catalog artifacts in the repository root `models/`,
`sources/`, and `releases/` directories.

## Scope

This workspace owns:

- `loadCatalog`, `loadBundledCatalog`, and `loadVendorCatalog`
- `validateCatalog` and catalog query helpers
- decimal-safe pricing contracts and catalog key semantics
- language workspaces under `sdkwork-models-sdk-<language>/`

This workspace does not own:

- HTTP `*-api` route crates or OpenAPI authorities
- application databases or migration lifecycle
- provider routing overlays (see `overlays/` at repository root)
- RPC catalog services

## Public Packages

| Language | Workspace | Public package |
| --- | --- | --- |
| TypeScript | `sdkwork-models-sdk-typescript` | `@sdkwork/models` |
| Python | `sdkwork-models-sdk-python` | `sdkwork-models` |
| Java | `sdkwork-models-sdk-java` | `com.sdkwork.models:sdkwork-models` |
| Rust | `sdkwork-models-sdk-rust` | `sdkwork-models` |
| Flutter | `sdkwork-models-sdk-flutter` | `sdkwork_models` |

## Catalog Root

Set `SDKWORK_MODELS_CATALOG_ROOT` to this repository root when loading from a local checkout.
When unset, SDKs fall back to the ClawRouter submodule mount `data/sdkwork-models`.

## Verification

```powershell
pnpm --filter @sdkwork/models run test
cargo test --manifest-path sdks/sdkwork-models-sdk/sdkwork-models-sdk-rust/Cargo.toml
```
