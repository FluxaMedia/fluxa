use super::the_introdb_wire_type;
use serde_json::{Value, json};

fn request_plan(
    url: String,
    method: &str,
    headers: &[(&str, String)],
    body: Option<&Value>,
) -> Value {
    let headers_obj: serde_json::Map<String, Value> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), Value::String(v.clone())))
        .collect();
    json!({
        "url": url,
        "method": method,
        "headers": headers_obj,
        "body": body.map(|b| serde_json::to_string(b).unwrap_or_default()),
    })
}

pub(crate) fn intro_db_segments_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let imdb_id = args.get("imdbId").and_then(Value::as_str)?;
    let season = args.get("season").and_then(Value::as_i64)?;
    let episode = args.get("episode").and_then(Value::as_i64)?;
    let url = format!(
        "https://api.introdb.app/segments?imdb_id={imdb_id}&season={season}&episode={episode}"
    );
    serde_json::to_string(&request_plan(url, "GET", &[], None)).ok()
}

pub(crate) fn intro_db_submit_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let api_key = args.get("apiKey").and_then(Value::as_str)?;
    let imdb_id = args.get("imdbId").and_then(Value::as_str)?;
    let season = args.get("season").and_then(Value::as_i64)?;
    let episode = args.get("episode").and_then(Value::as_i64)?;
    let segment_type = args.get("segmentType").and_then(Value::as_str)?;
    let start_sec = args.get("startSec").and_then(Value::as_f64)?;
    let end_sec = args.get("endSec").and_then(Value::as_f64)?;
    let body = json!({
        "imdb_id": imdb_id,
        "season": season,
        "episode": episode,
        "segment_type": segment_type,
        "start_sec": start_sec,
        "end_sec": end_sec,
    });
    let headers = [
        ("Content-Type", "application/json".to_string()),
        ("X-API-Key", api_key.to_string()),
    ];
    serde_json::to_string(&request_plan(
        "https://api.introdb.app/submit".to_string(),
        "POST",
        &headers,
        Some(&body),
    ))
    .ok()
}

pub(crate) fn skipdb_segments_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let imdb_id = args.get("imdbId").and_then(Value::as_str)?;
    let season = args.get("season").and_then(Value::as_i64)?;
    let episode = args.get("episode").and_then(Value::as_i64)?;
    let url = format!(
        "https://api.skipdb.tv/api/segments?imdb_id={imdb_id}&season={season}&episode={episode}"
    );
    serde_json::to_string(&request_plan(url, "GET", &[], None)).ok()
}

pub(crate) fn skipdb_submit_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let api_key = args.get("apiKey").and_then(Value::as_str)?;
    let imdb_id = args.get("imdbId").and_then(Value::as_str)?;
    let season = args.get("season").and_then(Value::as_i64)?;
    let episode = args.get("episode").and_then(Value::as_i64)?;
    let segment_type = args.get("segmentType").and_then(Value::as_str)?;
    let start_ms = args.get("startMs").and_then(Value::as_i64)?;
    let end_ms = args.get("endMs").and_then(Value::as_i64)?;
    let body = json!({
        "imdb_id": imdb_id,
        "season": season,
        "episode": episode,
        "segment_type": segment_type,
        "start_ms": start_ms,
        "end_ms": end_ms,
    });
    let headers = [
        ("Content-Type", "application/json".to_string()),
        ("X-API-Key", api_key.to_string()),
    ];
    serde_json::to_string(&request_plan(
        "https://api.skipdb.tv/api/segments".to_string(),
        "POST",
        &headers,
        Some(&body),
    ))
    .ok()
}

fn clean_anilist_title(title: &str) -> String {
    let trimmed = title.trim_end();
    if let Some(idx) = trimmed.rfind(" (") {
        let inner = &trimmed[idx + 2..];
        if trimmed.ends_with(')')
            && inner.len() == 5
            && inner[..4].chars().all(|c| c.is_ascii_digit())
        {
            return trimmed[..idx].trim().to_string();
        }
    }
    trimmed.trim().to_string()
}

pub(crate) fn anilist_media_id_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let title = args.get("title").and_then(Value::as_str)?;
    let field = args.get("field").and_then(Value::as_str).unwrap_or("id");
    let search = clean_anilist_title(title);
    if search.chars().count() < 2 {
        return None;
    }
    let body = json!({
        "query": format!("query ($search: String) {{ Media(search: $search, type: ANIME) {{ {field} }} }}"),
        "variables": { "search": search },
    });
    let headers = [("Content-Type", "application/json".to_string())];
    serde_json::to_string(&request_plan(
        "https://graphql.anilist.co".to_string(),
        "POST",
        &headers,
        Some(&body),
    ))
    .ok()
}

pub(crate) fn aniskip_segments_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mal_id = args.get("malId").and_then(Value::as_i64)?;
    let episode = args.get("episode").and_then(Value::as_i64)?;
    let url = format!(
        "https://api.aniskip.com/v2/skip-times/{mal_id}/{episode}?episodeLength=0&types=op&types=ed&types=recap"
    );
    serde_json::to_string(&request_plan(url, "GET", &[], None)).ok()
}

fn anime_skip_graphql_plan(client_id: &str, query: &str, variables: Value) -> Value {
    let body = json!({ "query": query, "variables": variables });
    let headers = [
        ("Content-Type", "application/json".to_string()),
        ("X-Client-ID", client_id.to_string()),
    ];
    request_plan(
        "https://api.anime-skip.com/graphql".to_string(),
        "POST",
        &headers,
        Some(&body),
    )
}

pub(crate) fn anime_skip_find_show_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let client_id = args.get("clientId").and_then(Value::as_str)?;
    let anilist_id = args.get("anilistId").and_then(Value::as_i64)?;
    let plan = anime_skip_graphql_plan(
        client_id,
        "query ($service: String!, $serviceId: String!) { findShowsByExternalId(service: $service, serviceId: $serviceId) { id } }",
        json!({ "service": "anilist.co", "serviceId": anilist_id.to_string() }),
    );
    serde_json::to_string(&plan).ok()
}

pub(crate) fn anime_skip_find_episodes_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let client_id = args.get("clientId").and_then(Value::as_str)?;
    let show_id = args.get("showId").and_then(Value::as_str)?;
    let plan = anime_skip_graphql_plan(
        client_id,
        "query ($showId: ID!) { findEpisodesByShowId(showId: $showId) { id season number absoluteNumber } }",
        json!({ "showId": show_id }),
    );
    serde_json::to_string(&plan).ok()
}

pub(crate) fn anime_skip_find_timestamps_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let client_id = args.get("clientId").and_then(Value::as_str)?;
    let episode_id = args.get("episodeId").and_then(Value::as_str)?;
    let plan = anime_skip_graphql_plan(
        client_id,
        "query ($episodeId: ID!) { findTimestampsByEpisodeId(episodeId: $episodeId) { at type { name } } }",
        json!({ "episodeId": episode_id }),
    );
    serde_json::to_string(&plan).ok()
}

pub(crate) fn the_introdb_media_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let tmdb_id = args.get("tmdbId").and_then(Value::as_i64);
    let imdb_id = args
        .get("imdbId")
        .and_then(Value::as_str)
        .filter(|s| s.starts_with("tt"));
    let season = args
        .get("season")
        .and_then(Value::as_i64)
        .filter(|s| *s > 0);
    let episode = args
        .get("episode")
        .and_then(Value::as_i64)
        .filter(|e| *e > 0);
    let duration_ms = args
        .get("durationMs")
        .and_then(Value::as_i64)
        .filter(|d| *d > 0);

    let mut query = match (tmdb_id, imdb_id) {
        (Some(id), _) => format!("tmdb_id={id}"),
        (None, Some(id)) => format!("imdb_id={id}"),
        (None, None) => return None,
    };
    if let (Some(s), Some(e)) = (season, episode) {
        query.push_str(&format!("&season={s}&episode={e}"));
    }
    if let Some(d) = duration_ms {
        query.push_str(&format!("&duration_ms={d}"));
    }
    let url = format!("https://api.theintrodb.org/v3/media?{query}");
    serde_json::to_string(&request_plan(url, "GET", &[], None)).ok()
}

pub(crate) fn the_introdb_submit_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let api_key = args.get("apiKey").and_then(Value::as_str)?;
    let tmdb_id = args.get("tmdbId").and_then(Value::as_i64)?;
    let media_type = args.get("mediaType").and_then(Value::as_str)?;
    let segment = args.get("segment").and_then(Value::as_str)?;
    let start_sec = args.get("startSec").and_then(Value::as_f64);
    let end_sec = args.get("endSec").and_then(Value::as_f64);
    let video_duration_ms = args.get("videoDurationMs").and_then(Value::as_i64);
    let imdb_id = args.get("imdbId").and_then(Value::as_str);
    let season = args
        .get("season")
        .and_then(Value::as_i64)
        .filter(|s| *s > 0);
    let episode = args
        .get("episode")
        .and_then(Value::as_i64)
        .filter(|e| *e > 0);

    let mut body = json!({
        "tmdb_id": tmdb_id,
        "type": media_type,
        "segment": the_introdb_wire_type(segment),
        "start_sec": start_sec,
        "end_sec": end_sec,
    });
    let obj = body.as_object_mut()?;
    if let (Some(s), Some(e)) = (season, episode) {
        obj.insert("season".to_string(), json!(s.to_string()));
        obj.insert("episode".to_string(), json!(e.to_string()));
    }
    if let Some(d) = video_duration_ms {
        obj.insert("video_duration_ms".to_string(), json!(d));
    }
    if let Some(id) = imdb_id {
        obj.insert("imdb_id".to_string(), json!(id));
    }

    let headers = [
        ("Content-Type", "application/json".to_string()),
        ("Authorization", format!("Bearer {api_key}")),
    ];
    serde_json::to_string(&request_plan(
        "https://api.theintrodb.org/v3/submit".to_string(),
        "POST",
        &headers,
        Some(&body),
    ))
    .ok()
}
