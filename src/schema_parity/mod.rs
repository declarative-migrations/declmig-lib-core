mod compare;
mod diesel;
mod error;
mod io;
mod model;
mod seaorm;

pub use compare::{Difference, DifferenceCode, ParityReport, ParityWarning, compare};
pub use diesel::parse_diesel_schema;
pub use error::ParityError;
pub use io::{read_json, write_json};
pub use model::{
    ColumnProjection, DatabaseEngine, DatabaseIdentity, GeneratorIdentity, PARITY_SCHEMA_VERSION,
    ProjectionSource, SchemaProjection, TableProjection, normalize_type_family,
};
pub use seaorm::{parse_seaorm_directory, parse_seaorm_entity};
