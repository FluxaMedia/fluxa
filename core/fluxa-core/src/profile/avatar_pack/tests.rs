use super::*;

#[test]
fn repository_plan_accepts_github_urls_only() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_repository_plan_json(
            r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars.git"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(output["owner"], "eueueue292");
    assert_eq!(output["repository"], "Fusion-Profile-Avatars");
    assert!(
        profile_avatar_pack_repository_plan_json(
            r#"{"repositoryUrl":"https://github.com.evil/a/b"}"#
        )
        .is_none()
    );
}

#[test]
fn catalog_discovers_nested_packs_and_builds_raw_urls() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_catalog_json(
            r#"{
                "repositoryUrl":"eueueue292/Fusion-Profile-Avatars",
                "reference":"main",
                "tree":{"tree":[
                    {"path":"Attack On Titan/pack.json","type":"blob"},
                    {"path":"Disney+/Marvel/json.pack","type":"blob"},
                    {"path":"README.md","type":"blob"}
                ]}
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(output["categories"].as_array().unwrap().len(), 2);
    assert_eq!(output["categories"][0]["path"], "Attack On Titan");
    assert_eq!(output["categories"][1]["name"], "Marvel");
    assert_eq!(
        output["categories"][1]["manifestUrl"],
        "https://raw.githubusercontent.com/eueueue292/Fusion-Profile-Avatars/main/Disney%2B/Marvel/json.pack"
    );
}

#[test]
fn repository_plan_accepts_a_url_pointing_at_one_pack() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_repository_plan_json(
            r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Solo%20Leveling%20S2/pack.json"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(output["owner"], "eueueue292");
    assert_eq!(output["repository"], "Fusion-Profile-Avatars");
}

#[test]
fn manifest_plan_rewrites_a_github_blob_url_to_raw() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_manifest_plan_json(
            r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Solo%20Leveling%20S2/pack.json"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        output["manifestUrl"],
        "https://raw.githubusercontent.com/eueueue292/Fusion-Profile-Avatars/main/Solo%20Leveling%20S2/pack.json"
    );
}

#[test]
fn manifest_plan_accepts_any_https_host_serving_a_manifest() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_manifest_plan_json(
            r#"{"repositoryUrl":"https://example.com/packs/solo-leveling/pack.json"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        output["manifestUrl"],
        "https://example.com/packs/solo-leveling/pack.json"
    );
}

#[test]
fn manifest_plan_rejects_urls_not_pointing_at_a_manifest_file() {
    assert!(
        profile_avatar_pack_manifest_plan_json(
            r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars"}"#
        )
        .is_none()
    );
    assert!(
        profile_avatar_pack_manifest_plan_json(
            r#"{"repositoryUrl":"https://example.com/packs/solo-leveling/avatar.png"}"#
        )
        .is_none()
    );
}

#[test]
fn catalog_scopes_to_the_pack_a_blob_url_points_at() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_catalog_json(
            r#"{
                "repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Solo%20Leveling%20S2/pack.json",
                "reference":"main",
                "tree":{"tree":[
                    {"path":"Solo Leveling S2/pack.json","type":"blob"},
                    {"path":"Attack On Titan/pack.json","type":"blob"}
                ]}
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    let categories = output["categories"].as_array().unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0]["path"], "Solo Leveling S2");
}

#[test]
fn catalog_discovers_root_level_pack_and_names_it_after_the_repository() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_catalog_json(
            r#"{
                "repositoryUrl":"you/your-repo",
                "reference":"main",
                "tree":{"tree":[
                    {"path":"pack.json","type":"blob"},
                    {"path":"images/luffy.png","type":"blob"}
                ]}
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(output["categories"].as_array().unwrap().len(), 1);
    assert_eq!(output["categories"][0]["name"], "your-repo");
    assert_eq!(output["categories"][0]["path"], "");
    assert_eq!(
        output["categories"][0]["manifestUrl"],
        "https://raw.githubusercontent.com/you/your-repo/main/pack.json"
    );
}

#[test]
fn pack_parser_accepts_bare_array_without_a_title() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_json(
            r#"{
                "manifestUrl":"https://raw.githubusercontent.com/you/your-repo/main/pack.json",
                "pack":[
                    {"name":"Luffy","url":"https://images.example/luffy.png"},
                    {"name":"Zoro","url":"https://images.example/zoro.png"}
                ]
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(output["title"], "your-repo");
    assert_eq!(output["avatars"].as_array().unwrap().len(), 2);
    assert_eq!(output["avatars"][0]["name"], "Luffy");
}

#[test]
fn pack_parser_keeps_only_unique_https_avatars() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_json(
            r#"{
                "manifestUrl":"https://example.com/pack.json",
                "pack":{"title":"Test pack","images":[
                    {"name":"A","url":"https://images.example/a.png"},
                    {"name":"Duplicate","url":"https://images.example/a.png"},
                    {"name":"Unsafe","url":"file:///tmp/avatar.png"}
                ]}
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(output["avatars"].as_array().unwrap().len(), 1);
    assert_eq!(output["avatars"][0]["name"], "A");
}

#[test]
fn pack_parser_rewrites_github_blob_urls_to_raw() {
    let output: Value = serde_json::from_str(
        &profile_avatar_pack_json(
            r#"{
                "manifestUrl":"https://example.com/pack.json",
                "pack":{"title":"Hell's Paradise","images":[
                    {"name":"Choubei","url":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Hells%20Paradise/Choubei.PNG?raw=true&v=3"}
                ]}
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        output["avatars"][0]["url"],
        "https://raw.githubusercontent.com/eueueue292/Fusion-Profile-Avatars/main/Hells%20Paradise/Choubei.PNG"
    );
}
