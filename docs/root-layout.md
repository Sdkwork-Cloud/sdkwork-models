# SDKWork Models Repository Layout

This repository is the SDKWork application repository for the portable AI model catalog product.

It owns versioned model vendor JSON, pricing evidence, catalog indexes, release metadata, the
`sdkwork-models-sdk` language SDK family, and the PC browser catalog explorer application.

## Standard Project Root

| Directory | Purpose |
| --- | --- |
| `apis/` | Reserved for future owned HTTP/RPC contracts |
| `apps/` | Runnable application roots; `apps/sdkwork-models-pc/` is the PC browser explorer |
| `crates/` | Reserved for future Rust route/service crates |
| `sdks/` | SDK families; `sdks/sdkwork-models-sdk/` owns all catalog language SDKs |
| `jobs/` | Reserved for scheduled catalog jobs |
| `tools/` | Catalog validation, index generation, audit, and release tooling |
| `plugins/` | Reserved for runtime plugins |
| `examples/` | Runnable SDK and application examples |
| `configs/` | Repository-wide safe config templates |
| `deployments/` | Packaging and deployment descriptors |
| `scripts/` | Thin command entrypoints |
| `docs/` | Architecture, alignment, and runbooks |
| `tests/` | Cross-package verification fixtures |
| `models/` | Vendor-region model facts, pricing files, generated indexes |
| `schemas/` | JSON Schema contracts |
| `sources/` | Official source evidence and verification policy |
| `overlays/` | Consumer-specific overlays such as ClawRouter routing |
| `releases/` | Immutable catalog release manifests |
| `specs/` | Repository component contract |
| `.sdkwork/` | Source-controlled workspace metadata |

## Application Roots

| Path | Standard |
| --- | --- |
| `apps/sdkwork-models-pc/` | `APP_PC_ARCHITECTURE_SPEC.md` PC browser catalog explorer |

## SDK Families

| Path | Standard |
| --- | --- |
| `sdks/sdkwork-models-sdk/` | Provider-standard file catalog SDK per `SDK_WORKSPACE_GENERATION_SPEC.md` non-OpenAPI exception |

## Related Standards

- [standards-alignment.md](./standards-alignment.md)
- [../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md](../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md)
- [../specs/component.spec.json](../specs/component.spec.json)
