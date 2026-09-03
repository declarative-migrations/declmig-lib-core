# declmig-lib-core

Domain mapping, persistence policy, and reconciliation core for Declarative Migrations.

## Peer contract authorities

TypeSpec and JSON Schema/OpenAPI are independent, top-level contract authorities. Neither is generated from the other and neither silently outranks the other.

```text
TypeSpec -----------------> SQL + Protobuf + gRPC + wire clients
JSON Schema / OpenAPI ----> SQL + interfaces/types + write clients
                  \          /
                   semantic comparison
                           |
                 proceed or pause/evaluate
```

Both lanes must emit deterministic SQL candidates and canonical type manifests. DPM materializes both SQL candidates and compares their normalized database catalogs. Type manifests are compared after removing formatting and generator-only metadata. Any missing output, unsupported feature, generator failure, or semantic discrepancy blocks publication and migration application with a machine-readable `pause` report.

## ORM peer projections

Diesel and SeaORM independently project the agreed candidate database. They cross-check each other through canonical ORM projection manifests and shared read/write fixtures. Neither ORM owns migrations or becomes the fallback winner when the other disagrees.

## Responsibility of this crate

`declmig-lib-core` owns:

- domain invariants that must be represented consistently in both contract lanes;
- explicit mappings where wire and relational concepts cannot be inferred safely;
- compatibility epochs and release metadata;
- reconciliation policy and approved, narrowly scoped equivalence rules;
- named business operations shared by service adapters.

It does **not** own a unilateral schema source, expose generic DDL, or apply production migrations. DPM is the only migration planner/applicator. Web, API, worker, Diesel, and SeaORM crates consume reviewed artifacts and database roles with the minimum required privilege.

The legacy Cargo feature named `migrate` is a compatibility marker only and must not expose migration execution. Its removal is tracked separately so downstream consumers can migrate without an unreviewed breaking release.

See [`docs/peer-persistence-reconciliation.md`](docs/peer-persistence-reconciliation.md) for the release state machine and discrepancy protocol.
