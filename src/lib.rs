#![forbid(unsafe_code)]

pub mod config;
pub mod connection;
pub mod error;
pub mod flavor;
pub mod schema;

pub use config::CoreConfig;
pub use connection::CorePool;
pub use error::CoreError;
pub use flavor::DatabaseFlavor;
pub use schema::{
    schema_release_policy, SchemaReleasePolicy, AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    PEER_AUTHORITY_CERTIFICATION_FORMAT, SCHEMA_RELEASE_REQUIRES_PEER_PARITY, SCHEMA_REVISION,
};
