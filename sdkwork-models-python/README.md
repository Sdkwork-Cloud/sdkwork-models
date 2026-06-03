# sdkwork-models Python SDK Standard

This package provides the Python implementation of the `sdkwork-models` catalog
loader, validator, and query API.

## Package Name

```text
sdkwork-models
```

## Runtime Targets

- Python 3.10+
- server applications
- CLI tools
- offline data validation jobs

Model and price lookups use the stable `vendorCode/modelId` catalog key, for
example `openai/gpt-5.5`. `regionCode` remains a loader, filter, deployment,
ranking, and pricing dimension for operating market, billing currency, and
legal jurisdiction.
`load_catalog(path_or_url)` reads `models/index.json` and then loads the
declared `modelFiles` and `pricingFiles`, which makes local paths and HTTP(S)
catalog roots behave the same. `load_bundled_catalog()` resolves
`SDKWORK_MODELS_CATALOG_ROOT` first and then falls back to `data/sdkwork-models`
for monorepo development.

## Required Public API

```python
load_catalog(path_or_url)
load_bundled_catalog()
load_vendor_catalog(path_or_url, vendor_code, region_code)
validate_catalog(catalog)
list_vendors(catalog)
list_vendor_regions(catalog)
list_models(catalog, filter=None)
list_available_models(catalog)
find_model(catalog, catalog_key)
find_model_by_vendor_region(catalog, vendor_code, region_code, model_id)
get_model_prices(catalog, catalog_key)
get_best_reference_price(catalog, catalog_key, meter_code)
list_models_by_capability(catalog, capability)
list_models_by_modality(catalog, input=None, output=None)
list_protocols(catalog)
find_protocol(catalog, protocol_code)
list_protocols_by_vendor(catalog, vendor_code)
list_models_by_protocol(catalog, protocol_code)
list_meters(catalog)
find_meter(catalog, meter_code)
```

`list_models(catalog, filter=None)` accepts the standard camelCase filter dict
and also supports Pythonic snake_case keyword aliases:

- `vendorCode` / `vendor_code`
- `regionCode` / `region_code`
- `familyCode` / `family_code`
- `capability`
- `inputModality` / `input_modality`
- `outputModality` / `output_modality`
- `releaseStage` / `release_stage`
- `shelfState` / `shelf_state`
- `routingState` / `routing_state`
- `apiFormat` / `api_format`

`apiFormat` values are protocol codes from `models/protocols.json`. Use the
protocol query helpers to discover protocol metadata, inspect vendor support,
and list models by protocol.

## Decimal Rule

Raw catalog records preserve decimal strings exactly as written. Helper methods
that perform arithmetic must use `decimal.Decimal`.

## Error Model

Validation issues must expose:

- `code`
- `path`
- `message`
- `severity`

Exceptions must distinguish parse, schema, reference, IO, network, and
unsupported schema version failures.

## Network Loading

Network loading must be explicit. The default local loader must not silently
fetch remote catalog data.

## Dependency Boundary

This package must not depend on ClawRouter app/backend SDKs. It is a portable
catalog SDK.
