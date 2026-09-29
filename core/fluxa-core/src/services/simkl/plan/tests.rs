use super::*;

fn plan(args: Value) -> Value {
    serde_json::from_str(&simkl_sync_plan_json(&args.to_string()).unwrap()).unwrap()
}

fn apply(args: Value) -> Value {
    serde_json::from_str(&simkl_sync_apply_json(&args.to_string()).unwrap()).unwrap()
}

fn entry(id: i64, status: &str) -> Value {
    json!({"status": status, "show": {"title": "Show", "ids": {"simkl": id, "imdb": format!("tt{id}")}}})
}

#[test]
fn first_sync_pulls_each_type_then_playback() {
    let planned = plan(json!({"clientId": "c", "token": "t", "activities": {"all": "a"}}));
    let keys: Vec<_> = planned["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|request| request["key"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(planned["mode"], "full");
    assert_eq!(
        keys,
        ["items_shows", "items_movies", "items_anime", "playback"]
    );
}

#[test]
fn unchanged_activities_make_no_item_requests() {
    let activities = json!({"all": "2026-01-01T00:00:00Z"});
    let planned = plan(json!({"state": {"activities": activities}, "activities": activities}));
    assert_eq!(planned["mode"], "cached");
    assert!(planned["requests"].as_array().unwrap().is_empty());
}

#[test]
fn only_the_moved_domain_is_fetched_with_date_from() {
    let saved = json!({"all": "t1", "tv_shows": {"all": "t1"}, "movies": {"all": "t1"}, "anime": {"all": "t1"}});
    let now = json!({"all": "t2", "tv_shows": {"all": "t2", "removed_from_list": "t2"}, "movies": {"all": "t1"}, "anime": {"all": "t1"}});
    let planned = plan(json!({"state": {"activities": saved}, "activities": now}));
    let urls: Vec<_> = planned["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|request| request["url"].as_str().unwrap())
        .collect();
    assert_eq!(urls.len(), 2);
    assert!(urls[0].contains("/sync/all-items/shows?date_from=t1"));
    assert!(urls[1].contains("extended=simkl_ids_only"));
}

#[test]
fn calendar_plan_skips_months_outside_the_archive() {
    let plan = |year: i64, month: i64| -> Value {
        let args =
            json!({"clientId": "c", "year": year, "month": month, "nowYear": 2026, "nowMonth": 9});
        serde_json::from_str(&simkl_calendar_plan_json(&args.to_string()).unwrap()).unwrap()
    };
    assert_eq!(plan(2026, 12).as_array().unwrap().len(), 3);
    assert_eq!(plan(2027, 1).as_array().unwrap().len(), 0);
    assert_eq!(plan(2025, 8).as_array().unwrap().len(), 0);
    assert!(
        plan(2026, 12)[0]["url"]
            .as_str()
            .unwrap()
            .contains("/calendar/v2/2026/12/tv.json")
    );
}

#[test]
fn week_old_state_forces_a_full_pull() {
    let activities = json!({"all": "t1"});
    let stale = plan(json!({
        "nowSeconds": 1_000_000,
        "state": {"activities": activities, "fullAt": 1_000_000 - 8 * 24 * 60 * 60},
        "activities": activities,
    }));
    assert_eq!(stale["mode"], "full");
    let fresh = plan(json!({
        "nowSeconds": 1_000_000,
        "state": {"activities": activities, "fullAt": 1_000_000 - 60},
        "activities": activities,
    }));
    assert_eq!(fresh["mode"], "cached");
}

#[test]
fn delta_replaces_changed_items_and_drops_removed_ones() {
    let state = json!({
        "activities": {"all": "t1"},
        "items": {"shows": [entry(1, "watching"), entry(2, "watching")], "movies": [], "anime": []},
    });
    let applied = apply(json!({
        "mode": "delta",
        "state": state,
        "activities": {"all": "t2"},
        "responses": {
            "items_shows": {"shows": [entry(1, "completed")]},
            "ids_shows": {"shows": [{"show": {"ids": {"simkl": 1}}}]},
        },
    }));
    let shows = applied["state"]["items"]["shows"].as_array().unwrap();
    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0]["status"], "completed");
    assert_eq!(applied["state"]["activities"]["all"], "t2");
}
