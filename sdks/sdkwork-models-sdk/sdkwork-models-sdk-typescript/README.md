# @sdkwork/models TypeScript SDK Standard

This package provides the TypeScript implementation of the `sdkwork-models`
catalog loader, validator, and query API.

## Package Name

```text
@sdkwork/models
```

## Runtime Targets

- Node.js 18+
- modern browsers
- Vite, Next.js, Electron, and desktop web shells

Filesystem loading is Node-only. Browser applications should use bundled data,
asset URLs, or remote immutable catalog URLs.
`loadCatalog(pathOrUrl)` reads `models/index.json` and then loads each declared
`modelFiles` and `pricingFiles` entry, so local paths and remote HTTP(S)
catalog roots use the same file manifest. `loadBundledCatalog()` resolves
`SDKWORK_MODELS_CATALOG_ROOT` first and then falls back to `data/sdkwork-models`
for monorepo development.

Model and price lookups use the stable `vendorCode/modelId` catalog key, for
example `openai/gpt-5.5`. `regionCode` remains a loader, filter, deployment,
ranking, and pricing dimension.

## Required Public API

```ts
loadCatalog(pathOrUrl)
loadBundledCatalog()
loadVendorCatalog(pathOrUrl, vendorCode, regionCode)
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

`listModels(catalog, filter)` must support these filter keys:

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

Price and quantity fields remain strings in the base API. The SDK may expose an
optional decimal adapter hook, but it must never coerce catalog prices to
JavaScript `number` by default.

## Entry Points

Recommended entry points:

```text
@sdkwork/models
@sdkwork/models/node
@sdkwork/models/browser
@sdkwork/models/bundled
```

The default entry point must avoid importing Node filesystem modules so browser
bundlers can tree-shake safely.

## Error Model

Errors and validation issues must expose:

- `code`
- `path`
- `message`
- `severity`

Human-readable messages are not enough for application integration.

## Dependency Boundary

This package must not depend on CloudRouter app/backend SDKs. It is a portable
catalog SDK.

## npm Release

Build and test the package before publishing:

```powershell
npm.cmd install
npm.cmd test
npm.cmd run pack:dry-run
```

The published npm package is intentionally limited to:

- `dist/`
- `README.md`
- `LICENSE`
- `package.json`

Publish from this directory with an npm account that has access to the
`@sdkwork` scope:

```powershell
npm.cmd login
npm.cmd run release:publish
```

For CI, set `NPM_TOKEN` and run:

```powershell
npm.cmd install
npm.cmd publish
```

`prepublishOnly` runs the build, tests, and package dry run before the publish
request is sent to the npm registry.

## SDKWork Documentation Contract

Domain: intelligence
Capability: model
Package type: node-package
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

- `pnpm --filter @sdkwork/models test`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
