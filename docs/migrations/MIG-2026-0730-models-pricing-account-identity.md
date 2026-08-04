# MIG-2026-0730 Models Pricing Account Identity

```yaml
id: MIG-2026-0730-models-pricing-account
owner: sdkwork-models-platform
status: completed
requirement: SDKWORK unified shared-schema startup compatibility
type: database
scope:
  producers:
    - sdkwork-models
    - database/contract/schema.yaml
    - docs/schema-registry/sdkwork-models.tables.yaml
  consumers:
    - sdkwork-models-catalog-service
    - sdkwork-cloudrouter-router-service
compatibility_window:
  starts_at: 2026-07-30
  ends_at: 2026-07-30
strategy: cutover
rollback:
  supported: true
  steps:
    - Stop consumers before database recovery or forward-fix.
    - Keep account_id as the authoritative identity because previous sdkwork-models code does not read the retired channel_id column.
    - Deploy the prior compatible application artifact against the forward schema if application rollback is required.
verification:
  - pnpm run db:materialize:contract
  - pnpm run db:validate
  - pnpm run db:plan
  - PostgreSQL legacy-shape upgrade smoke with a fallback-schema decoy table
  - PostgreSQL current-shape idempotency smoke
  - PostgreSQL ambiguous dual-column rejection smoke
  - pnpm run db:migrate
  - pnpm run db:status
```

`ai_model_pricing.channel_id` was a stale physical name for an upstream supplier
account identity. The shared models domain contract and its active Cloud Router
consumer already use `supplier_code` plus `account_id`; a workspace source scan
found no authored pricing consumer that requires `ai_model_pricing.channel_id`.
The migration therefore performs a metadata-only rename, preserves every stored
value, replaces the legacy provider/channel index with the canonical
supplier/account index, and verifies the exact resulting shape before commit.

The migration fails closed when both names exist, when neither name exists, or
when the target index is incompatible. It scopes discovery and DDL to
`current_schema()` so a same-named object in a fallback schema cannot be changed.
No row backfill or heap rewrite is required. Recovery is transactional before
commit and forward-fix after commit; migration history and applied migration SQL
must never be rewritten.
