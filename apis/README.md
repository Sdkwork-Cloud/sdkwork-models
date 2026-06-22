# SDKWork Models API contracts

Authoritative HTTP contracts for the `intelligence` domain catalog capability.

| Surface | Path | Authority |
| --- | --- | --- |
| Backend admin | `backend-api/intelligence/openapi.json` | `sdkwork-models-backend-api` |
| App read | `app-api/intelligence/openapi.json` | `sdkwork-models-app-api` |

OpenAPI JSON files are extracted from the composed host authority via `tools/models_openapi_export.mjs` and validated in `pnpm run check`.

Materialize aggregated authorities into `sdks/sdkwork-models-{app,backend}-sdk/openapi/` before SDK generation:

```bash
pnpm run openapi:export
pnpm run openapi:materialize
pnpm run sdk:generate
pnpm run sdk:build
```


Composed hosts such as `sdkwork-clawrouter` must consume generated SDK families and declare dependency surfaces in `specs/dependency-api-surfaces.json`; hosts must not duplicate catalog admin handlers or DDL.
