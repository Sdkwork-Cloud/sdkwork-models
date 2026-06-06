# sdkwork-models Java SDK Standard

This package provides the Java implementation of the `sdkwork-models` catalog
loader, validator, and query API.

## Maven Coordinates

```text
com.sdkwork.models:sdkwork-models
```

## Runtime Targets

- Java 21+
- Spring applications
- command line tools
- embedded server runtimes

The core package must not require Spring. Spring integration may live in a
separate optional module later.

Model and price lookups use the stable `vendorCode/modelId` catalog key, for
example `openai/gpt-5.5`. `regionCode` remains a loader, filter, deployment,
ranking, and pricing dimension.
`SdkworkModels.loadCatalog(Path)` and `SdkworkModels.loadCatalog(URI)` read
`models/index.json` and load the declared `modelFiles` and `pricingFiles`.
`SdkworkModels.loadBundledCatalog()` resolves `sdkwork.models.catalogRoot` or
`SDKWORK_MODELS_CATALOG_ROOT` first and then falls back to `data/sdkwork-models`
for monorepo development.

## Required Public API

```java
SdkworkModels.loadCatalog(Path path)
SdkworkModels.loadCatalog(URI uri)
SdkworkModels.loadBundledCatalog()
SdkworkModels.loadVendorCatalog(Path path, String vendorCode, String regionCode)
ModelCatalogValidator.validate(ModelCatalog catalog)
SdkworkModels.listVendors(ModelCatalog catalog)
SdkworkModels.listVendorRegions(ModelCatalog catalog)
SdkworkModels.listModels(ModelCatalog catalog)
SdkworkModels.listModels(ModelCatalog catalog, Map<String, String> filter)
SdkworkModels.listAvailableModels(ModelCatalog catalog)
SdkworkModels.listAvailableModels(ModelCatalog catalog, Map<String, String> filter)
SdkworkModels.findModel(ModelCatalog catalog, String catalogKey)
SdkworkModels.findModelByVendorRegion(ModelCatalog catalog, String vendorCode, String regionCode, String modelId)
SdkworkModels.getModelPrices(ModelCatalog catalog, String catalogKey)
SdkworkModels.getBestReferencePrice(ModelCatalog catalog, String catalogKey, String meterCode)
SdkworkModels.listModelsByCapability(ModelCatalog catalog, String capability)
SdkworkModels.listModelsByModality(ModelCatalog catalog, String input, String output)
SdkworkModels.listProtocols(ModelCatalog catalog)
SdkworkModels.findProtocol(ModelCatalog catalog, String protocolCode)
SdkworkModels.listProtocolsByVendor(ModelCatalog catalog, String vendorCode)
SdkworkModels.listModelsByProtocol(ModelCatalog catalog, String protocolCode)
SdkworkModels.listMeters(ModelCatalog catalog)
SdkworkModels.findMeter(ModelCatalog catalog, String meterCode)
```

`SdkworkModels.listModels(ModelCatalog catalog, Map<String, String> filter)`
must support these filter keys:

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

Prices and quantities use `java.math.BigDecimal` for arithmetic and preserve the
original decimal string for serialization.

## Error Model

Validation issues must expose:

- `code`
- `path`
- `message`
- `severity`

Exceptions must distinguish parse, schema, reference, IO, network, and
unsupported schema version failures.

## Compatibility Rule

Unknown optional JSON fields must be preserved in an extension map so newer
catalogs can be inspected by older SDKs when the schema version allows it.

## Dependency Boundary

This package must not depend on ClawRouter app/backend SDKs. It is a portable
catalog SDK.

## SDKWork Documentation Contract

Domain: intelligence
Capability: model
Package type: java-module
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

- `mvn test -f apps/sdkwork-claw-router/data/sdkwork-models/sdkwork-models-java/pom.xml`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
