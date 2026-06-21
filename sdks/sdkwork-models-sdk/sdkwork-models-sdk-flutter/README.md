# sdkwork_models Flutter SDK Standard

This package provides the Flutter/Dart implementation of the `sdkwork-models`
catalog loader, validator, and query API.

## Package Name

```text
sdkwork_models
```

## Runtime Targets

- Flutter mobile
- Flutter desktop
- Flutter web
- Dart CLI tools

Model and price lookups use the stable `vendorCode/modelId` catalog key, for
example `openai/gpt-5.5`. `regionCode` remains a loader, filter, deployment,
ranking, and pricing dimension.
`loadCatalog(pathOrUrl)` reads `models/index.json` and then loads the declared
`modelFiles` and `pricingFiles`, so Dart CLI/desktop local paths and HTTP(S)
catalog roots use the same file manifest. `loadBundledCatalog()` resolves
`SDKWORK_MODELS_CATALOG_ROOT` first and then falls back to `data/sdkwork-models`
for monorepo development.

## Required Public API

```dart
SdkworkModels.loadCatalog(pathOrUrl)
SdkworkModels.loadBundledCatalog()
SdkworkModels.loadVendorCatalog(pathOrUrl, vendorCode, regionCode)
validateCatalog(catalog)
listVendors(catalog)
listVendorRegions(catalog)
listModels(catalog, filter)
listAvailableModels(catalog)
findModel(catalog, catalogKey)
findModelByVendorRegion(catalog, vendorCode, regionCode, modelId)
getModelPrices(catalog, catalogKey)
getBestReferencePrice(catalog, catalogKey, meterCode)
listModelsByCapability(catalog, capability)
listModelsByModality(catalog, input, output)
listProtocols(catalog)
findProtocol(catalog, protocolCode)
listProtocolsByVendor(catalog, vendorCode)
listModelsByProtocol(catalog, protocolCode)
listMeters(catalog)
findMeter(catalog, meterCode)
```

`listModels(catalog, filter: {...})` and named filter arguments must support
these filter keys:

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

## Asset Loading

Flutter applications should be able to package the catalog under application
assets:

```yaml
flutter:
  assets:
    - assets/sdkwork-models/models/
```

The SDK must provide asset-aware loading in addition to file and optional remote
loading.

## Decimal Rule

Price and quantity fields remain strings by default. Optional decimal package
integration may be provided, but catalog data must not be coerced to binary
floating-point values.

## Error Model

Validation issues must expose:

- `code`
- `path`
- `message`
- `severity`

## Dependency Boundary

This package must not depend on ClawRouter app/backend SDKs. It is a portable
catalog SDK.

## SDKWork Documentation Contract

Domain: intelligence
Capability: model
Package type: dart-package
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

- `cd apps/sdks/sdkwork-models-sdk/sdkwork-models-sdk-flutter && flutter test`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
