pub(crate) const SETTINGS_MANIFEST_JSON: &str =
    include_str!("../../settings/settings-manifest.json");

#[cfg(test)]
mod tests {
    use super::SETTINGS_MANIFEST_JSON;
    use serde_json::Value;
    use std::collections::HashSet;

    #[test]
    fn manifest_contains_unique_camel_case_keys() {
        let manifest: Value =
            serde_json::from_str(SETTINGS_MANIFEST_JSON).expect("valid settings manifest");
        let settings = manifest
            .get("settings")
            .and_then(Value::as_array)
            .expect("settings array");
        let mut keys = HashSet::new();
        for setting in settings {
            let key = setting
                .get("key")
                .and_then(Value::as_str)
                .expect("setting key");
            assert!(
                key.chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_alphabetic())
            );
            assert!(
                key.chars()
                    .all(|character| character.is_ascii_alphanumeric())
            );
            assert!(keys.insert(key), "duplicate setting key: {key}");
        }
    }
}
