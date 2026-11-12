use super::*;
use crate::Snapshot;

pub(crate) fn take_native_actions(state: &mut RendererState) -> String {
    if state.pending_native_actions.is_empty() {
        return "[]".to_owned();
    }
    serde_json::to_string(&std::mem::take(&mut state.pending_native_actions))
        .unwrap_or_else(|_| "[]".to_owned())
}

pub(crate) fn profile_language(profile: &Value) -> String {
    profile
        .get("language")
        .and_then(Value::as_str)
        .filter(|language| !language.is_empty())
        .unwrap_or("en")
        .to_owned()
}

pub(crate) fn pull_session_snapshot(state: &mut RendererState, timer: &mut FrameTimer) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let view = session.snapshot_view();
    if state.session_revision == Some(view.revision) {
        return;
    }
    state.session_revision = Some(view.revision);
    fluxa_effects::flow(|| format!("host pulled revision {}", view.revision));
    let fixture = crate::preview::fixture_snapshot(&view.snapshot);
    timer.mark("snap-pull");
    let live_generations = fixture.is_none().then(|| view.generations.clone());
    let (snapshot, unchanged) = match fixture {
        Some(fixture) => {
            let unchanged = state
                .core_snapshot
                .as_ref()
                .is_some_and(|old| !projection_inputs_changed(old, &fixture));
            (fixture, unchanged)
        }
        None => {
            let unchanged = state
                .core_generations
                .as_deref()
                .is_some_and(|old| !slice_generations_changed(old, &view.generations));
            (view.snapshot, unchanged)
        }
    };
    fluxa_effects::flow(|| format!("projection inputs changed: {}", !unchanged));
    state.core_generations = live_generations;
    timer.mark("snap-compare");
    if let Some(old) = state.core_snapshot.replace(snapshot) {
        retire(old);
    }
    if unchanged {
        return;
    }
    state.core_snapshot_revision = state.core_snapshot_revision.wrapping_add(1);
    sync_home_from_core_snapshot(state);
    timer.mark("snap-home");
}

pub(super) const UNPROJECTED_SLICES: [&str; 5] =
    ["trailer", "lookup", "auth", "watchTogether", "_runtime"];

pub(super) fn slice_generations_changed(
    old: &fluxa_effects::SliceGenerations,
    new: &fluxa_effects::SliceGenerations,
) -> bool {
    let generation =
        |slices: &fluxa_effects::SliceGenerations, key: &str| slices.get(key).copied().unwrap_or(0);
    old.keys()
        .chain(new.keys())
        .filter(|key| !UNPROJECTED_SLICES.contains(&key.as_str()))
        .any(|key| generation(old, key) != generation(new, key))
}

pub(super) fn projection_inputs_changed(old: &Snapshot, new: &Snapshot) -> bool {
    old.names()
        .chain(new.names())
        .filter(|name| !UNPROJECTED_SLICES.contains(name))
        .any(|name| !old.slice_equal(new, name))
}

pub(crate) fn discover_command(
    profile: &Value,
    content_type: &str,
    catalog_key: &str,
    extra_name: &str,
    extra_value: &str,
    load_catalog_filters: bool,
) -> Value {
    let mut extra = serde_json::Map::new();
    if !extra_name.is_empty() && !extra_value.is_empty() {
        extra.insert(extra_name.to_owned(), json!(extra_value));
    }
    json!({
        "type": "discoverRequested",
        "loadCatalogFilters": load_catalog_filters,
        "contentType": content_type,
        "filters": {
            "catalogKey": (!catalog_key.is_empty()).then_some(catalog_key),
            "extra": extra,
        },
        "profile": profile,
        "language": profile_language(profile),
    })
}

pub(crate) fn folder_commands(
    snapshot: Option<&Snapshot>,
    folder_id: &str,
    profile: &Value,
) -> Vec<Value> {
    let mut commands = vec![json!({
        "type": "navigationRequested",
        "route": Route::Folder.as_str(),
        "params": {"folderId": folder_id},
    })];
    let categories = snapshot
        .and_then(|snapshot| snapshot.pointer("/home/categories"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for category in categories {
        let is_source = category.get("type").and_then(Value::as_str)
            == Some("collection_folder_source")
            && category.get("folderId").and_then(Value::as_str) == Some(folder_id);
        let loaded = category
            .get("items")
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty());
        if !is_source || loaded {
            continue;
        }
        commands.push(json!({
            "type": "catalogPageRequested",
            "categoryId": category.get("id"),
            "transportUrl": category.get("transportUrl"),
            "contentType": category.get("contentType"),
            "catalogId": category.get("catalogId"),
            "skip": 0,
            "genre": category.get("genre"),
            "remoteSource": category.get("remoteSource"),
            "profile": profile,
        }));
    }
    commands
}

pub(crate) fn navigation(route: Route) -> Value {
    json!({"type": "navigationRequested", "route": route.as_str(), "params": {}})
}

#[cfg(test)]
pub(crate) mod bench_support {
    use serde_json::{Value, json};

    fn item(seed: usize) -> Value {
        json!({
            "id": format!("tt{seed:07}"),
            "type": if seed % 3 == 0 { "series" } else { "movie" },
            "name": format!("Sample Title Number {seed}"),
            "poster": format!("https://images.example.test/poster/{seed}.jpg"),
            "background": format!("https://images.example.test/background/{seed}.jpg"),
            "logo": format!("https://images.example.test/logo/{seed}.png"),
            "description": "A long synopsis that repeats itself to look like real catalog metadata. ".repeat(4),
            "releaseInfo": "2021",
            "imdbRating": "7.4",
            "runtime": "118 min",
            "genres": ["Action", "Drama", "Thriller"],
            "cast": ["Actor One", "Actor Two", "Actor Three", "Actor Four"],
            "director": ["Director One"],
            "links": [
                {"name": "Action", "category": "Genres", "url": "stremio:///discover/a"},
                {"name": "Drama", "category": "Genres", "url": "stremio:///discover/b"}
            ],
            "behaviorHints": {"defaultVideoId": null, "hasScheduledVideos": false},
        })
    }

    fn items(count: usize, offset: usize) -> Vec<Value> {
        (0..count).map(|index| item(offset + index)).collect()
    }

    pub(crate) fn large_snapshot() -> Value {
        let categories: Vec<Value> = (0..14)
            .map(|index| {
                json!({
                    "id": format!("cat-{index}"),
                    "title": format!("Catalog {index}"),
                    "type": "catalog",
                    "items": items(30, index * 100),
                })
            })
            .collect();
        json!({
            "navigation": {"route": "home", "params": {}},
            "home": {"categories": categories, "isLoading": false},
            "library": {"items": items(400, 5000), "hydrated": true, "tab": "all"},
            "discover": {"results": items(200, 9000), "paging": {"hasMore": true, "nextSkip": 200}},
            "detail": {"item": item(1), "streams": []},
            "search": {"query": "", "results": []},
            "settings": {"values": {"language": "en", "crashReports": true}},
            "profile": {"active": {"id": "p1", "language": "en"}},
            "addons": {"installed": items(30, 12000)},
            "plugins": {"repositories": []},
            "trailer": {"resolutions": {}},
            "_runtime": {"ready": true},
        })
    }
}

#[cfg(test)]
mod bench {
    use super::bench_support::large_snapshot;
    use super::*;
    use std::time::Instant;

    fn best_of(runs: usize, mut run: impl FnMut()) -> f64 {
        (0..runs)
            .map(|_| {
                let started = Instant::now();
                run();
                started.elapsed().as_secs_f64() * 1000.0
            })
            .fold(f64::MAX, f64::min)
    }

    fn generations(entries: &[(&str, u64)]) -> fluxa_effects::SliceGenerations {
        entries
            .iter()
            .map(|(slice, generation)| ((*slice).to_owned(), *generation))
            .collect()
    }

    #[test]
    fn trailer_generation_bump_does_not_reproject() {
        let old = generations(&[("home", 2), ("trailer", 4)]);
        let new = generations(&[("home", 2), ("trailer", 5)]);
        assert!(!slice_generations_changed(&old, &new));
    }

    #[test]
    fn home_generation_bump_reprojects() {
        let old = generations(&[("home", 2)]);
        let new = generations(&[("home", 3)]);
        assert!(slice_generations_changed(&old, &new));
    }

    #[test]
    fn first_dirty_slice_after_empty_reprojects() {
        let new = generations(&[("library", 1)]);
        assert!(slice_generations_changed(&generations(&[]), &new));
        assert!(!slice_generations_changed(
            &generations(&[]),
            &generations(&[])
        ));
    }

    #[test]
    #[ignore]
    fn snapshot_phase_costs() {
        let base = large_snapshot();
        let bytes = serde_json::to_vec(&base).unwrap().len();
        let old = Snapshot::from_value(base.clone());
        let new = Snapshot::from_value(base.clone());
        let clone = best_of(20, || {
            std::hint::black_box(base.clone());
        });
        let compare = best_of(20, || {
            std::hint::black_box(projection_inputs_changed(&old, &new));
        });
        let generations: fluxa_effects::SliceGenerations =
            ["home", "library", "discover", "detail"]
                .into_iter()
                .map(|slice| (slice.to_owned(), 3))
                .collect();
        let generation_compare = best_of(20, || {
            std::hint::black_box(slice_generations_changed(
                &generations,
                &generations.clone(),
            ));
        });
        let arc_swap = best_of(20, || {
            std::hint::black_box(new.clone());
        });
        println!("snapshot size: {:.1} KiB", bytes as f64 / 1024.0);
        println!("deep clone (session thread, per revision): {clone:.3} ms");
        println!("deep compare (render thread, per revision): {compare:.3} ms");
        println!("generation compare (render thread, per revision): {generation_compare:.6} ms");
        println!("arc handoff: {arc_swap:.6} ms");
    }
}
