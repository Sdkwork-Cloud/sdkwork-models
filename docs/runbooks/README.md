# Runbooks

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-31

## AI Resource Group Mutation Failure

Signals: member assignment/removal returns a conflict or server problem, audit
events stop, routing versions stop advancing, database readiness fails, or pool
wait and retry counts exceed the deployment SLO.

1. Capture `traceId`, tenant, organization, group id, resource code, operation, and
   timestamp. Never capture authorization headers or database credentials.
2. Confirm `/readyz`, database pool saturation, transaction latency, and PostgreSQL
   SQLSTATE. Treat `40001` and `40P01` as bounded-retry candidates; do not blindly
   retry validation, capacity, uniqueness, or authorization failures.
3. Verify membership, `ops_audit_log`, and `ai_routing_config_change` together. A
   failed transaction must leave none of the three partially committed. A repeated
   delete may correctly leave all three unchanged.
4. If routing state lags committed data, stop further configuration rollout,
   preserve evidence, and repair through the approved routing-version recovery
   procedure. Do not manufacture audit or change rows by hand.
5. Reduce writer concurrency or isolate the affected tenant/group if contention is
   sustained. Roll back the application release before changing migrations, pool
   policy, retry policy, or production database configuration.
6. Close the incident only after a fresh member mutation, audit lookup, routing
   version observation, and paginated read all succeed.

Escalation owner: SDKWork Models platform maintainers. Database migration,
production configuration, or manual data repair requires human review.

Authority: `DATABASE_SPEC.md`, `SECURITY_SPEC.md`, `DOCUMENTATION_SPEC.md`.
