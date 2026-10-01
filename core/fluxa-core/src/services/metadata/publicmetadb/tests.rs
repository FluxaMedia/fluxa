use super::*;

#[test]
fn builds_bearer_header() {
    assert_eq!(publicmetadb_bearer("pm-abc"), "Bearer pm-abc");
}







