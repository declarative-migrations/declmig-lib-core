import "@typespec/json-schema";

using TypeSpec.JsonSchema;

namespace DeclarativeMigrations.SchemaParity;

union DatabaseEngine {
  "postgresql",
  "cockroachdb",
}

union ProjectionSource {
  "catalog",
  "diesel-schema",
  "seaorm-entity",
  "diesel-roundtrip",
}

model GeneratorIdentity {
  @minLength(1) name: string;
  @minLength(1) version: string;
}

model DatabaseIdentity {
  engine: DatabaseEngine;
  @minLength(1) schema: string;
}

model ColumnProjection {
  @minLength(1) name: string;
  @minValue(1) ordinal: uint32;
  @minLength(1) type_family: string;
  nullable: boolean;
  native_type?: string;
}

model TableProjection {
  @minLength(1) schema: string;
  @minLength(1) name: string;
  primary_key: string[];
  columns: ColumnProjection[];
}

@jsonSchema
model SchemaProjection {
  schema_version: 1;
  source: ProjectionSource;
  generator: GeneratorIdentity;
  database: DatabaseIdentity;
  tables: TableProjection[];
}

union DifferenceCode {
  "column-ordinal-mismatch",
  "database-engine-mismatch",
  "database-schema-mismatch",
  "extra-column",
  "extra-table",
  "missing-column",
  "missing-table",
  "nullability-mismatch",
  "primary-key-mismatch",
  "type-family-mismatch",
}

model Difference {
  code: DifferenceCode;
  @minLength(1) table: string;
  column?: string;
  expected?: string;
  actual?: string;
}

model ParityWarning {
  @minLength(1) code: string;
  @minLength(1) detail: string;
}

@jsonSchema
model ParityReport {
  schema_version: 1;
  expected_source: ProjectionSource;
  actual_source: ProjectionSource;
  compatible: boolean;
  differences: Difference[];
  warnings: ParityWarning[];
}
