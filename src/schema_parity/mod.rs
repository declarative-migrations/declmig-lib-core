mod compare;
mod diesel;
mod error;
mod io;
mod model;
mod seaorm;

pub use compare::{compare, Difference, DifferenceCode, ParityReport, ParityWarning};
pub use diesel::parse_diesel_schema;
pub use error::ParityError;
pub use io::{read_json, write_json};
pub use model::{
    normalize_type_family, ColumnProjection, DatabaseEngine, DatabaseIdentity, GeneratorIdentity,
    ProjectionSource, SchemaProjection, TableProjection, PARITY_SCHEMA_VERSION,
};
pub use seaorm::{parse_seaorm_directory, parse_seaorm_entity};
