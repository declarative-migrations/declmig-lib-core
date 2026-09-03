#![forbid(unsafe_code)]

/// Logical schema revision packaged by this crate.
pub const SCHEMA_REVISION: &str = "declmig-0002";

/// Versioned peer-authority evidence required before packaging a schema release.
pub const PEER_AUTHORITY_CERTIFICATION_FORMAT: &str =
    "declmig.peer-authority-certification/v1";

/// A schema release is ineligible without all-pass peer comparison evidence.
pub const SCHEMA_RELEASE_REQUIRES_PEER_PARITY: bool = true;

/// `lib-core` never elects a contract source or ORM when peers disagree.
pub const AUTOMATIC_AUTHORITY_WINNER_ALLOWED: bool = false;

/// Fail-closed policy consumed by schema-release packaging and validation code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemaReleasePolicy {
    pub peer_authority_certification_format: &'static str,
    pub requires_peer_parity: bool,
    pub automatic_authority_winner_allowed: bool,
}

/// Returns the immutable policy for the current schema-release format.
#[must_use]
pub const fn schema_release_policy() -> SchemaReleasePolicy {
    SchemaReleasePolicy {
        peer_authority_certification_format: PEER_AUTHORITY_CERTIFICATION_FORMAT,
        requires_peer_parity: SCHEMA_RELEASE_REQUIRES_PEER_PARITY,
        automatic_authority_winner_allowed: AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        schema_release_policy, SchemaReleasePolicy, PEER_AUTHORITY_CERTIFICATION_FORMAT,
        SCHEMA_REVISION,
    };

    #[test]
    fn schema_release_policy_is_fail_closed() {
        assert_eq!(SCHEMA_REVISION, "declmig-0002");
        assert_eq!(
            schema_release_policy(),
            SchemaReleasePolicy {
                peer_authority_certification_format: PEER_AUTHORITY_CERTIFICATION_FORMAT,
                requires_peer_parity: true,
                automatic_authority_winner_allowed: false,
            }
        );
    }
}
