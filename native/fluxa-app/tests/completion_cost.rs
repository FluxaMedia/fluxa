use fluxa_app::FluxaRuntime;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const ROWS: usize = 20;
const CARDS: usize = 100;

fn card(row: usize, index: usize) -> Value {
    json!({
        "id": format!("tt{row:02}{index:04}"),
        "type": "movie",
        "name": format!("Title {row} {index}"),
        "poster": format!("https://img.example.test/poster/{row}/{index}.jpg"),
        "background": format!("https://img.example.test/bg/{row}/{index}.jpg"),
        "logo": format!("https://img.example.test/logo/{row}/{index}.png"),
        "description": "A long enough description to look like real catalog metadata, with a few sentences of plot summary.",
        "releaseInfo": "2024",
        "imdbRating": "7.4",
        "genres": ["Drama", "Thriller", "Mystery"],
        "runtime": "118 min",
        "cast": ["Actor One", "Actor Two", "Actor Three", "Actor Four"],
        "director": ["Director One"],
        "trailers": [{"source": "abc123", "type": "Trailer"}],
        "links": [{"name": "Drama", "category": "Genres", "url": "stremio:///discover"}],
    })
}

fn row(row: usize) -> Value {
    json!({
        "id": format!("row{row}"),
        "name": format!("Row {row}"),
        "type": "movie",
        "items": (0..CARDS).map(|index| card(row, index)).collect::<Vec<_>>(),
    })
}

fn effect_ids(effects: &[Value], kind: &str) -> Vec<String> {
    effects
        .iter()
        .filter(|effect| effect["type"] == kind)
        .map(|effect| effect["id"].as_str().unwrap().to_owned())
        .collect()
}

fn percentile(samples: &mut [Duration], p: f64) -> Duration {
    samples.sort();
    samples[((samples.len() - 1) as f64 * p) as usize]
}

#[test]
#[ignore = "run with: cargo test --release -p fluxa-app --test completion_cost -- --ignored --nocapture"]
fn catalog_completion_cost() {
    let mut runtime = FluxaRuntime::new(json!({})).expect("runtime");
    let load = runtime
        .dispatch(json!({"type": "homeLoadRequested", "profile": {"id": "p1"}, "language": "en", "force": false}))
        .expect("load");
    let bootstrap = effect_ids(&load.effects, "readHomeBootstrap");
    let feeds: Vec<Value> = (0..ROWS).map(|i| json!({"key": format!("row{i}")})).collect();
    let started = runtime
        .complete_effect_result(&bootstrap[0], "ok", json!({"plan": {"feeds": feeds}}), Value::Null)
        .expect("bootstrap");
    let feed_ids = effect_ids(&started.effects, "fetchHomeFeed");
    assert_eq!(feed_ids.len(), ROWS);

    let mut samples = Vec::new();
    let burst = Instant::now();
    for (index, id) in feed_ids.iter().enumerate() {
        let held = runtime.snapshot_shared();
        let value = json!({"row": row(index)});
        let started = Instant::now();
        runtime
            .complete_effect_result(id, "ok", value, Value::Null)
            .expect("feed");
        samples.push(started.elapsed());
        drop(held);
    }
    let burst = burst.elapsed();
    let mut ordered = samples.clone();
    println!(
        "feed completions: {ROWS} rows x {CARDS} cards, burst {:?}, median {:?}, p90 {:?}, last {:?}",
        burst,
        percentile(&mut ordered, 0.5),
        percentile(&mut ordered, 0.9),
        samples.last().unwrap()
    );

    let page = runtime
        .dispatch(json!({
            "type": "catalogPageRequested",
            "categoryId": "row3",
            "transportUrl": null,
            "contentType": "movie",
            "catalogId": "top",
            "skip": 100,
            "genre": null,
            "search": null,
            "remoteSource": null,
            "profile": null,
        }))
        .expect("page request");
    let page_id = effect_ids(&page.effects, "fetchCatalogPage");
    let held = runtime.snapshot_shared();
    let items: Vec<Value> = (CARDS..CARDS * 2).map(|index| card(3, index)).collect();
    let started = Instant::now();
    runtime
        .complete_effect_result(&page_id[0], "ok", json!({"items": items}), Value::Null)
        .expect("page");
    println!("catalog page completion (+{CARDS} cards): {:?}", started.elapsed());
    drop(held);
}
