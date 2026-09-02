use std::fs;
use std::path::{Path, PathBuf};

use super::{
    normalize_type_family, ColumnProjection, DatabaseEngine, DatabaseIdentity, GeneratorIdentity,
    ParityError, ProjectionSource, SchemaProjection, TableProjection, PARITY_SCHEMA_VERSION,
};

pub fn parse_seaorm_directory(
    root: &Path,
    engine: DatabaseEngine,
    default_schema: &str,
    generator_version: &str,
) -> Result<SchemaProjection, ParityError> {
    let root = root.canonicalize().map_err(|_| ParityError::InvalidPath)?;
    if !root.is_dir() {
        return Err(ParityError::InvalidPath);
    }
    let mut files = Vec::new();
    collect_rust_files(&root, &root, &mut files)?;
    files.sort();
    if files.len() > 10_000 {
        return Err(ParityError::InvalidGeneratedSource);
    }

    let mut tables = Vec::new();
    for file in files {
        let source = fs::read_to_string(file)?;
        if source.len() > 32 * 1024 * 1024 {
            return Err(ParityError::InvalidGeneratedSource);
        }
        if let Some(table) = parse_seaorm_entity(&source, default_schema)? {
            tables.push(table);
        }
    }
    SchemaProjection {
        schema_version: PARITY_SCHEMA_VERSION,
        source: ProjectionSource::SeaOrmEntity,
        generator: GeneratorIdentity {
            name: "sea-orm-cli-generate-entity".to_owned(),
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

pub fn parse_seaorm_entity(
    source: &str,
    default_schema: &str,
) -> Result<Option<TableProjection>, ParityError> {
    let lines: Vec<_> = source.lines().collect();
    let mut index = 0;
    let mut model_attributes = Vec::new();
    while index < lines.len() {
        let line = clean(lines[index]);
        if line.starts_with("#[sea_orm(") {
            let (attribute, next) = collect_attribute(&lines, index)?;
            model_attributes.push(attribute);
            index = next;
            continue;
        }
        if line.starts_with("pub struct Model") {
            let table_name = model_attributes
                .iter()
                .find_map(|attribute| attribute_string(attribute, "table_name"));
            let Some(table_name) = table_name else {
                return Ok(None);
            };
            let schema = model_attributes
                .iter()
                .find_map(|attribute| attribute_string(attribute, "schema_name"))
                .unwrap_or_else(|| default_schema.to_owned());
            let mut columns = Vec::new();
            let mut primary_key = Vec::new();
            let mut field_attributes = Vec::new();
            index += 1;
            while index < lines.len() {
                let line = clean(lines[index]);
                if line == "}" {
                    return Ok(Some(TableProjection {
                        schema,
                        name: table_name,
                        primary_key,
                        columns,
                    }));
                }
                if line.starts_with("#[sea_orm(") {
                    let (attribute, next) = collect_attribute(&lines, index)?;
                    field_attributes.push(attribute);
                    index = next;
                    continue;
                }
                if line.starts_with("#[") {
                    index += 1;
                    continue;
                }
                if line.starts_with("pub ") {
                    let (rust_name, rust_type) = parse_field(line)?;
                    let column_name = field_attributes
                        .iter()
                        .find_map(|attribute| attribute_string(attribute, "column_name"))
                        .unwrap_or_else(|| strip_raw_identifier(rust_name).to_owned());
                    if field_attributes
                        .iter()
                        .any(|attribute| attribute_flag(attribute, "primary_key"))
                    {
                        primary_key.push(column_name.clone());
                    }
                    let column_type = field_attributes
                        .iter()
                        .find_map(|attribute| attribute_string(attribute, "column_type"));
                    let (nullable, inner_type) = peel_option(rust_type);
                    let type_family = normalize_seaorm_type(column_type.as_deref(), inner_type)?;
                    columns.push(ColumnProjection {
                        name: column_name,
                        ordinal: u32::try_from(columns.len() + 1)
                            .map_err(|_| ParityError::InvalidOrdinal)?,
                        type_family,
                        nullable,
                        native_type: Some(
                            column_type
                                .map(|value| format!("{value}:{rust_type}"))
                                .unwrap_or_else(|| rust_type.to_owned()),
                        ),
                    });
                    field_attributes.clear();
                } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with("///") {
                    return Err(ParityError::InvalidGeneratedSource);
                }
                index += 1;
            }
            return Err(ParityError::InvalidGeneratedSource);
        }
        if !line.is_empty()
            && !line.starts_with("//")
            && !line.starts_with("#[")
            && !line.starts_with("pub struct Model")
        {
            model_attributes.clear();
        }
        index += 1;
    }
    Ok(None)
}

fn collect_rust_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), ParityError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(ParityError::InvalidPath);
        }
        if metadata.is_dir() {
            collect_rust_files(root, &path, output)?;
        } else if metadata.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("rs")
        {
            let canonical = path.canonicalize().map_err(|_| ParityError::InvalidPath)?;
            if !canonical.starts_with(root) {
                return Err(ParityError::InvalidPath);
            }
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if !matches!(name, "mod.rs" | "prelude.rs") {
                output.push(canonical);
            }
        }
    }
    Ok(())
}

fn collect_attribute(lines: &[&str], start: usize) -> Result<(String, usize), ParityError> {
    let mut attribute = String::new();
    let mut index = start;
    while index < lines.len() {
        if !attribute.is_empty() {
            attribute.push(' ');
        }
        attribute.push_str(clean(lines[index]));
        index += 1;
        if attribute.ends_with(")]") {
            return Ok((attribute, index));
        }
        if attribute.len() > 16 * 1024 {
            return Err(ParityError::InvalidGeneratedSource);
        }
    }
    Err(ParityError::InvalidGeneratedSource)
}

fn parse_field(line: &str) -> Result<(&str, &str), ParityError> {
    let field = line
        .strip_prefix("pub ")
        .ok_or(ParityError::InvalidGeneratedSource)?
        .trim_end_matches(',')
        .trim();
    let (name, rust_type) = field
        .split_once(':')
        .ok_or(ParityError::InvalidGeneratedSource)?;
    let name = name.trim();
    let rust_type = rust_type.trim();
    if name.is_empty() || rust_type.is_empty() {
        Err(ParityError::InvalidGeneratedSource)
    } else {
        Ok((name, rust_type))
    }
}

fn peel_option(value: &str) -> (bool, &str) {
    peel_wrapper(value, "Option")
        .map(|inner| (true, inner))
        .unwrap_or((false, value.trim()))
}

fn normalize_seaorm_type(
    column_type: Option<&str>,
    rust_type: &str,
) -> Result<String, ParityError> {
    if let Some(column_type) = column_type {
        return normalize_seaorm_column_type(column_type);
    }
    let compact = rust_type
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if let Some(inner) = peel_wrapper(&compact, "Vec") {
        if inner == "u8" {
            return normalize_type_family("bytea");
        }
        let (_, inner) = peel_option(inner);
        return Ok(format!("array<{}>", normalize_seaorm_type(None, inner)?));
    }
    let leaf = compact.rsplit("::").next().unwrap_or(&compact);
    normalize_type_family(leaf)
}

fn normalize_seaorm_column_type(value: &str) -> Result<String, ParityError> {
    let value = value.trim();
    if let Some(inner) = peel_parenthesized(value, "Array") {
        return Ok(format!("array<{}>", normalize_seaorm_column_type(inner)?));
    }
    if value.starts_with("Custom(") {
        let custom = quoted_values(value)
            .into_iter()
            .next()
            .unwrap_or_else(|| "unknown".to_owned());
        return normalize_type_family(&custom);
    }
    let head = value.split(['(', '<']).next().unwrap_or(value).trim();
    normalize_type_family(head)
}

fn peel_parenthesized<'a>(value: &'a str, wrapper: &str) -> Option<&'a str> {
    let prefix = format!("{wrapper}(");
    value
        .strip_prefix(&prefix)
        .and_then(|inner| inner.strip_suffix(')'))
}

fn attribute_string(attribute: &str, key: &str) -> Option<String> {
    let position = attribute.find(key)?;
    let tail = &attribute[position + key.len()..];
    let equal = tail.find('=')?;
    let tail = &tail[equal + 1..];
    let mut escaped = false;
    let mut started = false;
    let mut output = String::new();
    for character in tail.chars() {
        if !started {
            if character == '"' {
                started = true;
            }
            continue;
        }
        if escaped {
            output.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(output);
        } else {
            output.push(character);
        }
    }
    None
}

fn attribute_flag(attribute: &str, key: &str) -> bool {
    attribute
        .split(|character: char| matches!(character, '#' | '[' | ']' | '(' | ')' | ','))
        .any(|part| part.trim() == key)
}

fn quoted_values(value: &str) -> Vec<String> {
    let mut output = Vec::new();
    let mut remainder = value;
    while let Some(start) = remainder.find('"') {
        remainder = &remainder[start + 1..];
        let Some(end) = remainder.find('"') else {
            break;
        };
        output.push(remainder[..end].to_owned());
        remainder = &remainder[end + 1..];
    }
    output
}

fn peel_wrapper<'a>(value: &'a str, wrapper: &str) -> Option<&'a str> {
    let value = value.trim();
    let prefix = format!("{wrapper}<");
    value
        .strip_prefix(&prefix)
        .and_then(|inner| inner.strip_suffix('>'))
}

fn clean(line: &str) -> &str {
    line.trim()
}

fn strip_raw_identifier(value: &str) -> &str {
    value.trim().strip_prefix("r#").unwrap_or(value.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_entity_with_schema_names_and_types() {
        let source = r#"
            use sea_orm::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
            #[sea_orm(table_name = "embedding_models", schema_name = "public")]
            pub struct Model {
                #[sea_orm(primary_key, auto_increment = false)]
                pub tenant_id: Uuid,
                #[sea_orm(primary_key, auto_increment = false)]
                pub id: Uuid,
                #[sea_orm(column_name = "type", column_type = "Text")]
                pub type_: String,
                pub dimensions: i32,
                #[sea_orm(column_type = "Array(String(None))")]
                pub aliases: Vec<Option<String>>,
                #[sea_orm(column_type = "Text", nullable)]
                pub notes: Option<String>,
            }
        "#;
        let table = parse_seaorm_entity(source, "public")
            .expect("parse")
            .expect("entity");
        assert_eq!(table.name, "embedding_models");
        assert_eq!(table.primary_key, ["tenant_id", "id"]);
        assert_eq!(table.columns[2].name, "type");
        assert_eq!(table.columns[2].type_family, "text");
        assert_eq!(table.columns[4].type_family, "array<varchar>");
        assert!(table.columns[5].nullable);
    }

    #[test]
    fn ignores_non_entity_source() {
        assert_eq!(
            parse_seaorm_entity("pub fn helper() {}", "public"),
            Ok(None)
        );
    }
}
