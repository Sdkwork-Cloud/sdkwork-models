# SDKWork Models PC Component Specs

Local standards index for `@sdkwork/models-pc`.

| Field | Value |
| --- | --- |
| Name | `@sdkwork/models-pc` |
| Root | `apps/sdkwork-models-pc` |
| Domain | `intelligence` |
| Capability | `model` |

## Verification

- `pnpm --filter @sdkwork/models-pc run test`
- `pnpm --filter @sdkwork/models-pc run build`

Catalog files are served from `/__sdkwork_catalog` in dev/preview and copied into `dist/` on build.
