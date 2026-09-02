use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{ColumnProjection, ParityError, ProjectionSource, SchemaProjection, TableProjection};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DifferenceCode {
    ColumnOrdinalMismatch,
    DatabaseEngineMismatch,
    DatabaseSchemaMismatch,
    ExtraColumn,
    ExtraTable,
    MissingColumn,
    MissingTable,
    NullabilityMismatch,
    PrimaryKeyMismatch,
    TypeFamilyMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Difference {
    pub code: DifferenceCode,
    pub table: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParityWarning {
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParityReport {
    pub schema_version: u32,
    pub expected_source: ProjectionSource,
    pub actual_source: ProjectionSource,
    pub compatible: bool,
    pub differences: Vec<Difference>,
    pub warnings: Vec<ParityWarning>,
}

pub fn compare(
    expected: SchemaProjection,
    actual: SchemaProjection,
) -> Result<ParityReport, ParityError> {
    let expected = expected.normalize()?;
    let actual = actual.normalize()?;
    let mut differences = Vec::new();

    if expected.database.engine != actual.database.engine {
        differences.push(Difference {
            code: DifferenceCode::DatabaseEngineMismatch,
            table: "*".to_owned(),
            column: None,
            expected: Some(format!("{:?}", expected.database.engine)),
            actual: Some(format!("{:?}", actual.database.engine)),
        });
    }
    if expected.database.schema != actual.database.schema {
        differences.push(Difference {
            code: DifferenceCode::DatabaseSchemaMismatch,
            table: "*".to_owned(),
            column: None,
            expected: Some(expected.database.schema.clone()),
            actual: Some(actual.database.schema.clone()),
        });
    }

    let expected_tables = expected.table_map();
    let actual_tables = actual.table_map();
    let all_tables: BTreeSet<_> = expected_tables
        .keys()
        .chain(actual_tables.keys())
        .copied()
        .collect();
    for key in all_tables {
        match (expected_tables.get(&key), actual_tables.get(&key)) {
            (Some(expected_table), Some(actual_table)) => {
                compare_table(expected_table, actual_table, &mut differences)
            }
            (Some(_), None) => differences.push(Difference {
                code: DifferenceCode::MissingTable,
                table: qualified_name(key.0, key.1),
                column: None,
                expected: Some("present".to_owned()),
                actual: Some("missing".to_owned()),
            }),
            (None, Some(_)) => differences.push(Difference {
                code: DifferenceCode::ExtraTable,
                table: qualified_name(key.0, key.1),
                column: None,
                expected: Some("missing".to_owned()),
                actual: Some("present".to_owned()),
            }),
            (None, None) => unreachable!("table key came from one projection"),
        }
    }
    differences.sort_by(|left, right| {
        (&left.table, &left.column, left.code).cmp(&(&right.table, &right.column, right.code))
    });

    let mut warning_codes = vec![
        ("constraints-not-proven", "Generated ORM projections do not prove CHECK constraints or exclusion constraints."),
        ("defaults-not-proven", "Generated ORM projections do not prove default expressions or generated-column expressions."),
        ("indexes-not-proven", "Generated ORM projections do not prove every unique, partial, expression, or vector index."),
        ("row-security-not-proven", "Generated ORM projections do not prove grants, roles, row-level security, or tenant authorization."),
    ];
    if expected
        .tables
        .iter()
        .chain(&actual.tables)
        .flat_map(|table| &table.columns)
        .any(|column| column.type_family == "vector")
    {
        warning_codes.push(("vector-dimensions-not-proven", "Rust ORM projections usually preserve the vector family but not its configured dimensions; verify dimensions from the catalog and DPM plan."));
    }
    let warnings = warning_codes
        .into_iter()
        .map(|(code, detail)| ParityWarning {
            code: code.to_owned(),
            detail: detail.to_owned(),
        })
        .collect();

    Ok(ParityReport {
        schema_version: super::PARITY_SCHEMA_VERSION,
        expected_source: expected.source,
        actual_source: actual.source,
        compatible: differences.is_empty(),
        differences,
        warnings,
    })
}

fn compare_table(
    expected: &TableProjection,
    actual: &TableProjection,
    differences: &mut Vec<Difference>,
) {
    let table = qualified_name(&expected.schema, &expected.name);
    if expected.primary_key != actual.primary_key {
        differences.push(Difference {
            code: DifferenceCode::PrimaryKeyMismatch,
            table: table.clone(),
            column: None,
            expected: Some(expected.primary_key.join(",")),
            actual: Some(actual.primary_key.join(",")),
        });
    }

    let expected_columns: BTreeMap<_, _> = expected
        .columns
        .iter()
        .map(|column| (column.name.as_str(), column))
        .collect();
    let actual_columns: BTreeMap<_, _> = actual
        .columns
        .iter()
        .map(|column| (column.name.as_str(), column))
        .collect();
    let all_columns: BTreeSet<_> = expected_columns
        .keys()
        .chain(actual_columns.keys())
        .copied()
        .collect();

    for name in all_columns {
        match (expected_columns.get(name), actual_columns.get(name)) {
            (Some(expected_column), Some(actual_column)) => {
                compare_column(&table, expected_column, actual_column, differences)
            }
            (Some(_), None) => differences.push(Difference {
                code: DifferenceCode::MissingColumn,
                table: table.clone(),
                column: Some(name.to_owned()),
                expected: Some("present".to_owned()),
                actual: Some("missing".to_owned()),
            }),
            (None, Some(_)) => differences.push(Difference {
                code: DifferenceCode::ExtraColumn,
                table: table.clone(),
                column: Some(name.to_owned()),
                expected: Some("missing".to_owned()),
                actual: Some("present".to_owned()),
            }),
            (None, None) => unreachable!("column key came from one projection"),
        }
    }
}

fn compare_column(
    table: &str,
    expected: &ColumnProjection,
    actual: &ColumnProjection,
    differences: &mut Vec<Difference>,
) {
    if expected.ordinal != actual.ordinal {
        differences.push(Difference {
            code: DifferenceCode::ColumnOrdinalMismatch,
            table: table.to_owned(),
            column: Some(expected.name.clone()),
            expected: Some(expected.ordinal.to_string()),
            actual: Some(actual.ordinal.to_string()),
        });
    }
    if expected.type_family != actual.type_family {
        differences.push(Difference {
            code: DifferenceCode::TypeFamilyMismatch,
            table: table.to_owned(),
            column: Some(expected.name.clone()),
            expected: Some(expected.type_family.clone()),
            actual: Some(actual.type_family.clone()),
        });
    }
    if expected.nullable != actual.nullable {
        differences.push(Difference {
            code: DifferenceCode::NullabilityMismatch,
            table: table.to_owned(),
            column: Some(expected.name.clone()),
            expected: Some(expected.nullable.to_string()),
            actual: Some(actual.nullable.to_string()),
        });
    }
}

fn qualified_name(schema: &str, table: &str) -> String {
    format!("{schema}.{table}")
}
