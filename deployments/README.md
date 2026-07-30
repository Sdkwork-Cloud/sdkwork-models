# Deployments

`sdkwork-models` supports **standalone** and **cloud** deployment profiles declared in `specs/topology.spec.json`.

## Runtime Processes

| Profile | Process | Binary |
| --- | --- | --- |
| `standalone.*` | Application ingress | `sdkwork-api-models-standalone-gateway` |
| `cloud.*` | Published API assembly consumed by the remote platform ingress | `sdkwork-api-models-assembly` |

## Bootstrap

1. Configure the shared workspace database via `SDKWORK_DATABASE_*` (or the explicit `SDKWORK_DATABASE_URL` override).
2. Set `SDKWORK_MODELS_CATALOG_ROOT` to the catalog JSON root (defaults to repository `models/`).
3. Enable IAM when required: `SDKWORK_MODELS_IAM_ENABLED=true`.
4. Bind ingress: `SDKWORK_MODELS_APPLICATION_PUBLIC_INGRESS_BIND=127.0.0.1:8080`.
5. Configure CORS allowlist: `SDKWORK_MODELS_CORS_ALLOWED_ORIGINS` (defaults follow `SDKWORK_MODELS_ENVIRONMENT`). Set `SDKWORK_MODELS_CORS_ALLOW_ANY=true` only for local debugging.

## Gateway

Platform ingress selection and rollout live in the platform deployment authority; this application publishes `sdkwork-api-models-assembly`.

Validate:

```powershell
pnpm run gateway:validate:cloud
pnpm run topology:validate
```

## PC Browser Bundle

The catalog explorer ships from `apps/sdkwork-models-pc/`. Admin libraries are composed into Claw Router PC for production operator workflows.

Per-application `deployments/deploy.yaml` will be added when SDKWork Deploy Server publication is enabled for this product.
