# Fleet adoption contract

Each database-backed product organization adopts this gate in its canonical
`<product>-lib-core` repository, not in the web/API server or compatibility ORM
repository.

Required inputs:

- canonical `desired.sql` and persistence JSON Schema;
- exact DPM, Diesel CLI and SeaORM CLI versions;
- database engine and schema namespace;
- generated Diesel schema and SeaORM entity directories;
- exact source, SQL, JSON Schema, generator and output digests;
- paired `*-test` shadow-database evidence before production promotion.

Every projection and report is a sealed artifact. It must validate against the
reviewed TypeSpec-generated Draft 2020-12 JSON Schema and deserialize through
Rust models that reject unknown fields. Column ordinals must be unique,
positive, and contiguous from one. This prevents a producer and consumer from
silently interpreting extra properties or sparse column order differently.

The gate is additive to, not a replacement for, DPM convergence, migration
replay, destructive-change review, database grants/RLS, cross-tenant negative
tests and live catalog readback. PostgreSQL and CockroachDB must be recorded as
separate evidence lanes. A PostgreSQL success is not CockroachDB certification.

CI must remain read-only: it may regenerate ephemeral evidence and compare it
to reviewed digests, but it must never push source, lockfiles, or generated
artifacts back to its own pull-request branch.

`ores-otel` may receive only low-cardinality result metadata: project, source
revision, engine, schema digest, generator versions, difference codes and final
status. It must never receive database URLs, SQL containing literal data,
credentials, row values, tenant identifiers or generated source bodies.
