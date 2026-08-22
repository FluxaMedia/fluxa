use super::super::*;
use serde_json::Value;

#[test]
fn wire_fixtures_match_golden_dispatch_output() {
    let actions_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/wire/actions");
    let expected_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/wire/expected");
    let update = std::env::var("UPDATE_WIRE_FIXTURES").is_ok();

    let mut entries: Vec<_> = std::fs::read_dir(&actions_dir)
        .expect("tests/wire/actions must exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no wire fixtures found");

    for path in entries {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let fixture: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let initial_state = serde_json::to_string(&fixture["initialState"]).unwrap();
        let action_json = serde_json::to_string(&fixture["action"]).unwrap();

        let handle = create_headless_engine(&initial_state);
        let actual: Value = serde_json::from_str(
            &headless_engine_dispatch_json(handle, &action_json)
                .unwrap_or_else(|| panic!("dispatch failed for fixture {name}")),
        )
        .unwrap();
        destroy_headless_engine(handle);

        let expected_path = expected_dir.join(format!("{name}.json"));
        if update {
            std::fs::create_dir_all(&expected_dir).unwrap();
            std::fs::write(
                &expected_path,
                serde_json::to_string_pretty(&actual).unwrap() + "\n",
            )
            .unwrap();
            continue;
        }

        let expected: Value = serde_json::from_str(&std::fs::read_to_string(&expected_path)
                .unwrap_or_else(|_| {
                    panic!("missing golden fixture {expected_path:?}; run with UPDATE_WIRE_FIXTURES=1 to generate")
                }))
            .unwrap();

        assert_eq!(actual, expected, "wire fixture drift in {name}");
    }
}
