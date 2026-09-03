# Isomorphic validation SDK

This directory is the runtime validation projection for `declarative-migrations/declmig-lib-core`.

- Public definitions are authored in `declmig-interfaces` and are safe for browser, mobile, desktop, CLI, and server consumers.
- Server definitions live only here. They may extend public definitions but are never copied into `declmig-clients`.
- Route bindings use stable `operationId` values from `ORESoftware/api-docs`; validators do not invent a second route namespace.
- TypeSpec and JSON Schema/OpenAPI remain independent peer authorities. Validation output is compared evidence, not an elected authority.
- Any semantic mismatch with either authority, Diesel, or SeaORM pauses release for evaluation.

Runtime choices are Zod for TypeScript, Garde for Rust, `go-playground/validator/v10` for Go, and Gleam's official dynamic decoder API.
