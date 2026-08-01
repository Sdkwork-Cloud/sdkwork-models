# Changelog

## 2026-07-31

- Resource, resource-group, and group-resource backend lists now expose SDKWork
  offset page envelopes and execute filtering, counting, ordering, and pagination
  in the repository. Generated Models backend TypeScript SDK types were regenerated
  from the authored backend OpenAPI authority and expose typed item/page payloads.
- Resource-group admin membership uses idempotent single-member update/delete
  operations. PostgreSQL and SQLite adapters commit membership, audit evidence, and
  routing configuration changes atomically and enforce the `512` member bound.
- No database migration is part of this change. PostgreSQL remains the authoritative
  server engine; SQLite remains the explicit local adapter.

Authority: `API_SPEC.md`, `PAGINATION_SPEC.md`, `SDK_SPEC.md`,
`DATABASE_SPEC.md`, `DOCUMENTATION_SPEC.md`.
