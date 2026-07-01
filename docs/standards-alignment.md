# SDKWork Models — Standards Alignment

Status: active  
Updated: 2026-06-29  
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
| `sdkwork-specs` | Aligned | `AGENTS.md`, `specs/`, envelope check in `pnpm run check` |
| `sdkwork-web-framework` | Aligned | Route manifests + `WebFrameworkLayer`; handlers use `WebRequestContext` + `finish_success` / `problem_for` |
| `sdkwork-database` | Aligned | `sdkwork-models-database-host` bootstraps lifecycle from `database.manifest.json` |
| `sdkwork-iam-web-adapter` | Aligned | `web_bootstrap.rs` in backend and app route crates |
| `sdkwork-utils` | Aligned | Rust handlers use `SdkWorkApiResponse`; tools/PC use `@sdkwork/utils` |
| `sdkwork-sdk-generator` | Aligned | `pnpm run openapi:export` + `sdk:generate` |
| `sdkwork-discovery` | N/A | No RPC services in this repository |
| `sdkwork-drive` | N/A | No file-upload surfaces in this product |

## HTTP API Envelope

| Requirement | Status | Evidence |
| --- | --- | --- |
| Success `{ code: 0, data, traceId }` | Aligned | `catalog-service/src/api/response.rs` (`ApiResponse`, `finish_success`) |
| Errors `application/problem+json` | Aligned | `problem_for(SdkWorkResultCode, …)` + `ProblemDetail`; request-scoped `traceId` |
| OpenAPI authority | Aligned | `apis/*/intelligence/openapi.json` migrated via `migrateOpenApiDocument` |
| Envelope gate | Aligned | `check-api-response-envelope.mjs` in `pnpm run check` |

## Production Readiness

| Area | Status | Notes |
| --- | --- | --- |
| Catalog JSON authority | Aligned | `pnpm run check` validates index, schema, freshness, audit, release gate |
| Backend admin API | Aligned | IAM permissions on all backend routes; server-side pagination |
| App read API | Aligned | App catalog list routes are public (`RouteAuth::Public`) |
| Standalone runtime | Aligned | `sdkwork-models-standalone-gateway` binary; topology `applicationServer.binary` matches |
| Readiness probe | Aligned | `/healthz` and `/readyz` are infra probes (not business API envelope); `/readyz` probes DB |
| Gateway production template | Aligned | Restricted CORS via `SDKWORK_MODELS_CORS_ALLOWED_ORIGINS`; cloud gateway configs validated |
| CI dependency closure | Aligned | `.github/workflows/verify.yml` checks out platform crates |
| Observability | Aligned | Request-scoped `traceId` on success and error via `WebRequestContext`; structured tracing on admin model list |
| Supply-chain SBOM | Deferred | `security.sbomRequired: false` in `sdkwork.workflow.json` until release gate enables evidence |
| PC browser catalog | Aligned | `apps/sdkwork-models-pc/` standalone catalog explorer via `@sdkwork/models` |
| PC admin UI | Composed | Admin packages (`sdkwork-models-pc-admin-*`) mount in Claw Router host per `APP_PC_ARCHITECTURE_SPEC` |

## Composed Host Integration

Claw Router mounts catalog routes locally. Admin PC packages (`sdkwork-models-pc-admin-*`) are **composed workspaces** requiring `@sdkwork/clawrouter-pc-commons` at runtime.

Standalone installs use `.npmrc` (`auto-install-peers=false`) so peer packages resolve from the composed host.

## Verification

```powershell
pnpm install
pnpm run route-manifest:check
pnpm run db:validate
pnpm run topology:validate
cargo check -p sdkwork-models-standalone-gateway
pnpm run verify
```

Expected: route manifests match handlers, database module validates, Rust workspace compiles, catalog checks pass, API envelope check passes, and `pnpm run verify` exits 0.
