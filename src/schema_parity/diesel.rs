use std::collections::BTreeMap;

use super::{
    ColumnProjection, DatabaseEngine, DatabaseIdentity, GeneratorIdentity, ParityError,
    ProjectionSource, SchemaProjection, TableProjection, PARITY_SCHEMA_VERSION,
    normalize_type_family,
};

pub fn parse_diesel_schema(
    source: &str,
    engine: DatabaseEngine,
    default_schema: &str,
    generator_version: &str,
) -> Result<SchemaProjection, ParityError> {
    if source.len() > 32 * 1024 * 1024 {
        return Err(ParityError::InvalidGeneratedSource);
    }
    let lines: Vec<_> = source.lines().collect();
    let mut tables = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if is_table_macro_start(lines[index]) {
            let (table, next) = parse_table_macro(&lines, index + 1, default_schema)?;
            tables.push(table);
            index = next;
        } else {
            index += 1;
        }
    }
    SchemaProjection {
        schema_version: PARITY_SCHEMA_VERSION,
        source: ProjectionSource::DieselSchema,
        generator: GeneratorIdentity {
            name: "diesel-print-schema".to_owned(),
            version: generator_version.to_owned(),
        },
        database: DatabaseIdentity {
            engine,
            schema: default_schema.to_owned(),
        },
        tables,
    }
    .normalize()
}

fn parse_table_macro(
    lines: &[&str],
    mut index: usize,
    default_schema: &str,
) -> Result<(TableProjection, usize), ParityError> {
    let mut table_sql_name = None;
    while index < lines.len() {
        let line = clean(lines[index]);
        if line.is_empty() || line.starts_with("use ") {
            index += 1;
            continue;
        }
        if line.starts_with("#[") {
            table_sql_name = parse_sql_name(line).or(table_sql_name);
            index += 1;
            continue;
        }
        if line.contains('(') && line.ends_with('{') && !line.contains("->") {
            let (schema, rust_table_name, primary_key) = parse_table_header(line, default_schema)?;
            let table_name = table_sql_name.unwrap_or(rust_table_name);
            let mut columns = Vec::new();
            let mut column_names = BTreeMap::new();
            let mut column_sql_name = None;
            index += 1;
            while index < lines.len() {
                let line = clean(lines[index]);
                if line == "}" {
                    let primary_key = primary_key
                        .into_iter()
                        .map(|key| {
                            column_names
                                .get(&key)
                                .cloned()
                                .ok_or(ParityError::UnknownPrimaryKeyColumn)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok((
                        TableProjection {
                            schema,
                            name: table_name,
                            primary_key,
                            columns,
                        },
                        index + 1,
                    ));
                }
                if line.starts_with("#[") {
                    column_sql_name = parse_sql_name(line).or(column_sql_name);
                } else if let Some((rust_name, generated_type)) = parse_column(line) {
                    let name = column_sql_name
                        .take()
                        .unwrap_or_else(|| strip_raw_identifier(rust_name).to_owned());
                    let (nullable, inner_type) = peel_nullable(generated_type);
                    let type_family = normalize_generator_type(inner_type)?;
                    column_names.insert(strip_raw_identifier(rust_name).to_owned(), name.clone());
                    columns.push(ColumnProjection {
                        name,
                        ordinal: u32::try_from(columns.len() + 1)
                            .map_err(|_| ParityError::InvalidOrdinal)?,
                        type_family,
                        nullable,
                        native_type: Some(generated_type.to_owned()),
                    });
                } else if !line.is_empty() {
                    return Err(ParityError::InvalidGeneratedSource);
                }
                index += 1;
            }
            return Err(ParityError::InvalidGeneratedSource);
        }
        return Err(ParityError::InvalidGeneratedSource);
    }
    Err(ParityError::InvalidGeneratedSource)
}

fn parse_table_header(line: &str, default_schema: &str) -> Result<(String, String, Vec<String>), ParityError> {
    let open = line.find('(').ok_or(ParityError::InvalidGeneratedSource)?;
    let close = line[open + 1..]
        .find(')')
        .map(|position| position + open + 1)
        .ok_or(ParityError::InvalidGeneratedSource)?;
    let qualified = line[..open].trim();
    let (schema, table) = qualified
        .split_once('.')
        .map(|(schema, table)| (strip_raw_identifier(schema), strip_raw_identifier(table)))
        .unwrap_or((default_schema, strip_raw_identifier(qualified)));
    let primary_key = line[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(strip_raw_identifier)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if schema.is_empty() || table.is_empty() || primary_key.is_empty() {
        return Err(ParityError::InvalidGeneratedSource);
    }
    Ok((schema.to_owned(), table.to_owned(), primary_key))
}

fn parse_column(line: &str) -> Option<(&str, &str)> {
    let (name, generated_type) = line.split_once("->")?;
    let name = name.trim();
    let generated_type = generated_type.trim().trim_end_matches(',').trim();
    (!name.is_empty() && !generated_type.is_empty()).then_some((name, generated_type))
}

fn peel_nullable(value: &str) -> (bool, &str) {
    peel_wrapper(value, "Nullable")
        .map(|inner| (true, inner))
        .unwrap_or((false, value.trim()))
}

fn normalize_generator_type(value: &str) -> Result<String, ParityError> {
    let compact = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if let Some(inner) = peel_wrapper(&compact, "Array") {
        return Ok(format!("array<{}>", normalize_generator_type(inner)?));
    }
    if let Some(inner) = peel_wrapper(&compact, "Nullable") {
        return normalize_generator_type(inner);
    }
    let leaf = compact.rsplit("::").next().unwrap_or(&compact);
    normalize_type_family(leaf)
}

fn peel_wrapper<'a>(value: &'a str, wrapper: &str) -> Option<&'a str> {
    let value = value.trim();
    let prefix = format!("{wrapper}<");
    value.strip_prefix(&prefix).and_then(|inner| inner.strip_suffix('>'))
}

fn parse_sql_name(attribute: &str) -> Option<String> {
    if !attribute.contains("sql_name") {
        return None;
    }
    let start = attribute.find('"')? + 1;
    let end = attribute[start..].find('"')? + start;
    (end > start).then(|| attribute[start..end].to_owned())
}

fn is_table_macro_start(line: &str) -> bool {
    let compact = clean(line).chars().filter(|character| !character.is_whitespace()).collect::<String>();
    compact.ends_with("table!{")
}

fn clean(line: &str) -> &str {
    line.split_once("//").map(|(before, _)| before).unwrap_or(line).trim()
}

fn strip_raw_identifier(value: &str) -> &str {
    value.trim().strip_prefix("r#").unwrap_or(value.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_diesel_tables_attributes_arrays_and_nullability() {
        let source = r#"
            diesel::table! {
                use diesel::sql_types::*;

                public.embedding_models (tenant_id, id) {
                    tenant_id -> Uuid,
                    id -> Uuid,
                    #[sql_name = "type"]
                    type_ -> Text,
                    dimensions -> Int4,
                    aliases -> Array<Nullable<Text>>,
                    notes -> Nullable<Text>,
                }
            }
        "#;
        let parsed = parse_diesel_schema(source, DatabaseEngine::PostgreSql, "public", "2.3.12")
            .expect("valid Diesel projection");
        assert_eq!(parsed.tables.len(), 1);
        let table = &parsed.tables[0];
        assert_eq!(table.primary_key, ["tenant_id", "id"]);
        assert_eq!(table.columns[2].name, "type");
        assert_eq!(table.columns[4].type_family, "array<text>");
        assert!(table.columns[5].nullable);
    }

    #[test]
    fn refuses_incomplete_macro() {
        assert_eq!(
            parse_diesel_schema(
                "diesel::table! { users (id) { id -> Int4,",
                DatabaseEngine::PostgreSql,
                "public",
                "2.3.12"
            ),
            Err(ParityError::InvalidGeneratedSource)
        );
    }
}
