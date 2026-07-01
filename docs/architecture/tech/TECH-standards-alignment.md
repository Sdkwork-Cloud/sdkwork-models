# SDKWork Standards Alignment

Owner: SDKWork maintainers  
Updated: 2026-06-29

This document records how `sdkwork-models` aligns with `sdkwork-specs` and the commerce/appstore composed-module reference.

## Repository Identity

| Item | Value |
| --- | --- |
| Application key | `sdkwork-models` |
| Component | `@sdkwork/models-catalog` |
| Domain | `intelligence` |
| Capability | `catalog` |
| Application manifest | `sdkwork.app.config.json` |
| PC application root | `apps/sdkwork-models-pc/` |
| HTTP composition | `crates/sdkwork-models-standalone-gateway/` |
| Route crates | `crates/sdkwork-routes-catalog-{app,backend}-api/` |
| Database module | `database/` (`sdkwork.database.module`) |
| SDK families | `sdks/sdkwork-models-{app,backend}-sdk/` |

## Framework Integration Matrix

| Framework | Required? | Status | Notes |
| --- | --- | --- | --- |
| `sdkwork-specs` | Yes | Aligned | `AGENTS.md`, `specs/`, `docs/` |
| `sdkwork-utils` | Yes | Aligned | `catalog_time` + handler envelopes use `sdkwork-utils-rust`; tools/PC use `@sdkwork/utils` |
| `sdkwork-web-framework` | Yes | Aligned | Route manifests + IAM web adapter; handlers inject `WebRequestContext` and finish through `finish_success` / `problem_for(SdkWorkResultCode)` |
| `sdkwork-database` | Yes | Aligned | L2 module with postgres/sqlite baselines, migrations/, seeds/ |
| `sdkwork-sdk-generator` | Yes | Aligned | `pnpm run openapi:export` + `sdk:generate` |
| `sdkwork-discovery` | No | Deferred | No RPC services in this repository |
| `sdkwork-drive` | No | N/A | No file-upload surfaces |

## HTTP API Envelope

All app-api and backend-api operations follow `API_SPEC.md` section 15:

- Success: `SdkWorkApiResponse` with numeric `code: 0`, typed `data`, and `traceId`
- Errors: HTTP 4xx/5xx with `application/problem+json` (`ProblemDetail`)
- OpenAPI export applies `migrateOpenApiDocument` before materialization
- `pnpm run check` runs `check-api-response-envelope.mjs`

## Security

- Application ingress CORS is allowlist-driven via `SDKWORK_MODELS_CORS_ALLOWED_ORIGINS` (see `application_cors_layer` in standalone gateway).
- Production defaults: `https://models.sdkwork.com`, `https://admin.sdkwork.com`.
- Development defaults: local Vite/API origins on `localhost` / `127.0.0.1`.
- Emergency local-only override: `SDKWORK_MODELS_CORS_ALLOW_ANY=true` (not for production).

## Composed Host Integration

Claw Router (`sdkwork-clawrouter`) mounts catalog routes locally and declares dependency surfaces in composed host specs:

| Surface | SDK family | Mount mode |
| --- | --- | --- |
| Backend catalog + resources | `@sdkwork/models-backend-sdk` | `composed-local-mount` |
| App catalog read | `@sdkwork/models-app-sdk` | `composed-local-mount` |

Gateway-owned surfaces remain in Claw Router:

| Surface | Owner |
| --- | --- |
| Site admin (`/backend/v3/api/sites/*`) | `sdkwork-clawrouter` |
| Pricing catalog snapshot port | `sdkwork-clawrouter` host |

## Directory Dictionary

| Path | Purpose |
| --- | --- |
| `apis/` | Authoritative OpenAPI for intelligence catalog app/backend |
| `crates/` | Service, repository, route, standalone-gateway, database-bootstrap |
| `database/` | Module manifest, baselines, migrations, seeds, drift policy |
| `sdks/` | Generated `@sdkwork/models-{app,backend}-sdk` families |
| `apps/sdkwork-models-pc/` | Catalog browser + admin-catalog packages |
| `models/`, `releases/`, `schemas/` | Catalog JSON authority and evidence |

## Script Surface

| Command | Purpose |
| --- | --- |
| `pnpm dev` | Start `apps/sdkwork-models-pc` browser explorer |
| `pnpm build` | Regenerate catalog indexes, build SDK and PC app |
| `pnpm test` | TypeScript SDK, PC typecheck, and Rust workspace tests |
| `pnpm test:rust` | Rust workspace tests |
| `pnpm check` | Catalog validation, OpenAPI drift, API envelope gate |
| `pnpm verify` | `check` plus tests |
| `pnpm openapi:export` | Extract owner-only OpenAPI from composed host authority |
| `pnpm openapi:materialize` | Copy OpenAPI into SDK family inputs |
| `pnpm sdk:generate` | Generate TypeScript SDK transport layers |
| `pnpm sdk:build` | Build composed `@sdkwork/models-*-sdk` packages |
| `pnpm route-manifest:check` | Route manifest contract tests |
| `pnpm db:validate` | Database framework standard check |

## Verification

```powershell
pnpm install
pnpm run route-manifest:check
pnpm run db:validate
cargo check --workspace
pnpm run verify
```

From composed host (`sdkwork-clawrouter`):

```powershell
cargo check -p sdkwork-routes-clawrouter-backend-api -p sdkwork-clawrouter-router-service
pnpm exec tsx admin-model-runtime.test.ts
```

Expected result: route manifests match handlers, database module validates, Rust workspace compiles, catalog checks pass, API envelope check passes, PC admin runtime tests pass.
