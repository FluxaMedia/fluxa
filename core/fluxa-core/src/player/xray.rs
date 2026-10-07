use crate::services::metadata::tmdb::tmdb_api_url;
use serde::Deserialize;
use serde_json::{Map, Value, json};

const CAST_LIMIT: usize = 100;
const PHOTO_BASE: &str = "https://image.tmdb.org/t/p/w185";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanInput {
    api_key: String,
    #[serde(default)]
    language: String,
    #[serde(default)]
    media_type: String,
    #[serde(default)]
    tmdb_id: Option<i64>,
    #[serde(default)]
    imdb_id: Option<String>,
    #[serde(default)]
    season: Option<i64>,
    #[serde(default)]
    episode: Option<i64>,
}

pub(crate) fn player_xray_credits_plan_json(args_json: &str) -> Option<String> {
    let input: PlanInput = serde_json::from_str(args_json).ok()?;
    if input.api_key.is_empty() {
        return None;
    }
    let series = input.media_type == "tv";
    let plan = match (input.tmdb_id, input.imdb_id.as_deref()) {
        (Some(id), _) => {
            let (path, extra): (String, &[(&str, &str)]) =
                match (series, input.season, input.episode) {
                    (true, Some(season), Some(episode)) => (
                        format!("3/tv/{id}/season/{season}/episode/{episode}"),
                        &[("append_to_response", "credits")],
                    ),
                    (true, _, _) => (format!("3/tv/{id}/credits"), &[]),
                    _ => (format!("3/movie/{id}/credits"), &[]),
                };
            let mut plan = json!({
                "url": tmdb_api_url(&path, &input.api_key, &input.language, extra)
            });
            if let (true, Some(_), Some(_)) = (series, input.season, input.episode) {
                let path = format!("3/tv/{id}/credits");
                plan["seriesUrl"] = json!(tmdb_api_url(&path, &input.api_key, &input.language, &[]));
            }
            plan
        }
        (None, Some(imdb)) => json!({
            "findUrl": tmdb_api_url(
                &format!("3/find/{imdb}"),
                &input.api_key,
                &input.language,
                &[("external_source", "imdb_id")],
            )
        }),
        _ => return None,
    };
    serde_json::to_string(&plan).ok()
}

pub(crate) fn player_xray_find_id_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let key = if args.get("mediaType")?.as_str()? == "tv" {
        "tv_results"
    } else {
        "movie_results"
    };
    let id = args.get("find")?.get(key)?.get(0)?.get("id")?.as_i64()?;
    serde_json::to_string(&json!({ "tmdbId": id })).ok()
}

pub(crate) fn player_xray_cast_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let empty = json!({});
    let episode = args.get("credits").unwrap_or(&empty);
    let series = args.get("series").unwrap_or(&empty);
    let mut cast: Vec<Value> = Vec::new();
    let nested = episode.get("credits").unwrap_or(&empty);
    for (credits, key) in [
        (nested, "cast"),
        (episode, "cast"),
        (episode, "guest_stars"),
        (nested, "guest_stars"),
        (series, "cast"),
    ] {
        for person in credits.get(key).and_then(Value::as_array).into_iter().flatten() {
            let (Some(id), Some(name)) = (
                person.get("id").and_then(Value::as_i64),
                person.get("name").and_then(Value::as_str),
            ) else {
                continue;
            };
            if cast.iter().any(|known| known["id"] == id) {
                continue;
            }
            let photo = person
                .get("profile_path")
                .and_then(Value::as_str)
                .map(|path| format!("{PHOTO_BASE}{path}"));
            cast.push(json!({
                "id": id,
                "name": name,
                "character": person.get("character").and_then(Value::as_str).filter(|c| !c.is_empty()),
                "photo": photo,
            }));
            if cast.len() >= CAST_LIMIT {
                break;
            }
        }
    }
    serde_json::to_string(&json!({ "cast": cast })).ok()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RowsInput {
    cast: Vec<Value>,
    #[serde(default)]
    marks: Map<String, Value>,
    position_ms: i64,
}

pub(crate) fn player_xray_rows_json(args_json: &str) -> Option<String> {
    let input: RowsInput = serde_json::from_str(args_json).ok()?;
    let mark_of = |person: &Value| {
        input
            .marks
            .get(&person["id"].as_i64()?.to_string())?
            .as_i64()
    };
    let mut appeared: Vec<(i64, Value)> = Vec::new();
    let mut pending = Vec::new();
    for person in input.cast {
        match mark_of(&person) {
            Some(start) if start <= input.position_ms => appeared.push((start, person)),
            _ => pending.push(person),
        }
    }
    appeared.sort_by_key(|(start, _)| *start);
    let appeared: Vec<Value> = appeared
        .into_iter()
        .map(|(start, mut person)| {
            person["startMs"] = json!(start);
            person
        })
        .collect();
    serde_json::to_string(&json!({ "appeared": appeared, "pending": pending })).ok()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToggleInput {
    #[serde(default)]
    marks: Map<String, Value>,
    person_id: i64,
    position_ms: i64,
}

pub(crate) fn player_xray_toggle_json(args_json: &str) -> Option<String> {
    let mut input: ToggleInput = serde_json::from_str(args_json).ok()?;
    let key = input.person_id.to_string();
    let shown = input
        .marks
        .get(&key)
        .and_then(Value::as_i64)
        .is_some_and(|start| start <= input.position_ms);
    if shown {
        input.marks.remove(&key);
    } else {
        input.marks.insert(key, json!(input.position_ms.max(0)));
    }
    serde_json::to_string(&json!({ "marks": input.marks })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: Option<String>) -> Value {
        serde_json::from_str(&text.unwrap()).unwrap()
    }

    #[test]
    fn guest_stars_do_not_duplicate_cast() {
        let credits = json!({
            "cast": [{ "id": 1, "name": "Aaron Paul", "character": "Jesse", "profile_path": "/a.jpg" }],
            "guest_stars": [
                { "id": 1, "name": "Aaron Paul" },
                { "id": 2, "name": "Guest", "character": "" }
            ]
        });
        let cast = parse(player_xray_cast_json(&json!({ "credits": credits }).to_string()));
        assert_eq!(cast["cast"].as_array().unwrap().len(), 2);
        assert_eq!(cast["cast"][0]["photo"], "https://image.tmdb.org/t/p/w185/a.jpg");
        assert!(cast["cast"][1]["character"].is_null());
    }

    #[test]
    fn series_cast_follows_episode_cast() {
        let args = json!({
            "credits": { "cast": [{ "id": 1, "name": "A" }] },
            "series": { "cast": [{ "id": 1, "name": "A" }, { "id": 3, "name": "C" }] }
        });
        let cast = parse(player_xray_cast_json(&args.to_string()));
        let ids: Vec<i64> = cast["cast"]
            .as_array()
            .unwrap()
            .iter()
            .map(|person| person["id"].as_i64().unwrap())
            .collect();
        assert_eq!(ids, [1, 3]);
    }

    #[test]
    fn episode_details_guest_stars_are_read() {
        let args = json!({
            "credits": {
                "guest_stars": [{ "id": 7, "name": "G" }],
                "credits": { "cast": [{ "id": 1, "name": "A" }] }
            }
        });
        let cast = parse(player_xray_cast_json(&args.to_string()));
        assert_eq!(cast["cast"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn future_mark_stays_hidden_and_moves_earlier_on_toggle() {
        let marks = json!({ "1": 600_000 });
        let cast = json!([{ "id": 1, "name": "A" }, { "id": 2, "name": "B" }]);
        let rows = parse(player_xray_rows_json(
            &json!({ "cast": cast, "marks": marks, "positionMs": 60_000 }).to_string(),
        ));
        assert!(rows["appeared"].as_array().unwrap().is_empty());
        assert_eq!(rows["pending"].as_array().unwrap().len(), 2);

        let moved = parse(player_xray_toggle_json(
            &json!({ "marks": marks, "personId": 1, "positionMs": 60_000 }).to_string(),
        ));
        assert_eq!(moved["marks"]["1"], 60_000);
    }

    #[test]
    fn toggling_a_shown_person_removes_the_mark() {
        let removed = parse(player_xray_toggle_json(
            &json!({ "marks": { "1": 1000 }, "personId": 1, "positionMs": 5000 }).to_string(),
        ));
        assert!(removed["marks"].as_object().unwrap().is_empty());
    }

    #[test]
    fn imdb_only_requests_a_find_first() {
        let plan = parse(player_xray_credits_plan_json(
            &json!({ "apiKey": "k", "mediaType": "tv", "imdbId": "tt0903747", "season": 1, "episode": 1 })
                .to_string(),
        ));
        assert!(plan["findUrl"].as_str().unwrap().contains("find/tt0903747"));
        assert!(plan.get("url").is_none());
    }
}
