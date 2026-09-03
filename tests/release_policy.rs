use std::{collections::BTreeSet, hint::black_box};

use declmig_lib_core::{
    schema_release_policy, SchemaReleasePolicy, AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    DISCREPANCY_STATUS, PEER_AUTHORITY_CERTIFICATION_FORMAT, REQUIRED_CONVERGENCE_PARTICIPANTS,
    SCHEMA_RELEASE_REQUIRES_PEER_PARITY, SCHEMA_REVISION,
};

const POLICY: SchemaReleasePolicy = schema_release_policy();

#[test]
fn public_release_policy_remains_fail_closed() {
    let policy = black_box(POLICY);

    assert_eq!(black_box(SCHEMA_REVISION), "declmig-0002");
    assert_eq!(
        policy.peer_authority_certification_format,
        black_box(PEER_AUTHORITY_CERTIFICATION_FORMAT)
    );
    assert_eq!(policy.discrepancy_status, black_box(DISCREPANCY_STATUS));
    assert_eq!(policy.discrepancy_status, "STOPPED_FOR_EVALUATION");
    assert!(policy.requires_peer_parity);
    assert!(black_box(SCHEMA_RELEASE_REQUIRES_PEER_PARITY));
    assert!(!policy.automatic_authority_winner_allowed);
    assert!(!black_box(AUTOMATIC_AUTHORITY_WINNER_ALLOWED));
    assert_eq!(
        policy.required_convergence_participants,
        black_box(&REQUIRED_CONVERGENCE_PARTICIPANTS)
    );
}

#[test]
fn public_release_policy_requires_six_unique_peer_projections() {
    let participants = black_box(POLICY)
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
