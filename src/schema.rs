#![forbid(unsafe_code)]

/// Logical schema revision packaged by this crate.
pub const SCHEMA_REVISION: &str = "declmig-0001";

/// Versioned peer-authority evidence required before packaging a schema release.
pub const PEER_AUTHORITY_CERTIFICATION_FORMAT: &str = "declmig.peer-authority-certification/v1";

/// `lib-core` never elects a contract source or ORM when peers disagree.
pub const AUTOMATIC_AUTHORITY_WINNER_ALLOWED: bool = false;

/// A schema release is ineligible without all-pass peer comparison evidence.
pub const SCHEMA_RELEASE_REQUIRES_PEER_PARITY: bool = true;

/// Runtime descriptor consumed by schema-release packaging and validation code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemaReleasePolicy {
    pub peer_authority_certification_format: &'static str,
    pub requires_peer_parity: bool,
    pub automatic_authority_winner_allowed: bool,
}

/// Returns the fail-closed schema-release policy.
#[must_use]
pub fn schema_release_policy() -> SchemaReleasePolicy {
    SchemaReleasePolicy {
        peer_authority_certification_format: PEER_AUTHORITY_CERTIFICATION_FORMAT,
        requires_peer_parity: SCHEMA_RELEASE_REQUIRES_PEER_PARITY,
        automatic_authority_winner_allowed: AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    }
}

#[cfg(test)]
mod tests {
    use super::{PEER_AUTHORITY_CERTIFICATION_FORMAT, SchemaReleasePolicy, schema_release_policy};

    #[test]
    fn schema_release_policy_is_fail_closed() {
        let policy = schema_release_policy();
        assert_eq!(
            policy,
            SchemaReleasePolicy {
                peer_authority_certification_format: PEER_AUTHORITY_CERTIFICATION_FORMAT,
                requires_peer_parity: true,
                automatic_authority_winner_allowed: false,
            }
        );
    }
}
