use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::ParityError;

pub const PARITY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DatabaseEngine {
    #[serde(rename = "postgresql")]
    PostgreSql,
    #[serde(rename = "cockroachdb")]
    CockroachDb,
}

impl DatabaseEngine {
    pub fn parse(value: &str) -> Result<Self, ParityError> {
        match value {
            "postgresql" => Ok(Self::PostgreSql),
            "cockroachdb" => Ok(Self::CockroachDb),
            _ => Err(ParityError::InvalidEngine),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ProjectionSource {
    #[serde(rename = "catalog")]
    Catalog,
    #[serde(rename = "diesel-schema")]
    DieselSchema,
    #[serde(rename = "seaorm-entity")]
    SeaOrmEntity,
    #[serde(rename = "diesel-roundtrip")]
    DieselRoundTrip,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratorIdentity {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DatabaseIdentity {
    pub engine: DatabaseEngine,
    pub schema: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ColumnProjection {
    pub name: String,
    pub ordinal: u32,
    pub type_family: String,
    pub nullable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TableProjection {
    pub schema: String,
    pub name: String,
    #[serde(default)]
    pub primary_key: Vec<String>,
    pub columns: Vec<ColumnProjection>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SchemaProjection {
    pub schema_version: u32,
    pub source: ProjectionSource,
    pub generator: GeneratorIdentity,
    pub database: DatabaseIdentity,
    pub tables: Vec<TableProjection>,
}

impl SchemaProjection {
    pub fn normalize(mut self) -> Result<Self, ParityError> {
        self.generator.name = required_text(self.generator.name)?;
        self.generator.version = required_text(self.generator.version)?;
        self.database.schema = required_text(self.database.schema)?;

        let mut table_names = BTreeSet::new();
        for table in &mut self.tables {
            table.schema = required_text(std::mem::take(&mut table.schema))?;
            table.name = required_text(std::mem::take(&mut table.name))?;
            let table_key = (table.schema.clone(), table.name.clone());
            if !table_names.insert(table_key) {
                return Err(ParityError::DuplicateTable);
            }

            let mut column_names = BTreeSet::new();
            let mut ordinals = BTreeSet::new();
            for column in &mut table.columns {
                column.name = required_text(std::mem::take(&mut column.name))?;
                column.type_family = normalize_type_family(&column.type_family)?;
                column.native_type = column.native_type.take().map(required_text).transpose()?;
                if column.ordinal == 0 || !ordinals.insert(column.ordinal) {
                    return Err(ParityError::InvalidOrdinal);
                }
                if !column_names.insert(column.name.clone()) {
                    return Err(ParityError::DuplicateColumn);
                }
            }
            if table.columns.is_empty() {
                return Err(ParityError::EmptyTable);
            }
            table.columns.sort_by(|left, right| {
                (left.ordinal, &left.name).cmp(&(right.ordinal, &right.name))
            });

            let known_columns: BTreeSet<_> = table
                .columns
                .iter()
                .map(|column| column.name.as_str())
                .collect();
            let mut primary_key_columns = BTreeSet::new();
            for column in &mut table.primary_key {
                *column = required_text(std::mem::take(column))?;
                if !known_columns.contains(column.as_str()) {
                    return Err(ParityError::UnknownPrimaryKeyColumn);
                }
                if !primary_key_columns.insert(column.clone()) {
                    return Err(ParityError::DuplicatePrimaryKeyColumn);
                }
            }
        }
        if self.schema_version != PARITY_SCHEMA_VERSION {
            return Err(ParityError::UnsupportedSchemaVersion);
        }
        if self.tables.is_empty() {
            return Err(ParityError::EmptyProjection);
        }
        self.tables.sort_by(|left, right| {
            (&left.schema, &left.name).cmp(&(&right.schema, &right.name))
        });
        Ok(self)
    }

    pub fn table_map(&self) -> BTreeMap<(&str, &str), &TableProjection> {
        self.tables
            .iter()
            .map(|table| ((table.schema.as_str(), table.name.as_str()), table))
            .collect()
    }
}

pub fn normalize_type_family(value: &str) -> Result<String, ParityError> {
    let compact = value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '_')
        .collect::<String>()
        .replace("::", "");
    let family = match compact.as_str() {
        "bool" | "boolean" => "bool",
        "int2" | "smallint" | "i16" => "int2",
        "int4" | "integer" | "i32" => "int4",
        "int8" | "bigint" | "i64" => "int8",
        "float4" | "real" | "f32" => "float4",
        "float8" | "double" | "doubleprecision" | "f64" => "float8",
        "numeric" | "decimal" | "bigdecimal" | "rustdecimaldecimal" => "numeric",
        "text" => "text",
        "varchar" | "charactervarying" | "string" => "varchar",
        "bpchar" | "character" | "char" => "char",
        "uuid" => "uuid",
        "json" => "json",
        "jsonb" | "jsonbinary" => "jsonb",
        "bytea" | "binary" | "varbinary" | "vecu8" => "bytea",
        "date" | "naivedate" => "date",
        "time" | "naivetime" => "time",
        "timestamp" | "datetime" | "naivedatetime" | "timestampwithouttimezone" => "timestamp",
        "timestamptz" | "datetimewithtimezone" | "timestampwithtimezone" => "timestamptz",
        "inet" | "ipnetwork" => "inet",
        "cidr" => "cidr",
        "tsvector" => "tsvector",
        value if value.starts_with("array<") && value.ends_with('>') => {
            let inner = &value[6..value.len() - 1];
            return Ok(format!("array<{}>", normalize_type_family(inner)?));
        }
        value if value.starts_with("vector") => "vector",
        value if value.starts_with("custom(") => return Ok(value.to_owned()),
        value if !value.is_empty() => return Ok(format!("custom({value})")),
        _ => return Err(ParityError::InvalidTypeFamily),
    };
    Ok(family.to_owned())
}

fn required_text(value: String) -> Result<String, ParityError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) {
        Err(ParityError::InvalidText)
    } else {
        Ok(trimmed.to_owned())
    }
}
