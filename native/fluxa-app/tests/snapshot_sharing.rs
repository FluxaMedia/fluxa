use fluxa_app::{FluxaRuntime, Snapshot};
use serde_json::{Value, json};
use std::sync::Arc;

fn effect_ids(effects: &[Value], kind: &str) -> Vec<String> {
    effects
        .iter()
        .filter(|effect| effect["type"] == kind)
        .map(|effect| effect["id"].as_str().unwrap().to_owned())
        .collect()
}

fn row(id: &str, cards: usize) -> Value {
    json!({
        "id": id,
        "name": id,
        "type": "movie",
        "items": (0..cards).map(|index| json!({"id": format!("{id}{index}"), "type": "movie"})).collect::<Vec<_>>(),
    })
}

fn loaded_home(rows: usize) -> (FluxaRuntime, Vec<String>) {
    let mut runtime = FluxaRuntime::new(json!({})).unwrap();
    let load = runtime
        .dispatch(json!({"type": "homeLoadRequested", "profile": {"id": "p1"}, "language": "en", "force": false}))
        .unwrap();
    let feeds: Vec<Value> = (0..rows)
        .map(|i| json!({"key": format!("row{i}")}))
        .collect();
    let bootstrap = effect_ids(&load.effects, "readHomeBootstrap");
    let started = runtime
        .complete_effect_result(
            &bootstrap[0],
            "ok",
            json!({"plan": {"feeds": feeds}}),
            Value::Null,
        )
        .unwrap();
    (runtime, effect_ids(&started.effects, "fetchHomeFeed"))
}

fn shared_rows(before: &Snapshot, after: &Snapshot) -> Vec<bool> {
    before
        .home_rows()
        .iter()
        .zip(after.home_rows())
        .map(|(a, b)| Arc::ptr_eq(a, b))
        .collect()
}

#[test]
fn requesting_a_page_leaves_every_row_shared() {
    let (mut runtime, feeds) = loaded_home(4);
    for (index, id) in feeds.iter().enumerate() {
        runtime
            .complete_effect_result(
                id,
                "ok",
                json!({"row": row(&format!("row{index}"), 5)}),
                Value::Null,
            )
            .unwrap();
    }
    let before = runtime.snapshot_shared();
    assert_eq!(before.home_rows().len(), 4);

    runtime
        .dispatch(json!({
            "type": "catalogPageRequested",
            "categoryId": "row2",
            "transportUrl": null,
            "contentType": "movie",
            "catalogId": "top",
            "skip": 5,
            "genre": null,
            "search": null,
            "remoteSource": null,
            "profile": null,
        }))
        .unwrap();
    let page = runtime.snapshot_shared();
    assert_eq!(shared_rows(&before, &page), [true, true, true, true]);
}

#[test]
fn a_catalog_page_replaces_one_row_and_keeps_the_others() {
    let (mut runtime, feeds) = loaded_home(4);
    for (index, id) in feeds.iter().enumerate() {
        runtime
            .complete_effect_result(
                id,
                "ok",
                json!({"row": row(&format!("row{index}"), 5)}),
                Value::Null,
            )
            .unwrap();
    }
    let request = runtime
        .dispatch(json!({
            "type": "catalogPageRequested",
            "categoryId": "row2",
            "transportUrl": null,
            "contentType": "movie",
            "catalogId": "top",
            "skip": 5,
            "genre": null,
            "search": null,
            "remoteSource": null,
            "profile": null,
        }))
        .unwrap();
    let before = runtime.snapshot_shared();
    let page_id = effect_ids(&request.effects, "fetchCatalogPage");
    runtime
        .complete_effect_result(
            &page_id[0],
            "ok",
            json!({"items": [{"id": "new1", "type": "movie"}, {"id": "new2", "type": "movie"}]}),
            Value::Null,
        )
        .unwrap();
    let after = runtime.snapshot_shared();
    assert_eq!(shared_rows(&before, &after), [true, true, false, true]);
    assert_eq!(after.home_rows()[2]["items"].as_array().unwrap().len(), 7);
    assert!(before.slice_shared(&after, "navigation"));
    assert!(before.slice_shared(&after, "library"));
}

#[test]
fn a_feed_completion_keeps_rows_that_did_not_change() {
    let (mut runtime, feeds) = loaded_home(5);
    for (index, id) in feeds.iter().take(3).enumerate() {
        runtime
            .complete_effect_result(
                id,
                "ok",
                json!({"row": row(&format!("row{index}"), 5)}),
                Value::Null,
            )
            .unwrap();
    }
    let before = runtime.snapshot_shared();
    runtime
        .complete_effect_result(&feeds[3], "ok", json!({"row": row("row3", 5)}), Value::Null)
        .unwrap();
    let after = runtime.snapshot_shared();
    assert_eq!(shared_rows(&before, &after), [true, true, true]);
    assert_eq!(after.home_rows().len(), 4);
    assert!(before.slice_shared(&after, "settings"));
}

#[test]
fn a_stale_completion_publishes_no_new_slices() {
    let (mut runtime, feeds) = loaded_home(2);
    runtime
        .complete_effect_result(&feeds[0], "ok", json!({"row": row("row0", 3)}), Value::Null)
        .unwrap();
    let before = runtime.snapshot_shared();
    let generations = runtime.generations();
    let update = runtime
        .complete_effect_result("no-such-effect", "ok", json!({}), Value::Null)
        .unwrap();
    assert!(update.effects.is_empty());
    let after = runtime.snapshot_shared();
    for name in before.names() {
        assert!(before.slice_shared(&after, name), "{name} was rebuilt");
    }
    assert_eq!(*generations, *runtime.generations());
}

#[test]
fn the_home_slice_reads_back_the_same_as_a_flat_snapshot() {
    let (mut runtime, feeds) = loaded_home(2);
    for (index, id) in feeds.iter().enumerate() {
        runtime
            .complete_effect_result(
                id,
                "ok",
                json!({"row": row(&format!("row{index}"), 2)}),
                Value::Null,
            )
            .unwrap();
    }
    let snapshot = runtime.snapshot_shared();
    let flat = snapshot.to_value();
    assert_eq!(
        snapshot.pointer("/home/categories"),
        flat.pointer("/home/categories")
    );
    assert_eq!(
        snapshot.pointer("/home/isLoading"),
        flat.pointer("/home/isLoading")
    );
    assert_eq!(snapshot["home"], flat["home"]);
    assert_eq!(
        snapshot.pointer("/home/categories/1/items/0/id"),
        Some(&json!("row10"))
    );
}
