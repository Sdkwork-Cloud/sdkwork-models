# SDKWork Models Technical Architecture
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-07-04

## 1. Overview

`sdkwork-models` is a **composed product module** that can run standalone or mount into Claw Router. It owns intelligence catalog domain data, HTTP route crates, database module, and SDK families.

## 2. Layering

```
models/ releases/ schemas/     → Catalog JSON authority
tools/                         → Validation, index, OpenAPI export
crates/
  contract-service             → Store traits, domain errors
  catalog-repository-sqlx      → Postgres/SQLite persistence
  catalog-service              → HTTP handlers, application services
  database-bootstrap           → Embedded DDL authority
  database-host                → sdkwork-database lifecycle bootstrap
  standalone-gateway             → Route composition + standalone binary
  router-catalog-{app,backend}-api → Route manifests + web framework
sdks/                          → Generated @sdkwork/models-* SDK families
apps/sdkwork-models-pc/        → Catalog browser + composed admin libraries
```

## 3. Runtime Topology

| Mode | Ingress | Notes |
| --- | --- | --- |
| Composed | Claw Router backend/app routers | Primary production path |
| Standalone | `sdkwork-models-standalone-gateway` | Declared in `specs/topology.spec.json` |
| Cloud | `sdkwork-api-cloud-gateway` + app upstream | `configs/sdkwork-api-cloud-gateway.models.*.toml` |

## 4. Security

- Backend routes: `with_required_permission(...)` on every operation
- App routes: public catalog list endpoints (`models.list`, `modelVendors.list`, `modelRankings.list`, `voices.list`, `modelVoices.list`, `videoProfiles.list`, `modelVideoProfiles.list`) plus IAM web framework layer for protected surfaces
- Backend voice and video profile routes require `intelligence.models.read`
- `/readyz`: fails closed; no internal error strings in response body
- Production gateway: restricted CORS, upstream readiness checks

## 5. Data

- L2 database module: `database/database.manifest.json`
- Catalog sync from JSON via admin `models.refresh` (imports models, pricing, voices, voice bindings, and video generation profiles)
- App catalog read uses in-memory `ModelCatalog` JSON snapshot in standalone mode
- TTS voice catalog: `voices.json` + `model-voices/` per vendor region; persisted to `ai_model_voice` / `ai_model_voice_binding`
- Video generation profiles: `model-video-profiles/{modelId}.json` per video model; persisted to `ai_model_video_profile`

## 6. Verification

```powershell
pnpm run verify
cargo check -p sdkwork-models-standalone-gateway
pnpm run topology:validate
```

See `docs/standards-alignment.md` for the current alignment matrix.

## 8. Architecture Decision Index

- [TECH-client-api-compatibility-schema.md](TECH-client-api-compatibility-schema.md)
- [TECH-client-api-plugins-standard.md](TECH-client-api-plugins-standard.md)
- [TECH-client-api-schema-simple.md](TECH-client-api-schema-simple.md)
- [TECH-client-api-standard-v2.md](TECH-client-api-standard-v2.md)
- [TECH-converter-naming-standard.md](TECH-converter-naming-standard.md)
- [TECH-root-layout.md](TECH-root-layout.md)
- [TECH-standards-alignment.md](TECH-standards-alignment.md)
- [TECH-vendor-model-architecture.md](TECH-vendor-model-architecture.md)
