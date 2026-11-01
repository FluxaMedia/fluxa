use crate::stream_pipeline::metadatakit_enabled;
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
    catalogs: Vec<Value>,
) -> Value {
    let url = transport(host, profile_id);
    let mut resources = vec![json!({
        "name": resource,
        "types": ["movie", "series", "anime"],
        "idPrefixes": ID_PREFIXES
    })];
    if !catalogs.is_empty() {
        resources.insert(0, json!("catalog"));
    }
    json!({
        "transportUrl": url,
        "manifest": {
            "id": id,
            "version": "1.0.0",
            "name": name,
            "description": description,
            "transportUrl": url,
            "types": ["movie", "series", "anime"],
            "resources": resources,
            "idPrefixes": ID_PREFIXES,
            "catalogs": catalogs,
        }
    })
}

fn metadatakit_catalogs(prefs: &Value) -> Vec<Value> {
    fluxa_aggmeta::tmdb_catalog::from_prefs(prefs)
        .iter()
        .map(|def| {
            json!({
                "type": def.kind.as_str(),
                "id": def.id,
                "name": def.name,
                "extra": [{"name": "skip"}]
            })
        })
        .collect()
}

pub fn is_kit(addon: &Value) -> bool {
    addon["transportUrl"]
        .as_str()
        .is_some_and(|url| url.contains(STREAMKIT_HOST) || url.contains(METADATAKIT_HOST))
}

pub fn addons(prefs: &Value, profile_id: &str) -> Vec<Value> {
    let mut list = Vec::new();
    if metadatakit_enabled(prefs) {
        list.push(descriptor(
            METADATAKIT_HOST,
            profile_id,
            "fluxa.metadatakit",
            "Metadatakit",
            "Merged metadata from TMDB with anime id mapping",
            "meta",
            metadatakit_catalogs(prefs),
        ));
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kit_is_absent_by_default() {
        assert!(addons(&json!({}), "p1").is_empty());
        assert!(addons(&Value::Null, "p1").is_empty());
    }

    #[test]
    fn metadatakit_appears_only_when_enabled() {
        assert!(addons(&json!({"streamkitEnabled": true}), "p1").is_empty());
        let list = addons(&json!({"metadatakitEnabled": true}), "p1");
        assert_eq!(
            list[0]["manifest"]["catalogs"].as_array().map(Vec::len),
            Some(6)
        );
        assert_eq!(list[0]["manifest"]["resources"][0], "catalog");
    }

    #[test]
    fn catalog_switches_change_the_manifest() {
        let prefs =
            json!({"metadatakitEnabled": true, "catalogTrending": false, "catalogNetflix": true});
        let list = addons(&prefs, "p1");
        let catalogs = list[0]["manifest"]["catalogs"].as_array().unwrap();
        assert_eq!(catalogs.len(), 6);
        assert!(catalogs.iter().any(|c| c["id"] == "svc.netflix"));
        assert!(catalogs.iter().all(|c| c["id"] != "tmdb.trending"));
    }
}
