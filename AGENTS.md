# Declarative Migrations — lib-core

Canonical domain-mapping and persistence-reconciliation repository for [`declarative-migrations`](https://github.com/declarative-migrations).

- TypeSpec and JSON Schema/OpenAPI are peer top-level contract authorities.
- TypeSpec independently generates SQL, Protobuf, gRPC, and wire-client artifacts.
- JSON Schema/OpenAPI independently generates SQL, interfaces/types, validators, and write-client artifacts.
- Compare both SQL candidates through DPM catalogs and both type candidates through canonical manifests.
- Any missing artifact or semantic discrepancy must stop with `pause`; never elect a source automatically.
- Diesel and SeaORM are peer ORM projections and must cross-check each other.
- Only DPM plans, verifies, and applies schema migrations. This crate must not expose migration execution.
- Internal runtimes: Rust, TypeScript, Dart, and Gleam where supported.
- Auth: github.com/shared-auth.
- Sync: github.com/opto-sync.
- Telemetry: github.com/ores-otel.
- Flags: github.com/flags-2-env.
- Packages: github.com/zed-pkg.
- Never use React/JSX or webviews.
- Resolve git conflicts semantically; never rebase, stash, or reset.
