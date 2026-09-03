use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParityError {
    DuplicateColumn,
    DuplicatePrimaryKeyColumn,
    DuplicateTable,
    EmptyProjection,
    EmptyTable,
    InvalidArgument,
    InvalidEngine,
    InvalidGeneratedSource,
    InvalidJson,
    InvalidOrdinal,
    InvalidPath,
    InvalidText,
    InvalidTypeFamily,
    Io,
    UnknownPrimaryKeyColumn,
    UnsupportedSchemaVersion,
}

impl fmt::Display for ParityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ParityError {}

impl From<std::io::Error> for ParityError {
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}

impl From<serde_json::Error> for ParityError {
    fn from(_: serde_json::Error) -> Self {
        Self::InvalidJson
    }
}
