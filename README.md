# declmig-lib-core

Domain mapping, persistence policy, and neutral convergence machinery for Declarative Migrations. Product `*-lib-core` repositories package independently authored TypeSpec and JSON Schema/OpenAPI roots, reviewed migration intent, generator locks, and immutable evidence. Packaging both roots in one repository does not collapse them into one compiler or make either root a generated child of the other.

## Independent top-level contract authorities

```text
TypeSpec
  -> TypeSpec semantic IR
  -> SQL_T
  -> Protobuf descriptors and gRPC services
  -> wire-client types and wire clients
```

paired with:

```text
human-authored JSON Schema + OpenAPI
  -> JSON/OpenAPI semantic IR
  -> SQL_J
  -> interfaces and types for supported clients
  -> validators and write-client request/response stubs
```

The forbidden production topology is:

```text
TypeSpec -> JSON Schema / OpenAPI -> downstream generation
```

TypeSpec may emit JSON Schema or OpenAPI only as a namespaced diagnostic or round-trip artifact. Such output must retain its TypeSpec origin and compiler digests and may not overwrite, replace, feed, or certify the independently authored JSON Schema/OpenAPI source lane. The inverse rule applies to TypeSpec projected from JSON Schema/OpenAPI.

`contracts/source-authority/authority.v1.json` makes these boundaries, required outputs, comparisons, convergence participants, and the fail-closed discrepancy state machine-checkable:

```sh
python3 scripts/ci/validate_source_authority.py
python3 -m unittest discover -s tests -p 'test_source_authority.py' -v
```

## SQL and client convergence

`SQL_T` and `SQL_J` must be generated independently, applied to separate disposable databases, introspected, normalized, and compared. Raw SQL equality is useful evidence; normalized catalog equivalence is the primary oracle. The comparison includes defaults, generated expressions, keys, references, checks, indexes, vector dimensions, RLS, policies, grants, roles, and ownership in addition to tables, columns, ordinals, types, and nullability.

TypeSpec-derived wire/client semantic manifests and JSON Schema/OpenAPI-derived interface/type/write-client manifests must also be compared. An explicit, versioned, reviewed mapping may account for casing and transport wrappers, but may not silently discard required/optional/nullable semantics, scalar widths, formats, constraints, unions, discriminators, enums, operations, or errors.

Any unexplained mismatch produces `STOPPED_FOR_EVALUATION`, one deterministic fingerprint and minimal semantic diff, and blocks migration/ORM promotion, package/client publication, merge, and deployment. No source lane or ORM wins by fallback.

## Diesel / SeaORM structural cross-check

`declmig-schema-crosscheck` turns pinned generator output into deterministic, secret-free structural projections and compares them. Diesel and SeaORM must be generated independently from the same pinned accepted catalog; neither ORM may generate the other or own production DDL.

The current structural pathway is:

1. Require independent `SQL_T` and `SQL_J` disposable catalogs to converge with the reviewed DPM desired-state catalog.
2. Retain exact source, compiler, SQL, catalog, and DPM-plan digests.
3. Run `diesel print-schema` against the accepted disposable catalog.
4. Run `sea-orm-cli generate entity --entity-format compact` independently against the same catalog.
5. Parse both Rust projections and compare table identity, ordered primary keys, columns, order, normalized type families, and nullability.
6. Run shared read/write/error/transaction fixtures through both ORMs against PostgreSQL and CockroachDB as separate evidence lanes.
7. Compare both ORM projections with normalized catalog/DPM evidence for defaults, checks, indexes, grants/RLS, vector dimensions, and migration safety outside the lossy ORM surface.
8. Publish exact source/generator/output digests through Zed only after all gates pass.

The optional Diesel `--diff-schema` round trip remains a lossy secondary oracle, never migration authority. A green ORM projection comparison does not prove defaults, custom checks, every index, grants/RLS, tenant authorization, vector dimensions, or migration safety.

### Examples

```sh
declmig-schema-crosscheck parse-diesel \
  --input generated/diesel/schema.rs \
  --output generated/diesel.json \
  --engine postgresql \
  --schema public \
  --generator-version 2.3.12

declmig-schema-crosscheck parse-seaorm \
  --input-dir generated/seaorm \
  --output generated/seaorm.json \
  --engine postgresql \
  --schema public \
  --generator-version 2.0.2

declmig-schema-crosscheck compare \
  --expected generated/diesel.json \
  --actual generated/seaorm.json \
  --output generated/parity-report.json
```

Exit code `0` means structural parity, `2` means deterministic drift, and `64` means malformed input or an unsupported generator shape. Errors contain stable codes only; database URLs and data values are never accepted or logged.

## Convergence participants

A candidate is not certified until all six participants agree at exact pinned revisions:

- the TypeSpec-generated SQL catalog;
- the JSON Schema/OpenAPI-generated SQL catalog;
- the reviewed DPM desired-state catalog;
- the Diesel projection;
- the SeaORM projection; and
- the shadow/live catalog read-back.

The TypeSpec contract under `contracts/schema-parity/` describes a TypeSpec-lane projection/report evidence envelope. Any JSON Schema emitted from that contract is diagnostic evidence for that lane only; it is not the independent JSON Schema/OpenAPI product source and may not validate that source into existence.

## Responsibility of this crate

`declmig-lib-core` owns:

- domain invariants that must be represented consistently in both contract lanes;
- explicit mappings where wire and relational concepts cannot be inferred safely;
- compatibility epochs and release metadata;
- reconciliation policy and approved, narrowly scoped equivalence rules;
- named business operations shared by service adapters;
- deterministic comparison tooling and evidence formats that do not confer authority on either lane.

It does **not** own a unilateral schema source, expose generic DDL, or apply production migrations. DPM is the only migration planner/applicator. Web, API, worker, Diesel, and SeaORM crates consume reviewed artifacts and database roles with the minimum required privilege.

The legacy Cargo feature named `migrate` is a compatibility marker only and must not expose migration execution. Its removal is tracked separately so downstream consumers can migrate without an unreviewed breaking release.

Generated files are derived and must carry exact source/compiler/generator digests. Runtime services use opaque named capabilities and separate read, write, and migrator credentials. Neither ORM runs production DDL at API/web startup.

The independent compiler, full-catalog, client-semantic, and shared ORM behavior lanes remain blocking implementation work. This repository reports that incompleteness honestly rather than claiming parity from name presence or compile-only checks.

See [`docs/peer-persistence-reconciliation.md`](docs/peer-persistence-reconciliation.md) for the release state machine and discrepancy protocol.
