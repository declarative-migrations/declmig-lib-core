#![forbid(unsafe_code)]

/// Logical schema revision packaged by this crate.
pub const SCHEMA_REVISION: &str = "declmig-0001";

/// Versioned peer-authority evidence required before packaging a schema release.
pub const PEER_AUTHORITY_CERTIFICATION_FORMAT: &str = "declmig.peer-authority-certification/v1";

/// `lib-core` never elects a contract source or ORM when peers disagree.
pub const AUTOMATIC_AUTHORITY_WINNER_ALLOWED: bool = false;

/// A schema release is ineligible without all-pass peer comparison evidence.
pub const SCHEMA_RELEASE_REQUIRES_PEER_PARITY: bool = true;

#[cfg(test)]
mod tests {
    use super::{
        AUTOMATIC_AUTHORITY_WINNER_ALLOWED, PEER_AUTHORITY_CERTIFICATION_FORMAT,
        SCHEMA_RELEASE_REQUIRES_PEER_PARITY,
    };

    #[test]
    fn schema_release_policy_is_fail_closed() {
        assert_eq!(
            PEER_AUTHORITY_CERTIFICATION_FORMAT,
            "declmig.peer-authority-certification/v1"
        );
        assert!(SCHEMA_RELEASE_REQUIRES_PEER_PARITY);
        assert!(!AUTOMATIC_AUTHORITY_WINNER_ALLOWED);
    }
}
