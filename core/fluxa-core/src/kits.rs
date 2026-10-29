use crate::stream_pipeline::{enabled, metadatakit_enabled};
use serde_json::{Value, json};

pub const STREAMKIT_HOST: &str = "streamkit.fluxa.internal";
pub const METADATAKIT_HOST: &str = "metadatakit.fluxa.internal";

const ID_PREFIXES: [&str; 7] = [
    "tt", "tmdb:", "tvdb:", "kitsu:", "mal:", "anilist:", "anidb:",
];

fn transport(host: &str, profile_id: &str) -> String {
    format!("https://{host}/{profile_id}/manifest.json")
}

fn descriptor(
    host: &str,
    profile_id: &str,
    id: &str,
    name: &str,
    description: &str,
    resource: &str,
) -> Value {
    let url = transport(host, profile_id);
    json!({
        "transportUrl": url,
        "manifest": {
            "id": id,
            "version": "1.0.0",
            "name": name,
            "description": description,
            "transportUrl": url,
            "types": ["movie", "series", "anime"],
            "resources": [
                {"name": resource, "types": ["movie", "series", "anime"], "idPrefixes": ID_PREFIXES},
            ],
            "idPrefixes": ID_PREFIXES,
            "catalogs": [],
        }
    })
}

pub fn is_kit(addon: &Value) -> bool {
    addon["transportUrl"]
        .as_str()
        .is_some_and(|url| url.contains(STREAMKIT_HOST) || url.contains(METADATAKIT_HOST))
}

pub fn addons(prefs: &Value, profile_id: &str) -> Vec<Value> {
    let mut list = Vec::new();
    if enabled(prefs) {
        list.push(descriptor(
            STREAMKIT_HOST,
            profile_id,
            "fluxa.streamkit",
            "Streamkit",
            "Filters, sorts and formats the streams of your other add-ons",
            "stream",
        ));
    }
    if metadatakit_enabled(prefs) {
        list.push(descriptor(
            METADATAKIT_HOST,
            profile_id,
            "fluxa.metadatakit",
            "Metadatakit",
            "Merged metadata from TMDB with anime id mapping",
            "meta",
        ));
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_kits_are_absent_by_default() {
        assert!(addons(&json!({}), "p1").is_empty());
        assert!(addons(&Value::Null, "p1").is_empty());
    }

    #[test]
    fn each_kit_appears_only_when_enabled() {
        let list = addons(&json!({"streamkitEnabled": true}), "p1");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["manifest"]["id"], "fluxa.streamkit");
        assert_eq!(list[0]["manifest"]["resources"][0]["name"], "stream");
        let list = addons(&json!({"metadatakitEnabled": true}), "p1");
        assert_eq!(list[0]["manifest"]["resources"][0]["name"], "meta");
    }
}
