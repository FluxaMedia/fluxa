use fluxa_skin::{FormFactor, load_skin, load_skins};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, file: &str, value: serde_json::Value) {
    let path = dir.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, value.to_string()).unwrap();
}

fn midnight(dir: &Path) {
    write(
        dir,
        "skin.json",
        json!({
            "schemaVersion": 1, "id": "midnight", "name": "Midnight", "version": "1.0.0",
            "screens": {"home": "screens/home.json"},
            "includes": ["includes.json"]
        }),
    );
    write(
        dir,
        "includes.json",
        json!({
            "card": {"type": "poster_card", "props": {"title": "{card.title}", "image": "{card.artworkUrl}"},
                     "action": {"name": "open_detail", "args": {"id": "{card.id}", "type": "{card.itemType}"}}}
        }),
    );
    write(
        dir,
        "screens/home.json",
        json!({
            "type": "column", "gap": 12, "children": [
                {"type": "text", "text": "{home.title}"},
                {"each": "home.cards", "as": "card", "template": {"include": "card"}}
            ]
        }),
    );
}

#[test]
fn skin_loads_and_resolves_against_data() {
    let dir = workspace("skin-loads");
    midnight(&dir);
    let skin = load_skin(&dir).unwrap();
    assert!(skin.replaces("home"));
    assert!(!skin.replaces("player"));
    let data = json!({"home": {"title": "Welcome", "cards": [
        {"id": "tt1", "itemType": "movie", "title": "One", "artworkUrl": "https://example.com/a.jpg"},
        {"id": "tt2", "itemType": "series", "title": "Two", "artworkUrl": null}
    ]}});
    let tree = skin
        .resolve("home", &data, FormFactor::Tv)
        .unwrap()
        .unwrap();
    let column = &tree[0];
    assert_eq!(column.children.len(), 3);
    let second = &column.children[2];
    assert_eq!(second.props["title"], json!("Two"));
    assert_eq!(
        second.action.as_ref().unwrap().args["type"],
        json!("series")
    );
    assert!(skin.resolve("player", &data, FormFactor::Tv).is_none());
}

#[test]
fn path_escapes_are_refused() {
    let dir = workspace("skin-escape");
    midnight(&dir);
    write(
        &dir,
        "skin.json",
        json!({
            "schemaVersion": 1, "id": "bad", "name": "Bad", "version": "1",
            "screens": {"home": "../secret.json"}
        }),
    );
    let problems = load_skin(&dir).unwrap_err();
    assert!(
        problems.iter().any(|p| p.contains("relative path")),
        "{problems:?}"
    );
}

#[test]
fn every_problem_in_a_skin_is_reported_together() {
    let dir = workspace("skin-problems");
    midnight(&dir);
    write(
        &dir,
        "screens/home.json",
        json!({
            "type": "column", "children": [
                {"type": "marquee"},
                {"type": "button", "action": {"name": "format_disk"}},
                {"include": "missing"}
            ]
        }),
    );
    let problems = load_skin(&dir).unwrap_err();
    assert_eq!(problems.len(), 3, "{problems:?}");
}

#[test]
fn unknown_screen_names_are_rejected() {
    let dir = workspace("skin-screen");
    midnight(&dir);
    write(
        &dir,
        "skin.json",
        json!({
            "schemaVersion": 1, "id": "x", "name": "X", "version": "1",
            "screens": {"lobby": "screens/home.json"}
        }),
    );
    assert!(load_skin(&dir).is_err());
}

#[test]
fn broken_skin_does_not_hide_the_good_ones() {
    let root = workspace("skin-many");
    midnight(&root.join("good"));
    write(&root.join("broken"), "skin.json", json!({"nonsense": true}));
    midnight(&root.join("twin"));
    let (skins, problems) = load_skins(&root);
    let ids: Vec<_> = skins.iter().map(|s| s.manifest.id.as_str()).collect();
    assert_eq!(ids, ["midnight"]);
    assert!(problems.iter().any(|p| p.starts_with("broken")));
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("twin") && p.contains("already used"))
    );
}
