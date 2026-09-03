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
    assert_eq!(black_box(DISCREPANCY_STATUS), "STOPPED_FOR_EVALUATION");
    assert!(policy.requires_peer_parity);
    assert!(black_box(SCHEMA_RELEASE_REQUIRES_PEER_PARITY));
    assert!(!policy.automatic_authority_winner_allowed);
    assert!(!black_box(AUTOMATIC_AUTHORITY_WINNER_ALLOWED));
    assert_eq!(
        policy.required_convergence_participants,
        black_box(&REQUIRED_CONVERGENCE_PARTICIPANTS)
    );

    let unique = policy
        .required_convergence_participants
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 6);
}
