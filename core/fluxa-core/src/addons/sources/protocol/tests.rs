use super::{
    build_resource_url, 
    normalize_addon_descriptor_json, parse_manifest, resolve_manifest_assets_json,
    supports_resource
};
use serde_json::{Value, json};

#[test]
fn resolve_manifest_assets_normalizes_transport_and_relative_assets() {
    let descriptor = r#"{
        "transportUrl":"addon.example/root/manifest.json",
        "manifest":{
            "id":"addon",
            "name":"Addon",
            "description":"",
            "resources":[],
            "types":[],
            "catalogs":[],
            "logo":"logo.png",
            "background":"/bg.jpg"
        }
    }"#;
    let resolved = resolve_manifest_assets_json(descriptor)
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("resolved descriptor");

    assert_eq!(
        resolved.get("transportUrl").and_then(Value::as_str),
        Some("https://addon.example/root/manifest.json")
    );
    assert_eq!(
        resolved
            .get("manifest")
            .and_then(|manifest| manifest.get("description")),
        Some(&Value::Null)
    );
    assert_eq!(
        resolved
            .get("manifest")
            .and_then(|manifest| manifest.get("logo"))
            .and_then(Value::as_str),
        Some("https://addon.example/root/logo.png")
    );
    assert_eq!(
        resolved
            .get("manifest")
            .and_then(|manifest| manifest.get("background"))
            .and_then(Value::as_str),
        Some("https://addon.example/bg.jpg")
    );
}


#[test]
fn manifest_fetch_plan_owns_cache_key_and_candidates() {
    let plan = super::manifest_fetch_plan_json("127.0.0.1:7000")
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("manifest fetch plan");

    assert_eq!(
        plan.get("normalizedTransportUrl").and_then(Value::as_str),
        Some("http://127.0.0.1:7000/manifest.json")
    );
    assert_eq!(
        plan.get("cacheKey").and_then(Value::as_str),
        Some("manifest_v10_http://127.0.0.1:7000/manifest.json")
    );
    assert_eq!(
        plan.get("candidateUrls")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str),
        Some("http://127.0.0.1:7000/manifest.json")
    );
}


#[test]
fn parse_manifest_preserves_stremio_manifest_fields() {
    let parsed = parse_manifest(
        r#"{
            "id":"org.example.full",
            "version":"1.2.3",
            "name":"Full",
            "description":"All fields",
            "resources":["catalog",{"name":"stream","types":["movie"],"idPrefixes":["tt"]}],
            "types":["movie","series"],
            "idPrefixes":["tt"],
            "catalogs":[{"type":"movie","id":"top","name":"Top","extra":[{"name":"genre","isRequired":false,"options":["Drama"],"optionsLimit":2,"default":"Drama"}],"extraSupported":["search"],"extraRequired":["genre"]}],
            "addonCatalogs":[{"type":"addon","id":"community","name":"Community"}],
            "config":[{"key":"token","type":"password","default":"x","title":"Token","required":true}],
            "background":"/bg.jpg",
            "logo":"logo.png",
            "contactEmail":"ops@example.com",
            "behaviorHints":{"adult":true,"p2p":true,"configurable":true,"configurationRequired":true}
        }"#,
        "https://addon.example/root/manifest.json",
        "Unknown",
    )
    .and_then(|json| serde_json::from_str::<Value>(&json).ok())
    .expect("parsed manifest");
    let manifest = parsed.get("manifest").expect("manifest");

    assert_eq!(
        manifest["addonCatalogs"][0]["id"].as_str(),
        Some("community")
    );
    assert_eq!(manifest["config"][0]["key"].as_str(), Some("token"));
    assert_eq!(manifest["contactEmail"].as_str(), Some("ops@example.com"));
    assert_eq!(
        manifest["behaviorHints"]["configurationRequired"].as_bool(),
        Some(true)
    );
    assert_eq!(
        manifest["catalogs"][0]["extraRequired"][0].as_str(),
        Some("genre")
    );
    assert_eq!(
        manifest["catalogs"][0]["extra"][0]["default"].as_str(),
        Some("Drama")
    );
    assert_eq!(
        manifest["logo"].as_str(),
        Some("https://addon.example/root/logo.png")
    );
    assert_eq!(
        manifest["background"].as_str(),
        Some("https://addon.example/bg.jpg")
    );
    assert_eq!(manifest["supportsCatalog"].as_bool(), Some(true));
    assert_eq!(
        manifest["catalogs"][0]["supportsInitialLoad"].as_bool(),
        Some(true)
    );
    assert_eq!(
        manifest["catalogs"][0]["supportsSearch"].as_bool(),
        Some(true)
    );
    assert_eq!(
        manifest["catalogs"][0]["hasRequiredExtraExceptGenre"].as_bool(),
        Some(false)
    );
}


#[test]
fn build_resource_url_appends_extra_path_segment_and_omits_blank_values() {
    assert_eq!(
        build_resource_url(
            "https://addon.example/manifest.json",
            "stream",
            "movie",
            "tt123",
            None
        ),
        "https://addon.example/stream/movie/tt123.json"
    );
    assert_eq!(
        build_resource_url(
            "https://addon.example/manifest.json",
            "catalog",
            "movie",
            "top",
            Some(r#"{"genre":"action"}"#),
        ),
        "https://addon.example/catalog/movie/top/genre=action.json"
    );
    // A blank extra value contributes nothing — same shape as no extra at all.
    assert_eq!(
        build_resource_url(
            "https://addon.example/manifest.json",
            "stream",
            "movie",
            "tt123",
            Some(r#"{"genre":""}"#),
        ),
        "https://addon.example/stream/movie/tt123.json"
    );
}

#[test]
fn supports_resource_gates_on_content_type_but_catalog_bypasses_id_prefix() {
    let manifest = json!({
        "resources": ["stream"],
        "types": ["movie"],
        "idPrefixes": ["tt"]
    });
    assert!(supports_resource(
        &manifest.to_string(),
        "streams",
        Some("movie"),
        Some("tt123")
    ));
    // Wrong content type for this manifest's declared types.
    assert!(!supports_resource(
        &manifest.to_string(),
        "stream",
        Some("series"),
        Some("tt123")
    ));

    let catalog_manifest = json!({
        "resources": [{ "name": "catalog", "types": ["movie"], "idPrefixes": ["tt"] }]
    });
    // catalog resources never gate on id prefix, even though one is declared.
    assert!(supports_resource(
        &catalog_manifest.to_string(),
        "catalog",
        Some("movie"),
        Some("does-not-match-prefix")
    ));

    assert!(!supports_resource("not json", "stream", None, None));
}

#[test]
fn legacy_descriptor_normalization_reuses_manifest_parser() {
    let normalized: Value = serde_json::from_str(
        &normalize_addon_descriptor_json(
            r#"{"id":"legacy","name":"Legacy","resources":["stream"],"customField":"kept"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(normalized["transportUrl"], "legacy");
    assert_eq!(normalized["manifest"]["customField"], "kept");
}
