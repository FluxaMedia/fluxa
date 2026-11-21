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
            "card": {"type": "row", "children": [{"type": "text", "text": "{card.title}"}],
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
        .resolve("home", &data, &mut Default::default())
        .unwrap()
        .unwrap();
    let column = &tree[0];
    assert_eq!(column.children.len(), 3);
    let second = &column.children[2];
    assert_eq!(second.children[0].text.as_deref(), Some("Two"));
    assert_eq!(
        second.action.as_ref().unwrap().args["type"],
        json!("series")
    );
    assert!(
        skin.resolve("player", &data, &mut Default::default())
            .is_none()
    );
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
                {"type": "row", "action": {"name": "format_disk"}},
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

#[test]
fn constants_canvas_and_manifest_name_are_read() {
    let dir = workspace("skin-constants");
    write(
        &dir,
        "manifest.json",
        json!({
            "schemaVersion": 1, "id": "scaled", "name": "Scaled", "version": "1.0.0",
            "forms": ["tv"], "canvas": {"tv": 960}, "constants": "constants.json",
            "screens": {"home": "screens/home.json"}
        }),
    );
    write(
        &dir,
        "constants.json",
        json!({"boxWidth": 300, "boxGap": 12}),
    );
    write(
        &dir,
        "screens/home.json",
        json!({"type": "column", "gap": "{const.boxGap}", "padding": "{const.boxGap}",
               "width": "{const.boxWidth}", "children": []}),
    );
    let skin = load_skin(&dir).unwrap();
    assert_eq!(skin.canvas(FormFactor::Tv), Some(960.0));
    assert_eq!(skin.canvas(FormFactor::Desktop), None);
    assert!(skin.supports(FormFactor::Tv));
    assert!(!skin.supports(FormFactor::Mobile));
    let data = json!({"const": skin.constants()});
    let tree = skin
        .resolve("home", &data, &mut Default::default())
        .unwrap()
        .unwrap();
    assert_eq!(tree[0].width, Some(300.0));
    assert_eq!(tree[0].gap, Some(12.0));
    assert_eq!(tree[0].padding, Some([12.0; 4]));
}

#[test]
fn unknown_canvas_name_is_rejected() {
    let dir = workspace("skin-bad-canvas");
    write(
        &dir,
        "manifest.json",
        json!({"schemaVersion": 1, "id": "bad", "name": "Bad", "version": "1.0.0",
               "canvas": {"watch": 300}}),
    );
    assert!(load_skin(&dir).is_err());
}

#[test]
fn token_document_is_carried_with_the_skin() {
    let dir = workspace("skin-tokens");
    write(
        &dir,
        "manifest.json",
        json!({"schemaVersion": 1, "id": "owned", "name": "Owned", "version": "1.0.0",
               "tokens": "tokens.json"}),
    );
    write(
        &dir,
        "tokens.json",
        json!({"colors": {"background": "#000000"}}),
    );
    let skin = load_skin(&dir).unwrap();
    assert!(
        skin.tokens()
            .is_some_and(|source| source.contains("#000000"))
    );
    write(&dir, "tokens.json", json!([]));
    fs::write(dir.join("tokens.json"), "not json").unwrap();
    assert!(load_skin(&dir).is_err());
}

#[test]
fn zip_with_a_top_level_folder_loads() {
    use std::io::Write;
    let dir = workspace("skin-zip");
    let path = dir.join("midnight.zip");
    let mut archive = zip::ZipWriter::new(fs::File::create(&path).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    let manifest = json!({"schemaVersion": 1, "id": "zipped", "name": "Zipped", "version": "1.0.0",
        "screens": {"home": "screens/home.json"}});
    archive
        .start_file("midnight-main/manifest.json", options)
        .unwrap();
    archive.write_all(manifest.to_string().as_bytes()).unwrap();
    archive
        .start_file("midnight-main/screens/home.json", options)
        .unwrap();
    archive
        .write_all(
            json!({"type": "column", "children": []})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    archive.finish().unwrap();
    let (skins, problems) = load_skins(&dir);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(skins.len(), 1);
    assert!(skins[0].replaces("home"));
}

#[test]
fn zip_without_manifest_is_rejected() {
    use std::io::Write;
    let dir = workspace("skin-zip-empty");
    let path = dir.join("nothing.zip");
    let mut archive = zip::ZipWriter::new(fs::File::create(&path).unwrap());
    archive
        .start_file("readme.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"hi").unwrap();
    archive.finish().unwrap();
    assert!(fluxa_skin::load_zip(&path).is_err());
}

#[test]
fn installing_a_folder_makes_it_loadable() {
    let source = workspace("skin-install-source");
    midnight(&source);
    let skins = workspace("skin-install-target");
    let id = fluxa_skin::install(&source, &skins).unwrap();
    assert_eq!(id, "midnight");
    let (found, problems) = load_skins(&skins);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(found.len(), 1);
}

#[test]
fn skin_carries_its_own_language_tables() {
    let dir = workspace("skin-languages");
    write(
        &dir,
        "manifest.json",
        json!({"schemaVersion": 1, "id": "words", "name": "Words", "version": "1.0.0",
               "languages": {"en": "languages/en.json", "tr": "languages/tr.json"}}),
    );
    write(
        &dir,
        "languages/en.json",
        json!({"hello": "Hello", "count": 3}),
    );
    write(&dir, "languages/tr.json", json!({"hello": "Merhaba"}));
    let skin = load_skin(&dir).unwrap();
    assert_eq!(skin.strings()["tr"]["hello"], "Merhaba");
    assert_eq!(skin.strings()["en"].len(), 1);
}
