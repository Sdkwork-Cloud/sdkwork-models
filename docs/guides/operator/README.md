# Operator Guide

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-31

PostgreSQL is the authoritative server database for SDKWork Models. Do not use
the SQLite adapter as a multi-node production authority. Run schema lifecycle
through `sdkwork-database`; production startup must not silently migrate or seed.

## Capacity Controls

- Admin lists use server-side offset pagination (`page=1`, `page_size=20` by
  default; `page_size` must not exceed `200`). Monitor latency, row count, pool
  wait time, and response size rather than increasing the cap.
- Manual AI resource groups are capped at `512` members. A capacity conflict is
  an operator-visible contract response, not a reason to retry without changing
  the group design. Dynamic groups must not persist manual relationships.
- Member assignment and removal use the single-member SDK commands. Avoid clients
  that aggregate all members before a mutation, because they increase memory and
  transaction cost and cannot preserve independent concurrent changes.

## Operational Signals

- Correlate failed commands by HTTP `traceId`, `ops_audit_log.request_id`, target
  id, tenant, organization, and operator. Do not expose database error strings to
  API callers.
- Observe `ai_routing_config_change` after committed AI resource or group changes.
  Missing events indicate cache invalidation risk; duplicate-delete requests are
  expected to produce no event.
- Alert on database readiness failure, connection-pool wait saturation, repeated
  serialization/deadlock errors, audit insertion failure, and routing-version lag.

## Release Evidence

Before commercial release, run the repository verification plus isolated
PostgreSQL contention tests. Record the test database identity, migration version,
concurrency level, retry budget, SQLSTATE distribution, pool limits, and rollback
result without recording credentials. See the resource-group runbook for triage.

Authority: `DATABASE_SPEC.md`, `PAGINATION_SPEC.md`, `SECURITY_SPEC.md`,
`DOCUMENTATION_SPEC.md`.
