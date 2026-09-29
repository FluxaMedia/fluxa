use super::provider::{SIMKL_API, provider_library_snapshot_json, request};
use serde_json::{Map, Value, json};

const TYPES: [(&str, &str); 3] = [
    ("shows", "tv_shows"),
    ("movies", "movies"),
    ("anime", "anime"),
];
const FULL_RESYNC_SECONDS: i64 = 7 * 24 * 60 * 60;
const STATUSES: [&str; 5] = ["plantowatch", "watching", "completed", "dropped", "hold"];

fn keyed(args: &Value, key: &str, path: String) -> Value {
    let mut plan = request(
        "simkl",
        args,
        "GET",
        format!("{SIMKL_API}{path}"),
        Value::Null,
    );
    plan["key"] = json!(key);
    plan
}

pub(crate) fn simkl_calendar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let number = |key: &str| args.get(key).and_then(Value::as_i64);
    let (year, month) = (number("year")?, number("month")?);
    let offset = (year * 12 + month) - (number("nowYear")? * 12 + number("nowMonth")?);
    if !(-12..=3).contains(&offset) {
        return Some("[]".to_owned());
    }
    let client_id = args.get("clientId").and_then(Value::as_str).unwrap_or("");
    let requests: Vec<Value> = [
        ("shows", "tv"),
        ("anime", "anime"),
        ("movies", "movie_release"),
    ]
    .into_iter()
    .map(|(key, file)| {
        let mut plan = request(
            "simkl",
            &json!({"clientId": client_id}),
            "GET",
            format!(
                "https://data.simkl.in/calendar/v2/{year}/{month}/{file}.json?client_id={client_id}"
            ),
            Value::Null,
        );
        plan["key"] = json!(key);
        plan
    })
    .collect();
    serde_json::to_string(&requests).ok()
}

fn stamp<'a>(activities: &'a Value, domain: Option<&str>, field: &str) -> Option<&'a str> {
    let scope = match domain {
        Some(domain) => activities.get(domain)?,
        None => activities,
    };
    scope.get(field)?.as_str()
}

pub(crate) fn simkl_sync_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let Some(activities) = args.get("activities").filter(|value| value.is_object()) else {
        let plan = json!({"mode": "activities", "requests": [keyed(&args, "activities", "/sync/activities".into())]});
        return serde_json::to_string(&plan).ok();
    };
    let now = args.get("nowSeconds").and_then(Value::as_i64);
    let stale = now.is_some_and(|now| {
        args.pointer("/state/fullAt")
            .and_then(Value::as_i64)
            .is_none_or(|full_at| now - full_at > FULL_RESYNC_SECONDS)
    });
    let saved = args
        .get("state")
        .and_then(|state| state.get("activities"))
        .filter(|value| value.is_object() && !stale);
    let Some(saved) = saved else {
        let mut requests: Vec<Value> = TYPES
            .iter()
            .map(|(kind, _)| {
                keyed(
                    &args,
                    &format!("items_{kind}"),
                    format!("/sync/all-items/{kind}"),
                )
            })
            .collect();
        requests.push(keyed(&args, "playback", "/sync/playback".into()));
        return serde_json::to_string(&json!({"mode": "full", "requests": requests})).ok();
    };
    if stamp(activities, None, "all") == stamp(saved, None, "all") {
        return serde_json::to_string(&json!({"mode": "cached", "requests": []})).ok();
    }
    let mut requests = Vec::new();
    let mut playback_moved = false;
    for (kind, domain) in TYPES {
        if stamp(activities, Some(domain), "all") == stamp(saved, Some(domain), "all") {
            continue;
        }
        let since = stamp(saved, Some(domain), "all").or_else(|| stamp(saved, None, "all"));
        let path = match since {
            Some(since) => format!("/sync/all-items/{kind}?date_from={since}"),
            None => format!("/sync/all-items/{kind}"),
        };
        requests.push(keyed(&args, &format!("items_{kind}"), path));
        if stamp(activities, Some(domain), "removed_from_list")
            != stamp(saved, Some(domain), "removed_from_list")
        {
            requests.push(keyed(
                &args,
                &format!("ids_{kind}"),
                format!("/sync/all-items/{kind}?extended=simkl_ids_only"),
            ));
        }
        playback_moved |=
            stamp(activities, Some(domain), "playback") != stamp(saved, Some(domain), "playback");
    }
    if playback_moved {
        requests.push(keyed(&args, "playback", "/sync/playback".into()));
    }
    let mode = if requests.is_empty() {
        "cached"
    } else {
        "delta"
    };
    serde_json::to_string(&json!({"mode": mode, "requests": requests})).ok()
}

fn entry_id(entry: &Value) -> Option<i64> {
    ["show", "movie", "anime"]
        .iter()
        .filter_map(|block| entry.get(block))
        .chain([entry])
        .find_map(|source| source.get("ids")?.get("simkl")?.as_i64())
}

fn response_entries(response: &Value, kind: &str) -> Vec<Value> {
    response
        .get(kind)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn simkl_sync_apply_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let responses = args.get("responses")?;
    let full = args.get("mode").and_then(Value::as_str) == Some("full");
    let full_at = if full {
        args.get("nowSeconds").and_then(Value::as_i64)
    } else {
        args.pointer("/state/fullAt").and_then(Value::as_i64)
    };
    let previous = args.get("state").filter(|_| !full);
    let mut items = Map::new();
    for (kind, _) in TYPES {
        let mut current = previous
            .and_then(|state| state.pointer(&format!("/items/{kind}")))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if let Some(response) = responses.get(&format!("items_{kind}")) {
            for entry in response_entries(response, kind) {
                let position = entry_id(&entry)
                    .and_then(|id| current.iter().position(|old| entry_id(old) == Some(id)));
                match position {
                    Some(position) => current[position] = entry,
                    None => current.push(entry),
                }
            }
        }
        if let Some(response) = responses.get(&format!("ids_{kind}")) {
            let alive: Vec<i64> = response_entries(response, kind)
                .iter()
                .filter_map(entry_id)
                .collect();
            current.retain(|entry| entry_id(entry).is_some_and(|id| alive.contains(&id)));
        }
        items.insert(kind.to_owned(), Value::Array(current));
    }
    let playback = responses
        .get("playback")
        .or_else(|| previous.and_then(|state| state.get("playback")))
        .cloned()
        .unwrap_or_else(|| json!([]));
    let mut buckets = Map::new();
    for (kind, _) in TYPES {
        let entries = items.get(kind).and_then(Value::as_array)?;
        for status in STATUSES {
            let matching: Vec<Value> = entries
                .iter()
                .filter(|entry| entry.get("status").and_then(Value::as_str) == Some(status))
                .cloned()
                .collect();
            buckets.insert(format!("{status}_{kind}"), json!({ kind: matching }));
        }
    }
    buckets.insert("playback".to_owned(), playback.clone());
    let snapshot: Value = serde_json::from_str(&provider_library_snapshot_json(
        &json!({"provider": "simkl", "responses": buckets}).to_string(),
    )?)
    .ok()?;
    serde_json::to_string(&json!({
        "state": {"activities": args.get("activities")?, "items": items, "playback": playback, "fullAt": full_at},
        "snapshot": snapshot,
    }))
    .ok()
}

#[cfg(test)]
mod tests {
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
            let args = json!({"clientId": "c", "year": year, "month": month, "nowYear": 2026, "nowMonth": 9});
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
}
