use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Contract {
    pub screens: Vec<String>,
    pub primitives: Vec<String>,
    pub components: BTreeMap<String, Vec<String>>,
    pub actions: BTreeMap<String, Vec<String>>,
    pub events: Vec<String>,
    pub permissions: Vec<String>,
}

pub fn contract() -> &'static Contract {
    static CONTRACT: OnceLock<Contract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        serde_json::from_str(include_str!("../../../shared/contracts/skin-contract.json"))
            .expect("skin-contract.json is valid")
    })
}

pub(crate) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub(crate) fn extension_action(name: &str) -> bool {
    let mut parts = name.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("ext"), Some(id), Some(action), None) if valid_id(id) && valid_id(action)
    )
}
