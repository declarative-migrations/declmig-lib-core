# Declarative Migrations — lib-core

Canonical packaging boundary for reconciled domain and schema-release artifacts
in [`declarative-migrations`](https://github.com/declarative-migrations).

- TypeSpec and JSON Schema/OpenAPI are independent top-level contract authorities.
- TypeSpec emits SQL, Protobuf/gRPC, and wire-client types.
- JSON Schema/OpenAPI emit SQL, client/interface types, and write-client contracts.
- Compare both SQL candidates through DPM catalogs and both type surfaces through
  `declmig.generated-types/v1`.
- SeaORM and Diesel are independent peer projections of one exact DPM catalog;
  compare them through `declmig.orm-projection/v1`.
- A missing peer artifact, invalid evidence, generator error, or discrepancy must
  pause. Never select an automatic winner.
- Package a schema release only when an immutable
  `declmig.peer-authority-certification/v1` recomputes to `continue`.
- Schema migration execution belongs to DPM's privileged deployment plane, not
  this library and not application startup.
- Auth: github.com/shared-auth.
- Sync: github.com/opto-sync.
- Telemetry: github.com/ores-otel.
- Flags: github.com/flags-2-env.
- Packages: github.com/zed-pkg.
- Never use React/JSX or webviews.
- Resolve git conflicts semantically; never rebase, stash, or reset.
