#![forbid(unsafe_code)]

/// Logical schema revision packaged by this crate.
pub const SCHEMA_REVISION: &str = "declmig-0002";

/// Versioned evidence envelope required before a schema release can be packaged.
pub const PEER_AUTHORITY_CERTIFICATION_FORMAT: &str = "declmig.peer-authority-certification/v1";

/// Stable terminal state for any missing, invalid, or unequal peer evidence.
pub const DISCREPANCY_STATUS: &str = "STOPPED_FOR_EVALUATION";

/// A schema release is ineligible unless every peer comparison is complete.
pub const SCHEMA_RELEASE_REQUIRES_PEER_PARITY: bool = true;

/// No compiler, source lane, catalog, or ORM may win automatically on mismatch.
pub const AUTOMATIC_AUTHORITY_WINNER_ALLOWED: bool = false;

/// Every release certificate must bind all independently produced projections.
pub const REQUIRED_CONVERGENCE_PARTICIPANTS: [&str; 6] = [
    "typespec-sql-catalog",
    "json-schema-openapi-sql-catalog",
    "reviewed-dpm-desired-catalog",
    "diesel-projection",
    "seaorm-projection",
    "shadow-live-catalog-readback",
];

/// Runtime descriptor consumed by package and release validation code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemaReleasePolicy {
    pub peer_authority_certification_format: &'static str,
    pub discrepancy_status: &'static str,
    pub requires_peer_parity: bool,
    pub automatic_authority_winner_allowed: bool,
    pub required_convergence_participants: &'static [&'static str],
}

/// Returns the immutable, fail-closed schema-release policy.
#[must_use]
pub const fn schema_release_policy() -> SchemaReleasePolicy {
    SchemaReleasePolicy {
        peer_authority_certification_format: PEER_AUTHORITY_CERTIFICATION_FORMAT,
        discrepancy_status: DISCREPANCY_STATUS,
        requires_peer_parity: SCHEMA_RELEASE_REQUIRES_PEER_PARITY,
        automatic_authority_winner_allowed: AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
        required_convergence_participants: &REQUIRED_CONVERGENCE_PARTICIPANTS,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        schema_release_policy, SchemaReleasePolicy, AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
        DISCREPANCY_STATUS, PEER_AUTHORITY_CERTIFICATION_FORMAT, REQUIRED_CONVERGENCE_PARTICIPANTS,
        SCHEMA_RELEASE_REQUIRES_PEER_PARITY,
    };

    #[test]
    fn schema_release_policy_is_fail_closed() {
        assert_eq!(
            schema_release_policy(),
            SchemaReleasePolicy {
                peer_authority_certification_format: PEER_AUTHORITY_CERTIFICATION_FORMAT,
                discrepancy_status: DISCREPANCY_STATUS,
                requires_peer_parity: SCHEMA_RELEASE_REQUIRES_PEER_PARITY,
                automatic_authority_winner_allowed: AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
                required_convergence_participants: &REQUIRED_CONVERGENCE_PARTICIPANTS,
            }
        );
        assert!(schema_release_policy().requires_peer_parity);
        assert!(!schema_release_policy().automatic_authority_winner_allowed);
        assert_eq!(
            schema_release_policy().discrepancy_status,
            "STOPPED_FOR_EVALUATION"
        );
    }

    #[test]
    fn every_required_convergence_participant_is_unique() {
        let unique = REQUIRED_CONVERGENCE_PARTICIPANTS
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), REQUIRED_CONVERGENCE_PARTICIPANTS.len());
        assert!(unique.contains("typespec-sql-catalog"));
        assert!(unique.contains("json-schema-openapi-sql-catalog"));
        assert!(unique.contains("diesel-projection"));
        assert!(unique.contains("seaorm-projection"));
    }
}
