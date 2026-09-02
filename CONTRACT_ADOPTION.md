# Fleet adoption contract

Each database-backed product organization adopts the convergence gates in its
canonical `<product>-lib-core` repository. Web/API servers and `*-orm-core`
packages consume reviewed generated artifacts; they do not become schema or
migration authorities.

## Required independent roots

Every adopter must retain:

- a TypeSpec source manifest, source revision, compiler lock, semantic IR,
  independently generated SQL, Protobuf descriptors/gRPC services, and
  wire-client semantic/output digests;
- a separately human-authored JSON Schema/OpenAPI source manifest, source
  revision, generator lock, semantic IR, independently generated SQL,
  client-interface/type manifests, and write-client output digests;
- an explicit, versioned, reviewed mapping for shared concepts across the two
  lanes; and
- derived translation artifacts only under a non-authoritative namespace with
  origin, compiler, input, and output digests.

A TypeSpec-emitted JSON Schema/OpenAPI file may not replace or feed the
independent JSON Schema/OpenAPI production root. A generated TypeSpec projection
may not replace or feed the TypeSpec root.

## Required convergence evidence

Before promotion, CI must:

1. generate `SQL_T` and `SQL_J` independently;
2. apply them to separate disposable databases;
3. compare normalized catalogs, including defaults, constraints, indexes,
   vector dimensions, RLS/policies/grants, and ownership;
4. compare both catalogs with reviewed DPM desired state and shadow/live
   catalog read-back;
5. compare TypeSpec-derived and JSON Schema/OpenAPI-derived shared client
   semantics through the reviewed mapping;
6. generate Diesel and SeaORM independently from the same pinned accepted
   catalog, compare their structural projections, and run shared behavioral
   fixtures; and
7. record PostgreSQL and CockroachDB as separate evidence lanes.

Required artifacts include exact source, compiler/generator, SQL, catalog,
client-manifest, ORM, DPM-plan, and test-output digests. Every structured artifact
must reject unknown fields and preserve deterministic ordering. The
TypeSpec-generated Draft 2020-12 schema under `contracts/schema-parity/` may
validate its own TypeSpec-lane evidence envelope, but it is not the neutral
arbiter or the independently authored JSON Schema/OpenAPI source.

## Discrepancy protocol

Any unexplained difference changes the run to `STOPPED_FOR_EVALUATION`. CI must
stop migration and ORM promotion, package/client publication, merge, and
deployment; retain a deterministic fingerprint and minimal semantic diff;
idempotently update one canonical GitHub/Linear record; and require a reviewed
repair or a narrow expiring waiver followed by a clean rerun. No TypeSpec,
JSON Schema/OpenAPI, Diesel, SeaORM, DPM, or live-catalog lane may be selected as
a silent winner.

The gate is additive to migration replay, destructive-change review, database
permission/RLS review, cross-tenant negative tests, backup/restore, and live
catalog read-back. ORM parity alone cannot certify persistence features outside
the ORM projection surface.

CI remains read-only: it may regenerate ephemeral evidence and compare it with
reviewed digests, but it may not push source, locks, or generated artifacts back
to its pull-request branch. Production mutation requires a separate reviewed
migration/deployment plan with rollback evidence.

`ores-otel` may receive only low-cardinality result metadata: project, exact
revision, engine, source/generator versions, digests, stable difference codes,
and final status. It must never receive database URLs, SQL containing literal
data, credentials, row values, tenant identifiers, private source bodies, or
generated client/ORM source.

Implementations must follow the feature-branch, independent-review,
exact-evidence, least-privilege, and credential boundaries in
`ORESoftware/my-ai/AGENTS.md`.
