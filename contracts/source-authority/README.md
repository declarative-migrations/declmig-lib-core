# Dual source-authority contract

`authority.v1.json` is a small, hand-reviewed architecture contract. It makes the
schema topology machine-checkable without pretending that the TypeSpec, JSON
Schema/OpenAPI, SQL, client, Diesel, or SeaORM compilers are already complete.
The validator is dependency-free and returns `STOPPED_FOR_EVALUATION` whenever a
required root, output, comparison, convergence participant, or blocking action
is removed.

## Production roots

The two production roots are independent:

```text
TypeSpec
  -> semantic IR_T
  -> SQL_T
  -> Protobuf descriptors and gRPC services
  -> wire-client types and wire clients
```

paired with:

```text
human-authored JSON Schema + OpenAPI
  -> semantic IR_J
  -> SQL_J
  -> client interfaces and types
  -> write-client operations
```

Neither source root may be generated from, replaced by, or silently certified
by the other. A JSON Schema or OpenAPI document emitted from TypeSpec is a
derived diagnostic artifact only. The same rule applies to a TypeSpec model
projected from JSON Schema/OpenAPI. Derived translations must live outside the
production source roots, retain origin/compiler/input/output digests, and may
never feed production generation.

## Required comparisons

`SQL_T` and `SQL_J` are applied to separate disposable databases. Their raw SQL
may be compared, but the primary oracle is normalized catalog equivalence. The
catalog comparison includes native types, defaults, generated expressions,
keys, references, checks, indexes, vector dimensions, RLS, policies, grants,
roles, and ownership—not just table and column names.

TypeSpec-derived wire/client semantic manifests and JSON Schema/OpenAPI-derived
interface/type/write-client manifests are compared through an explicit,
versioned, reviewed mapping. Naming conventions and transport wrappers may be
mapped; required/optional/nullable semantics, scalar widths and formats,
constraints, unions, discriminators, enums, operations, and error envelopes may
not be silently discarded.

Diesel and SeaORM are generated independently from the same pinned accepted
catalog. They are compared structurally and with shared behavioral fixtures.
Neither ORM generates the other, and ORM agreement cannot certify persistence
features outside their projection surface.

## Convergence and stopping

A release candidate is not converged until all of these agree at exact pinned
revisions:

```text
TypeSpec SQL catalog
JSON Schema/OpenAPI SQL catalog
reviewed DPM desired-state catalog
Diesel projection
SeaORM projection
shadow/live catalog read-back
```

PostgreSQL and CockroachDB are separate evidence lanes. Any unexplained mismatch
must produce one deterministic discrepancy fingerprint, block migration and ORM
promotion, package/client publication, merge, and deployment, and remain
`STOPPED_FOR_EVALUATION` until a reviewed repair—or a narrow expiring waiver—is
followed by a clean rerun.

## Validate

```sh
python3 scripts/ci/validate_source_authority.py
python3 -m unittest discover -s tests -p 'test_source_authority.py' -v
```

The validator emits stable public-safe error codes. It reads no database URL,
credential, source body, row value, or tenant identifier.
