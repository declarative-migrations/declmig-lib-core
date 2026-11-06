# Six-party schema authority bounded admission

`authority_model.cjs` explores independently generated TypeSpec SQL, independently authored JSON Schema/OpenAPI SQL, reviewed DPM desired state, Diesel, SeaORM, and shadow/live readback. Each participant can be unobserved, matching, or divergent.

The model rejects promotion while any participant is missing or divergent, never elects a winning source, and requires all six matches plus independent review before publication. Positive and negative witnesses are mandatory; deliberately corrupted states must be rejected.

Run `node formal/atto-schema/authority_model.cjs`.

`schema_authority.atto` is AttoLang **source-only** policy, not SQL generation, executable DDL, ORM implementation, or a guest-compiled program. Independently authored TypeSpec and JSON Schema remain peer contract authorities. Only DPM plans/verifies/applies migrations; do not add a migration execution capability to this crate.

To claim concrete conformance, replay counterexamples through two disposable independently generated catalogs and both ORMs, compare normalized semantic diff and permissions/RLS, and execute the exact `.atto` policy on pinned compilers with stepful CI, independent review and explicit negative evidence. Model success alone cannot authorize a production migration.
