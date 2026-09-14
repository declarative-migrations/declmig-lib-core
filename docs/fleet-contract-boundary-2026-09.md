# Fleet contract dependency boundary

Declarative Migrations owns migration planning/execution semantics. When migration coordination needs exclusivity, leases, or fencing, the public fleet dependency is `oresoftware/ores-locks-and-leases`; applications must not depend directly on Fiducia merely to acquire a distributed lock.

## Fleet dependency rules

- Reuse transport/deployment-neutral primitives from `oresoftware/ores-interfaces` where they are genuinely fleet-generic.
- Keep migration-specific contracts and persistence semantics in Declarative Migrations.
- Kubernetes, CRD, Helm/Kustomize, NATS/Redis deployment topology belongs in `oresoftware/k8s-libs-and-shared-defs`, not in product semantic contracts.
- Coordination goes through `oresoftware/ores-locks-and-leases`; concrete backends such as local file locks, Postgres advisory locks, Fiducia, Redis, or Cloudflare Durable Objects are implementation/provider choices behind that boundary.
- Independently authored TypeSpec and JSON Schema Draft 2020-12 authorities are admitted with `oresoftware/typespec-json-schema-validator`; neither authority is regenerated from the other to force parity.
- `oresoftware/ores-cli` is governance/build tooling, not a runtime dependency. It should reject forbidden dependency edges and run cross-repo contract checks against the exact zed-pkg resolved graph.
- `.zpkg.toml` expresses dependency intent; `.zpkg.lock` is execution truth. Blocking compatibility uses the locked artifact/version/provenance. Tip-of-default-branch comparison is a separately reported forward-compatibility canary.

## Anti-cycle rule

Shared fleet primitives and tooling may be consumed by this repo, but shared/tooling repositories must not reach back into Declarative Migrations runtime implementations. Direct provider imports that bypass `ores-locks-and-leases` require an explicit, narrowly scoped adapter exception.