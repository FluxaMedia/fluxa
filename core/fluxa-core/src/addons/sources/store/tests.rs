use super::ownership::effective_shared_owner_id;







#[test]
fn effective_owner_id_falls_back_to_primary_only_when_flag_set() {
    let profiles = r#"[{"id":"p1"},{"id":"p2","usesPrimaryAddons":true},{"id":"p3"}]"#;
    assert_eq!(
        effective_shared_owner_id(profiles, "p1", "usesPrimaryAddons"),
        Some("p1".to_string())
    );
    assert_eq!(
        effective_shared_owner_id(profiles, "p2", "usesPrimaryAddons"),
        Some("p1".to_string())
    );
    assert_eq!(
        effective_shared_owner_id(profiles, "p3", "usesPrimaryAddons"),
        Some("p3".to_string())
    );
}




