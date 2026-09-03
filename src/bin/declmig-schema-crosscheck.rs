use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use declmig_lib_core::schema_parity::{
    compare, parse_diesel_schema, parse_seaorm_directory, read_json, write_json, DatabaseEngine,
    ParityError, SchemaProjection,
};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(ExitDisposition::Compatible) => ExitCode::SUCCESS,
        Ok(ExitDisposition::Drift) => ExitCode::from(2),
        Err(error) => {
            eprintln!("declmig-schema-crosscheck: {error}");
            ExitCode::from(64)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExitDisposition {
    Compatible,
    Drift,
}

fn run(arguments: Vec<String>) -> Result<ExitDisposition, ParityError> {
    let (command, options) = parse_arguments(arguments)?;
    match command.as_str() {
        "parse-diesel" => {
            require_only(
                &options,
                &["input", "output", "engine", "schema", "generator-version"],
            )?;
            let input = required_path(&options, "input")?;
            let output = required_path(&options, "output")?;
            let source = read_text(&input)?;
            let projection = parse_diesel_schema(
                &source,
                DatabaseEngine::parse(required(&options, "engine")?)?,
                required(&options, "schema")?,
                required(&options, "generator-version")?,
            )?;
            write_json(&output, &projection)?;
            Ok(ExitDisposition::Compatible)
        }
        "parse-seaorm" => {
            require_only(
                &options,
                &[
                    "input-dir",
                    "output",
                    "engine",
                    "schema",
                    "generator-version",
                ],
            )?;
            let projection = parse_seaorm_directory(
                &required_path(&options, "input-dir")?,
                DatabaseEngine::parse(required(&options, "engine")?)?,
                required(&options, "schema")?,
                required(&options, "generator-version")?,
            )?;
            write_json(&required_path(&options, "output")?, &projection)?;
            Ok(ExitDisposition::Compatible)
        }
        "compare" => {
            require_only(&options, &["expected", "actual", "output"])?;
            let expected: SchemaProjection = read_json(&required_path(&options, "expected")?)?;
            let actual: SchemaProjection = read_json(&required_path(&options, "actual")?)?;
            let report = compare(expected, actual)?;
            write_json(&required_path(&options, "output")?, &report)?;
            Ok(if report.compatible {
                ExitDisposition::Compatible
            } else {
                ExitDisposition::Drift
            })
        }
        "validate" => {
            require_only(&options, &["input"])?;
            let projection: SchemaProjection = read_json(&required_path(&options, "input")?)?;
            projection.normalize()?;
            Ok(ExitDisposition::Compatible)
        }
        _ => Err(ParityError::InvalidArgument),
    }
}

fn parse_arguments(
    arguments: Vec<String>,
) -> Result<(String, BTreeMap<String, String>), ParityError> {
    let mut iter = arguments.into_iter();
    let command = iter.next().ok_or(ParityError::InvalidArgument)?;
    let mut options = BTreeMap::new();
    while let Some(flag) = iter.next() {
        let key = flag
            .strip_prefix("--")
            .filter(|key| !key.is_empty())
            .ok_or(ParityError::InvalidArgument)?;
        let value = iter.next().ok_or(ParityError::InvalidArgument)?;
        if value.starts_with("--") || options.insert(key.to_owned(), value).is_some() {
            return Err(ParityError::InvalidArgument);
        }
    }
    Ok((command, options))
}

fn require_only(options: &BTreeMap<String, String>, allowed: &[&str]) -> Result<(), ParityError> {
    let allowed: BTreeSet<_> = allowed.iter().copied().collect();
    if options.keys().all(|key| allowed.contains(key.as_str())) {
        Ok(())
    } else {
        Err(ParityError::InvalidArgument)
    }
}

fn required<'a>(options: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, ParityError> {
    options
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(ParityError::InvalidArgument)
}

fn required_path(options: &BTreeMap<String, String>, key: &str) -> Result<PathBuf, ParityError> {
    let value = required(options, key)?;
    let path = PathBuf::from(value);
    if path.as_os_str().is_empty() {
        Err(ParityError::InvalidPath)
    } else {
        Ok(path)
    }
}

fn read_text(path: &Path) -> Result<String, ParityError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ParityError::InvalidPath)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 32 * 1024 * 1024
    {
        return Err(ParityError::InvalidPath);
    }
    Ok(fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_are_pairs_and_duplicates_fail_closed() {
        assert!(parse_arguments(vec!["validate".into(), "--input".into(), "x".into()]).is_ok());
        assert_eq!(
            parse_arguments(vec![
                "validate".into(),
                "--input".into(),
                "x".into(),
                "--input".into(),
                "y".into(),
            ]),
            Err(ParityError::InvalidArgument)
        );
        assert_eq!(
            parse_arguments(vec!["validate".into(), "input".into(), "x".into()]),
            Err(ParityError::InvalidArgument)
        );
    }
}
