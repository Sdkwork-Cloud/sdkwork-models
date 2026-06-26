# SDKWork Models PRD
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

Status: active  
Owner: SDKWork maintainers  
Application: sdkwork-models  
Updated: 2026-06-24

## 1. Background

SDKWork Models is the portable AI model catalog, pricing evidence, and cross-language SDK workspace for the SDKWork platform. It provides authoritative model metadata, admin APIs, and generated SDK families consumed by Claw Router and standalone deployments.

## 2. Target Users

- Platform operators managing model catalog, mappings, rankings, and AI resources
- Application developers consuming `@sdkwork/models` and `@sdkwork/models-{app,backend}-sdk`
- Release engineers validating catalog evidence and OpenAPI/SDK drift

## 3. Goals

- Authoritative catalog JSON with verification gates
- Multi-tenant admin HTTP API with IAM enforcement
- App read API for authenticated catalog consumers
- Standalone deployable `sdkwork-models-api-server` binary
- Generated TypeScript/Rust/Java/Python/Dart SDK families

## 4. Non-Goals

- LLM inference routing (owned by Claw Router)
- Site-level admin (owned by Claw Router)
- Full PC operator shell without composed host (admin libraries require Claw Router PC commons)

## 5. Success Metrics

- `pnpm run verify` passes on every merge
- OpenAPI authority matches composed host export (`models_openapi_export.mjs --check`)
- Route manifest tests pass for app and backend crates
- Standalone topology can start `sdkwork-models-api-server`

## 6. Phases

| Phase | Scope | Status |
| --- | --- | --- |
| P0 | Catalog data + validation pipeline | Complete |
| P1 | Backend admin API + IAM | Complete |
| P2 | Standalone api-server + app-api web framework | Complete |
| P3 | Full PC application shell per APP_PC_ARCHITECTURE_SPEC | In progress |
| P4 | SBOM/signing release gate | Planned |

## 7. Linked Specs

- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/SECURITY_SPEC.md`
- `../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- `docs/standards-alignment.md`
