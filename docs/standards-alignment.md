# SDKWork Models — Standards Alignment

Status: active  
Updated: 2026-06-24  
Authority: `../sdkwork-specs/README.md`

This document records the **verified** alignment posture for `sdkwork-models`. It must match evidence from `pnpm run verify`, `cargo test --workspace`, and topology validation.

## Repository Identity

| Item | Value |
| --- | --- |
| Application key | `sdkwork-models` |
| Component | `@sdkwork/models-catalog` |
| Domain / capability | `intelligence` / `catalog` |
| Archetype | `composed-product-module` with standalone HTTP server |
| PC root | `apps/sdkwork-models-pc/` |
| HTTP composition | `crates/sdkwork-models-standalone-gateway/` |
| Standalone binary | `sdkwork-models-standalone-gateway` |
| Database module | `database/` (`sdkwork.database.module`) |

## Framework Integration

| Framework | Status | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Aligned | Backend + app route crates integrate IAM resolver and route manifests |
| `sdkwork-database` | Aligned | `sdkwork-models-database-host` bootstraps lifecycle from `database.manifest.json` |
| `sdkwork-iam-web-adapter` | Aligned | `web_bootstrap.rs` in backend and app route crates |
| `sdkwork-sdk-generator` | Aligned | `pnpm run openapi:export` + `sdk:generate` |
| `sdkwork-discovery` | N/A | No RPC services in this repository |

## Production Readiness

| Area | Status | Notes |
| --- | --- | --- |
| Catalog JSON authority | Aligned | `pnpm run check` validates index, schema, freshness, audit, release gate |
| Backend admin API | Aligned | IAM permissions on all backend routes; server-side pagination |
| App read API | Aligned | App route manifest + `require_subject: true` for rankings |
| Standalone runtime | Aligned | `sdkwork-models-standalone-gateway` binary; topology `applicationServer.binary` matches |
| Readiness probe | Aligned | `/readyz` probes DB; errors are logged server-side only |
| Gateway production template | Aligned | Restricted CORS, upstream readiness checks, metrics/tracing enabled |
| CI dependency closure | Aligned | `.github/workflows/verify.yml` checks out `sdkwork-iam` |
| Observability | Partial | Structured tracing on admin list; `PlusApiResult.traceId` field added; full handler coverage pending |
| Supply-chain SBOM | Planned | Enable in `sdkwork.workflow.json` when release gate requires |
| PC full application shell | Partial | Catalog explorer + composed admin libraries; full `APP_PC_ARCHITECTURE_SPEC` shell pending |

## Composed Host Integration

Claw Router mounts catalog routes locally. Admin PC packages (`sdkwork-models-pc-admin-*`) are **composed workspaces** requiring `@sdkwork/clawrouter-pc-commons` at runtime.

Standalone installs use `.npmrc` (`auto-install-peers=false`) so peer packages resolve from the composed host.

## Verification

```powershell
pnpm install
pnpm run route-manifest:check
pnpm run db:validate
cargo check -p sdkwork-models-standalone-gateway
pnpm run verify
```

Expected: route manifests match handlers, database module validates, Rust workspace compiles, catalog checks pass, and `pnpm run verify` exits 0.
