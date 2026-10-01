use super::*;

fn parse(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}















// Renaming or removing a routed method must show up as a diff in this fixture.
