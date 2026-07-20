# SDKWork Models API contracts

Authoritative HTTP contracts for the `intelligence` domain catalog capability.

| Surface | Path | Authority |
| --- | --- | --- |
| Backend admin | `backend-api/intelligence/openapi.json` | `sdkwork-models-backend-api` |
| App read | `app-api/intelligence/openapi.json` | `sdkwork-models-app-api` |

OpenAPI JSON files are extracted from the composed host authority via `tools/models_openapi_export.mjs` and validated in `pnpm run check`.

Materialize aggregated authorities into `sdks/sdkwork-models-{app,backend}-sdk/openapi/` before SDK generation:

```bash
pnpm run api:materialize:openapi
pnpm run api:materialize:models
pnpm run sdk:generate
pnpm run sdk:build
```

## Backend `models.list` query contract

| Query param | SDK field | Description |
| --- | --- | --- |
| `vendor_id` | `vendorId` | Filter by vendor id |
| `vendor_code` | `vendorCode` | Filter by vendor code |
| `q` | `q` | Case-insensitive search on display name or model id |
| `model_types` | `modelTypes` | Comma-separated admin labels (`Chat`, `Image`, `Embedding`, …) mapped to capability codes server-side |
| `limit` | `limit` | Page size (default 50, max 200) |
| `offset` | `offset` | Zero-based page offset |

Responses include `totalCount` for server-side pagination. Admin consumers must not re-filter or re-slice pages client-side.

The export tool injects `model_types` into owner OpenAPI until ClawRouter route metadata declares it upstream.

Composed hosts such as `sdkwork-clawrouter` must consume generated SDK families and declare dependency surfaces in `specs/dependency-api-surfaces.json`; hosts must not duplicate catalog admin handlers or DDL.
