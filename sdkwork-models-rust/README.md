# sdkwork-models Rust SDK Standard

This crate provides the Rust implementation of the `sdkwork-models` catalog
loader, validator, and query API.

## Crate Name

```text
sdkwork-models
```

## Runtime Targets

- Rust 2021 edition or newer
- ClawRouter importer
- CLI tooling
- server and desktop applications

Model and price lookups use the stable `vendorCode/modelId` catalog key, for
example `openai/gpt-5.5`. `regionCode` remains a loader, filter, deployment,
ranking, and pricing dimension for operating market, billing currency, and
legal jurisdiction.
`load_catalog(path_or_url)` reads `models/index.json` and loads the declared
`modelFiles` and `pricingFiles` instead of scanning unpublished directories.
`load_bundled_catalog()` resolves the crate-local catalog snapshot used by the
ClawRouter importer.

## Required Public API

```rust
load_catalog(path_or_url)
load_bundled_catalog()
load_vendor_catalog(path_or_url, vendor_code, region_code)
validate_catalog(&catalog)
list_vendors(&catalog)
list_vendor_regions(&catalog)
list_models(&catalog, filter)
list_available_models(&catalog, filter)
find_model(&catalog, catalog_key)
find_model_by_vendor_region(&catalog, vendor_code, region_code, model_id)
get_model_prices(&catalog, catalog_key)
get_best_reference_price(&catalog, catalog_key, meter_code)
list_models_by_capability(&catalog, capability)
list_models_by_modality(&catalog, input, output)
list_protocols(&catalog)
find_protocol(&catalog, protocol_code)
list_protocols_by_vendor(&catalog, vendor_code)
list_models_by_protocol(&catalog, protocol_code)
list_meters(&catalog)
find_meter(&catalog, meter_code)
```

`list_models(&catalog, filter)` must support these filter fields:

- `vendorCode`
- `regionCode`
- `familyCode`
- `capability`
- `inputModality`
- `outputModality`
- `releaseStage`
- `shelfState`
- `routingState`
- `apiFormat`

`apiFormat` values are protocol codes from `models/protocols.json`. Use the
protocol query helpers to discover protocol metadata, inspect vendor support,
and list models by protocol.

## Decimal Rule

The base model must preserve decimal strings exactly. The crate may expose an
optional `rust_decimal` feature for arithmetic, but binary floating-point values
must not be used for prices.

## Error Model

Errors and validation issues must distinguish:

- parse failures
- schema validation failures
- reference validation failures
- IO failures
- network failures
- unsupported schema versions

Validation issues must expose stable machine-readable codes.

## ClawRouter Boundary

ClawRouter may depend on this crate for import. This crate must not depend on
ClawRouter crates.

## SDKWork Documentation Contract

Domain: intelligence
Capability: model
Package type: rust-crate
Status: standardizing

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

- None declared in `specs/component.spec.json`.

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including deployment and runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `cargo test --manifest-path apps/sdkwork-clawrouter/data/sdkwork-models/sdkwork-models-rust/Cargo.toml`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
