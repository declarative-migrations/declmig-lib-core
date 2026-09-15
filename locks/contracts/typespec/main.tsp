// Declarative Migrations coordination catalog — independently authored TypeSpec authority.
// Peer of ../json-schema/contract.schema.json. Neither authority is generated from the other.
namespace DeclarativeMigrations.Locks;

@doc("Whether the distributed lease and PostgreSQL advisory-lock layers are enabled.")
model LockLayers {
  fiducia: boolean;
  pgAdvisory: boolean;
}

@doc("PostgreSQL advisory-lock scope for the protected execution window.")
enum PgScope {
  transaction: "transaction",
  session: "session",
}

@doc("One admitted Declarative Migrations lock-catalog entry.")
model LockCatalogEntry {
  @maxLength(64)
  domain: string;

  @maxLength(256)
  name: string;

  layers: LockLayers;
  pgScope: PgScope;
  wait: boolean;

  @maxLength(1024)
  description: string;
}

@doc("Declarative Migrations lock catalog. Runtime implementations prefix keys as declarative-migrations/<domain>/<name> and delegate coordination semantics to ORESoftware/ores-locks-and-leases.")
model LockCatalog {
  @maxLength(64)
  org: string;

  @maxLength(64)
  prefix: string;

  entries: LockCatalogEntry[];
}
