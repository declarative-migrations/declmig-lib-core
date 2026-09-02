# Reconciled peer-authority package

`declmig-lib-core` is not the sole schema author and does not choose between
contract or ORM generators. It packages domain rules and database artifacts only
after independent peer paths have been reconciled.

## Required upstream authorities

```text
TypeSpec → SQL + Protobuf/gRPC + wire-client types
JSON Schema/OpenAPI → SQL + interface/client types + write-client contracts
```

Both contract paths are top-level. DPM compares their independently generated
SQL through normalized database catalogs; their generated types compare through
`declmig.generated-types/v1`. SeaORM and Diesel independently project the same
approved DPM catalog and compare through `declmig.orm-projection/v1`.

`declmig-lib-core` may package a schema release only when an immutable
`declmig.peer-authority-certification/v1` recomputes to `continue`. A missing
artifact, invalid certificate, generator error, or semantic discrepancy pauses
the release. This crate must never select TypeSpec, JSON Schema/OpenAPI, SeaORM,
or Diesel as an automatic winner.

## Runtime boundary

Application read/write operations belong behind narrow, named repository
capabilities. Schema migration execution belongs to DPM's privileged deployment
plane, not this library and not an application startup path. The legacy
`migrate` Cargo feature is retained temporarily for compatibility but grants no
DDL implementation; its removal and downstream migration are tracked as an
explicit issue.

## Packaged release identity

Every packaged schema release must bind:

- logical schema revision;
- exact peer-authority policy format;
- TypeSpec, JSON Schema/OpenAPI, SQL, generated-type, SeaORM, and Diesel input
  and output SHA-256 digests;
- exact database engine/version and normalized desired-catalog digest;
- exact DPM binary and generator identities;
- the immutable all-pass certificate digest.

The current repository is not eligible to claim an all-pass schema release,
because the independent TypeSpec, JSON Schema/OpenAPI SQL, and Diesel projection
paths are not yet complete.
