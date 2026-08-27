#![forbid(unsafe_code)]

use crate::error::CoreError;
use crate::flavor::DatabaseFlavor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreConfig {
    pub database_url: String,
    pub flavor: DatabaseFlavor,
    pub read_only: bool,
}

impl CoreConfig {
    pub fn from_env() -> Result<Self, CoreError> {
        let database_url = std::env::var("DECLMIG_DATABASE_URL")
            .map_err(|_| CoreError::InvalidDatabaseUrl)
            .and_then(|url| {
                (url.starts_with("postgres://") || url.starts_with("postgresql://"))
                    .then_some(url)
                    .ok_or(CoreError::InvalidDatabaseUrl)
            })?;
        let flavor = match database_url.contains("cockroach") {
            true => DatabaseFlavor::CockroachDb,
            false => DatabaseFlavor::PostgreSql,
        };
        Ok(Self {
            database_url,
            flavor,
            read_only: std::env::var("DECLMIG_DB_READ_ONLY").ok().as_deref() != Some("0"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_classifies_url_and_read_only_flag() {
        std::env::set_var("DECLMIG_DATABASE_URL", "postgres://localhost/app");
        std::env::remove_var("DECLMIG_DB_READ_ONLY");
        let config = CoreConfig::from_env().expect("postgres url");
        assert_eq!(config.flavor, DatabaseFlavor::PostgreSql);
        assert!(config.read_only);

        std::env::set_var("DECLMIG_DATABASE_URL", "postgresql://cockroach.example/db");
        std::env::set_var("DECLMIG_DB_READ_ONLY", "0");
        let config = CoreConfig::from_env().expect("cockroach url");
        assert_eq!(config.flavor, DatabaseFlavor::CockroachDb);
        assert!(!config.read_only);

        std::env::set_var("DECLMIG_DATABASE_URL", "mysql://localhost/db");
        assert!(CoreConfig::from_env().is_err());
    }
}
