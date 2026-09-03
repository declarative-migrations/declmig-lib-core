use std::collections::BTreeSet;

use declmig_lib_core::{
    schema_release_policy, SchemaReleasePolicy, AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    DISCREPANCY_STATUS, PEER_AUTHORITY_CERTIFICATION_FORMAT, REQUIRED_CONVERGENCE_PARTICIPANTS,
    SCHEMA_RELEASE_REQUIRES_PEER_PARITY, SCHEMA_REVISION,
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
    assert_eq!(DISCREPANCY_STATUS, "STOPPED_FOR_EVALUATION");
    assert!(POLICY.requires_peer_parity);
    assert!(SCHEMA_RELEASE_REQUIRES_PEER_PARITY);
    assert!(!POLICY.automatic_authority_winner_allowed);
    assert!(!AUTOMATIC_AUTHORITY_WINNER_ALLOWED);
    assert_eq!(
        POLICY.required_convergence_participants,
        &REQUIRED_CONVERGENCE_PARTICIPANTS
    );

    let unique = POLICY
        .required_convergence_participants
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 6);
}
