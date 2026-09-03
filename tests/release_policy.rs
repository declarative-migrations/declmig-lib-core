use declmig_lib_core::{
    schema_release_policy, SchemaReleasePolicy, AUTOMATIC_AUTHORITY_WINNER_ALLOWED,
    PEER_AUTHORITY_CERTIFICATION_FORMAT, SCHEMA_RELEASE_REQUIRES_PEER_PARITY, SCHEMA_REVISION,
};

const POLICY: SchemaReleasePolicy = schema_release_policy();

#[test]
fn public_release_policy_remains_fail_closed() {
    assert_eq!(SCHEMA_REVISION, "declmig-0002");
    assert_eq!(
        POLICY.peer_authority_certification_format,
        PEER_AUTHORITY_CERTIFICATION_FORMAT
    );
    assert!(POLICY.requires_peer_parity);
    assert!(SCHEMA_RELEASE_REQUIRES_PEER_PARITY);
    assert!(!POLICY.automatic_authority_winner_allowed);
    assert!(!AUTOMATIC_AUTHORITY_WINNER_ALLOWED);
}
