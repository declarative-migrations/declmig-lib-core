# Peer persistence reconciliation

This document defines the release state machine for TypeSpec, JSON Schema/OpenAPI, Diesel, SeaORM, and DPM.

## Independent generation lanes

The contract sources are peers:

```text
TypeSpec
  -> PostgreSQL/CockroachDB SQL candidate
  -> Protobuf descriptors
  -> gRPC service/client surfaces
  -> canonical wire-type manifest

JSON Schema / OpenAPI
  -> PostgreSQL/CockroachDB SQL candidate
  -> Rust/Dart/TypeScript/Gleam interfaces and validators
  -> write-client surfaces
  -> canonical wire-type manifest
```

One source is never regenerated from the other during certification. Shared fixtures may be authored once, but each lane must validate them independently.

## Certification state machine

1. `generated`: every required output exists and records exact tool/source digests.
2. `sql-compared`: both SQL candidates materialize successfully and DPM reports equivalent catalogs.
3. `types-compared`: canonical type manifests agree on names, variants, requiredness, nullability, ranges, formats, identifiers, timestamps, decimals, and operation request/response shapes.
4. `orm-generated`: Diesel and SeaORM independently project the agreed database.
5. `orm-compared`: canonical ORM manifests and shared fixture effects agree.
6. `verified`: DPM proves migration convergence from every supported previous release.
7. `publishable`: all reports say `proceed` and their digests are bound into the release manifest.

Any failure moves the release to `paused`, not to a fallback lane.

## Discrepancy handling

A discrepancy report must identify:

- stage and artifact class;
- TypeSpec/JSON Schema/OpenAPI or Diesel/SeaORM source identities;
- exact input, generator, SQL, type-manifest, catalog, and fixture-result digests;
- semantic paths and both observed values;
- whether the mismatch is missing output, invalid output, unsupported construct, or unequal semantics;
- reviewer decision and rationale once evaluated.

While paused, CI blocks migration apply, package publication, generated client publication, and promotion of the affected release evidence. An explicit review may change a source contract, repair a generator, or add a narrowly scoped and versioned equivalence rule. CI then regenerates every artifact from clean inputs. It never edits generated outputs to make the comparison pass.

## Approved equivalence rules

Equivalence rules must be rare, named, versioned, and reviewed. Examples may include field/import ordering or generator provenance metadata. Rules must never hide differences in:

- SQL catalog objects or database invariants;
- integer width, decimal precision/scale, timestamp/offset behavior, or identifier representation;
- optional versus nullable fields;
- enum membership or discriminators;
- request/response directionality;
- column nullability, defaults, generated status, keys, constraints, or relationship targets;
- read/write capability.

## Migration authority

DPM alone plans, verifies, and applies schema migrations. Contract generators and ORMs may emit proposed SQL, but those proposals are materialized in disposable databases and compared through DPM before use. Application processes never acquire migration credentials.
