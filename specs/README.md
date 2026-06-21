# SDKWork Models Catalog Component Specs

This directory is the local standards index for `@sdkwork/models-catalog`.

Root SDKWork standards remain authoritative. Local component specs can narrow or document this component, but they must not contradict [the root standards](../sdkwork-specs/README.md).

## Component

| Field | Value |
| --- | --- |
| Name | `@sdkwork/models-catalog` |
| Type | `node-package` |
| Root | `.` |
| Domain | `intelligence` |
| Capability | `model` |
| Languages | `javascript`, `typescript`, `python`, `java`, `rust`, `dart` |
| Status | `standardizing` |

## Contract Manifest

- [component.spec.json](./component.spec.json) is the machine-readable component contract.
- Consumers integrate through `@sdkwork/models` language SDKs, immutable catalog JSON releases, or the documented catalog tools.
- Generated SDK language outputs live under `sdkwork-models-<language>/` and must preserve the cross-language catalog query contract.

## Canonical Specs

| Spec | Applies Because |
| --- | --- |
| [COMPONENT_SPEC.md](../sdkwork-specs/COMPONENT_SPEC.md) | Local component specs directory and manifest rules. |
| [SDKWORK_WORKSPACE_SPEC.md](../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md) | Repository root directory dictionary and `.sdkwork/` workspace metadata. |
| [PNPM_SCRIPT_SPEC.md](../sdkwork-specs/PNPM_SCRIPT_SPEC.md) | Uniform public `package.json` script names. |
| [DEPENDENCY_MANAGEMENT_SPEC.md](../sdkwork-specs/DEPENDENCY_MANAGEMENT_SPEC.md) | `@sdkwork/utils` workspace dependency and sibling repository paths. |
| [DOMAIN_SPEC.md](../sdkwork-specs/DOMAIN_SPEC.md) | Canonical domain ownership and naming. |
| [MODULE_SPEC.md](../sdkwork-specs/MODULE_SPEC.md) | Reusable package contract and dependency direction. |
| [SDK_SPEC.md](../sdkwork-specs/SDK_SPEC.md) | SDK generation and SDK integration rules. |
| [RELEASE_SPEC.md](../sdkwork-specs/RELEASE_SPEC.md) | Catalog release versioning and release evidence. |
| [TEST_SPEC.md](../sdkwork-specs/TEST_SPEC.md) | Contract, SDK, and catalog verification rules. |
| [CODE_STYLE_SPEC.md](../sdkwork-specs/CODE_STYLE_SPEC.md) | Authored source structure and generated code boundaries. |
| [NAMING_SPEC.md](../sdkwork-specs/NAMING_SPEC.md) | Canonical SDKWork naming rules. |
| [TYPESCRIPT_CODE_SPEC.md](../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md) | TypeScript and Node package rules. |
| [DOCUMENTATION_SPEC.md](../sdkwork-specs/DOCUMENTATION_SPEC.md) | Module README, examples, ADR, changelog, and runbook rules. |
| [GOVERNANCE_SPEC.md](../sdkwork-specs/GOVERNANCE_SPEC.md) | Standard ownership, exception, compatibility, and migration rules. |

## Public Exports

- `@sdkwork/models` (TypeScript package in `sdks/sdkwork-models-sdk/sdkwork-models-sdk-typescript/`)
- Equivalent packages in `sdks/sdkwork-models-sdk/sdkwork-models-sdk-python/`, `sdkwork-models-sdk-java/`, `sdkwork-models-sdk-rust/`, and `sdkwork-models-sdk-flutter/`
- PC application packages under `apps/sdkwork-models-pc/`

## SDK Clients

- No generated HTTP/RPC SDK client is declared at this component boundary.

## Local Extension Specs

- [docs/standards-alignment.md](../docs/standards-alignment.md) documents framework applicability for this data-catalog repository.

## Verification

- `pnpm run check`
- `pnpm run verify`
