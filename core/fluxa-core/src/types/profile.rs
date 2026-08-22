use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// The profile object crosses the wire in full on every ProfileActivated/AuthRefresh
// action and gets echoed back unchanged in several effect payloads, so `extra`
// preserves every field this crate doesn't otherwise need to read (settings,
// addon lists, auth tokens, ...) instead of only round-tripping the fields listed
// here.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Profile {
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_unknown_fields_through_extra() {
        let raw = serde_json::json!({
            "id": "p1",
            "settings": {"language": "en"},
            "authTokens": {"trakt": "abc"}
        });
        let profile: Profile = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(profile.id.as_deref(), Some("p1"));
        assert_eq!(profile.to_value(), raw);
    }

    #[test]
    fn missing_id_defaults_to_none() {
        let profile: Profile = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(profile.id, None);
    }
}
