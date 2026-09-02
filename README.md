# declmig-lib-core

Reconciled domain and schema-artifact boundary for Declarative Migrations.

TypeSpec and JSON Schema/OpenAPI are independent top-level contract authorities.
They independently generate SQL and client/interface artifacts, then cross-check
through DPM catalogs and canonical semantic manifests. SeaORM and Diesel are
independent peer projections of the same approved catalog and must cross-check as
well.

This crate packages an approved schema release only after an immutable
`declmig.peer-authority-certification/v1` recomputes to `continue`. Any missing
peer artifact, generator error, invalid evidence, or semantic discrepancy pauses
the release; this crate never selects an automatic winner.

Application services may consume narrow read/write capabilities, but schema
migration execution remains in DPM's privileged deployment plane. See
[`docs/schema-authority.md`](docs/schema-authority.md).
