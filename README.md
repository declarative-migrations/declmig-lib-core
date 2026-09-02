# declmig-lib-core

Canonical persistence and migration source for Declarative Migrations. Product
`*-lib-core` repositories own human-reviewed desired SQL, persistence JSON
Schema, migration inputs and generator configuration. `*-orm-core` packages are
runtime-only generated compatibility boundaries; they never become a second
schema authority.

## Diesel / SeaORM / TypeSpec cross-check

`declmig-schema-crosscheck` turns pinned generator output into a deterministic,
secret-free structural projection and compares two projections. The intended
CI pathway is:

1. Apply the canonical desired SQL to a new disposable shadow database.
2. Run DPM diff/verify and retain the catalog/SQL digest.
3. Run `diesel print-schema` against that shadow database.
4. Run `sea-orm-cli generate entity --entity-format compact` against the same
   database.
5. Parse both generated Rust projections and compare table identity, ordered
   primary keys, columns, order, normalized type families and nullability.
6. Optionally run `diesel migration generate --diff-schema` against a second
   empty database, apply its generated SQL, regenerate SeaORM entities and
   compare again. This is a deliberately lossy round-trip check, not migration
   authority.
7. Validate projection/report JSON with the TypeSpec-generated Draft 2020-12
   JSON Schema and publish source/generator/output digests through Zed.

Diesel documents that `--diff-schema` cannot preserve defaults, custom check
constraints and similar SQL features. Accordingly, a green ORM projection
comparison never certifies defaults, checks, every index, grants, RLS, tenant
authorization, vector dimensions or migration safety. Those remain DPM/catalog,
permission and behavioral-test gates.

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

Exit code `0` means structural parity, `2` means deterministic drift, and `64`
means malformed input or an unsupported generator shape. Errors contain stable
codes only; database URLs and data values are never accepted or logged.

## Five-layer authority

- TypeSpec authors public/wire and machine-evidence models.
- JSON Schema validates JSON instances and checked generator artifacts.
- Human-reviewed desired SQL plus DPM own persistence convergence.
- Diesel and SeaORM are independent Rust projections over the same catalog.
- Zed owns package identity, cross-repository pins, tasks and artifact digests.

Neither ORM is allowed to run production DDL at API/web startup. Generated files
are derived and must carry exact generator/source digests. Runtime services use
opaque named capabilities and separate read/write/migrator credentials.
