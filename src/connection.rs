#![forbid(unsafe_code)]

use crate::config::CoreConfig;
use crate::error::CoreError;
use crate::schema::SCHEMA_REVISION;

#[derive(Clone, Debug)]
pub struct CorePool {
    pub flavor: crate::flavor::DatabaseFlavor,
    read_only: bool,
}

impl CorePool {
    pub fn connect(config: &CoreConfig) -> Result<Self, CoreError> {
        Ok(Self {
            flavor: config.flavor,
            read_only: config.read_only,
        })
    }

    pub fn assert_schema(&self, found: &str) -> Result<(), CoreError> {
        match found == SCHEMA_REVISION {
            true => Ok(()),
            false => Err(CoreError::SchemaRevision {
                required: SCHEMA_REVISION.to_string(),
                found: found.to_string(),
            }),
        }
    }

    pub fn migrate(&self) -> Result<(), CoreError> {
        match self.read_only {
            true => Err(CoreError::WritesDisabled),
            false => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flavor::DatabaseFlavor;

    fn pool(read_only: bool) -> CorePool {
        CorePool::connect(&CoreConfig {
            database_url: "postgres://localhost/db".into(),
            flavor: DatabaseFlavor::PostgreSql,
            read_only,
        })
        .expect("connect")
    }

    #[test]
    fn schema_and_migrate_use_exhaustive_matches() {
        let read_only = pool(true);
        assert!(read_only.assert_schema(SCHEMA_REVISION).is_ok());
        assert!(read_only.assert_schema("other").is_err());
        assert!(read_only.migrate().is_err());
        assert!(pool(false).migrate().is_ok());
    }
}
