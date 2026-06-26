# SDKWork Models Technical Architecture
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-06-24

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
  api-server                   → Route composition + standalone binary
  router-catalog-{app,backend}-api → Route manifests + web framework
sdks/                          → Generated @sdkwork/models-* SDK families
apps/sdkwork-models-pc/        → Catalog browser + composed admin libraries
```

## 3. Runtime Topology

| Mode | Ingress | Notes |
| --- | --- | --- |
| Composed | Claw Router backend/app routers | Primary production path |
| Standalone | `sdkwork-models-api-server` | Declared in `specs/topology.spec.json` |
| Cloud | `sdkwork-api-cloud-gateway` + app upstream | `configs/sdkwork-api-cloud-gateway.models.*.toml` |

## 4. Security

- Backend routes: `with_required_permission(...)` on every operation
- App routes: IAM web framework layer + `require_subject: true` for rankings
- `/readyz`: fails closed; no internal error strings in response body
- Production gateway: restricted CORS, upstream readiness checks

## 5. Data

- L2 database module: `database/database.manifest.json`
- Catalog sync from JSON via admin `models.refresh`
- App catalog read uses `JsonPricingCatalog` snapshot in standalone mode

## 6. Verification

```powershell
pnpm run verify
cargo check -p sdkwork-models-api-server
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
