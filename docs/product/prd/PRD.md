# SDKWork Models PRD
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

Status: active  
Owner: SDKWork maintainers  
Application: sdkwork-models  
Updated: 2026-07-31

## 1. Background

SDKWork Models is the portable AI model catalog, pricing evidence, and cross-language SDK workspace for the SDKWork platform. It provides authoritative model metadata, admin APIs, and generated SDK families consumed by Cloud Router and standalone deployments.

## 2. Target Users

- Platform operators managing model catalog, mappings, rankings, and AI resources
- Application developers consuming `@sdkwork/models` and `@sdkwork/models-{app,backend}-sdk`
- Release engineers validating catalog evidence and OpenAPI/SDK drift

## 3. Goals

- Authoritative catalog JSON with verification gates
- TTS voice (speaker) catalog with many-to-many model bindings and multi-language SDK query helpers
- Video generation profile catalog (modes, duration tiers, wire mappings) for all video models with DB import sync
- Multi-tenant admin HTTP API with IAM enforcement
- Production resource-group administration with SDKWork offset pagination,
  per-member assignment commands, bounded group size, atomic audit evidence, and
  routing configuration version events
- App read API for catalog consumers (models, vendors, rankings, voices, video profiles)
- Standalone deployable `sdkwork-api-models-standalone-gateway` binary
- Generated TypeScript/Rust/Java/Python/Dart SDK families

## 4. Non-Goals

- LLM inference routing (owned by Cloud Router)
- Site-level admin (owned by Cloud Router)
- Full PC operator shell without composed host (admin libraries require Cloud Router PC commons)

## 5. Success Metrics

- `pnpm run verify` passes on every merge
- OpenAPI authority matches composed host export (`models_openapi_export.mjs --check`)
- Route manifest tests pass for app and backend crates
- Standalone topology can start `sdkwork-api-models-standalone-gateway`
- P0 admin list paths request one server page at a time (`20` by default, `200`
  maximum), never materialize an unbounded catalog in the browser or service, and
  return `items` plus offset `pageInfo`
- Resource groups reject a 513th persisted member, while idempotent single-member
  upsert/delete operations keep the membership, audit record, and routing
  configuration event in one database transaction
- PostgreSQL remains the authoritative server database; an isolated PostgreSQL
  integration environment must prove concurrent mutation, retryable SQLSTATE,
  rollback, and deadlock/serialization handling before commercial release

## 6. Phases

| Phase | Scope | Status |
| --- | --- | --- |
| P0 | Catalog data + validation pipeline | Complete |
| P1 | Backend admin API + IAM | Complete |
| P2 | Standalone gateway + app-api web framework | Complete |
| P3 | Full PC application shell per APP_PC_ARCHITECTURE_SPEC | In progress |
| P4 | Supply-chain and production concurrency evidence | In progress | Enable SBOM/signing policy and retain isolated PostgreSQL contention evidence before commercial release |

## 7. Linked Specs

- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/PAGINATION_SPEC.md`
- `../sdkwork-specs/DATABASE_SPEC.md`
- `../sdkwork-specs/SECURITY_SPEC.md`
- `../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- `docs/standards-alignment.md`
