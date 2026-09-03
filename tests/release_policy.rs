use std::collections::BTreeSet;

use declmig_lib_core::{
    schema_release_policy, SchemaReleasePolicy, AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    DISCREPANCY_STATUS, PEER_AUTHORITY_CERTIFICATION_FORMAT,
    REQUIRED_CONVERGENCE_PARTICIPANTS, SCHEMA_RELEASE_REQUIRES_PEER_PARITY, SCHEMA_REVISION,
};

const POLICY: SchemaReleasePolicy = schema_release_policy();

#[test]
fn public_release_policy_remains_fail_closed() {
    assert_eq!(SCHEMA_REVISION, "declmig-0002");
    assert_eq!(
        POLICY.peer_authority_certification_format,
        PEER_AUTHORITY_CERTIFICATION_FORMAT
    );
    assert_eq!(POLICY.discrepancy_status, DISCREPANCY_STATUS);
    assert_eq!(POLICY.discrepancy_status, "STOPPED_FOR_EVALUATION");
    assert!(POLICY.requires_peer_parity);
    assert!(SCHEMA_RELEASE_REQUIRES_PEER_PARITY);
    assert!(!POLICY.automatic_authority_winner_allowed);
    assert!(!AUTOMATIC_AUTHORITY_WINNER_ALLOWED);
    assert_eq!(
        POLICY.required_convergence_participants,
        &REQUIRED_CONVERGENCE_PARTICIPANTS
    );
}

#[test]
fn public_release_policy_requires_six_unique_peer_projections() {
    let participants = POLICY
        .required_convergence_participants
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    assert_eq!(participants.len(), 6);
    assert!(participants.contains("typespec-sql-catalog"));
    assert!(participants.contains("json-schema-openapi-sql-catalog"));
    assert!(participants.contains("reviewed-dpm-desired-catalog"));
    assert!(participants.contains("diesel-projection"));
    assert!(participants.contains("seaorm-projection"));
    assert!(participants.contains("shadow-live-catalog-readback"));
}
