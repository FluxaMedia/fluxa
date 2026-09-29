use super::{
    anilist_id_json, anilist_mal_id_json, anilist_media_id_plan_json,
    anime_skip_find_episodes_plan_json, anime_skip_find_show_plan_json,
    anime_skip_find_timestamps_plan_json, anime_skip_show_id_json, aniskip_segments_plan_json,
    intro_db_segments_plan_json, match_anime_skip_episode_id, merge_intro_segments_json,
    parse_anime_skip_results_json, parse_aniskip_results_json, parse_intro_db_segments_json,
    parse_publicmetadb_segments_json, parse_skipdb_segments_json, parse_the_introdb_segments_json,
    skipdb_segments_plan_json, the_introdb_media_plan_json,
};
use crate::player::playback::desktop::chapter_skip_segments_json;
use crate::services::publicmetadb::{publicmetadb_bearer, publicmetadb_skips_url};
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Context {
    imdb_id: Option<String>,
    tmdb_id: Option<i64>,
    media_type: String,
    season: Option<i64>,
    episode: Option<i64>,
    title: Option<String>,
    duration_ms: Option<i64>,
    anime: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Providers {
    intro_db: bool,
    skip_db: bool,
    the_intro_db: bool,
    ani_skip: bool,
    anime_skip_client_id: Option<String>,
    public_meta_db_key: Option<String>,
    chapters: bool,
}

impl Default for Providers {
    fn default() -> Self {
        Self {
            intro_db: true,
            skip_db: true,
            the_intro_db: true,
            ani_skip: true,
            anime_skip_client_id: None,
            public_meta_db_key: None,
            chapters: true,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Input {
    ctx: Context,
    providers: Providers,
    chapters: Value,
    responses: Map<String, Value>,
}

fn request(id: &str, plan: Option<String>) -> Option<Value> {
    let mut plan: Value = serde_json::from_str(&plan?).ok()?;
    plan.as_object_mut()?.insert("id".into(), json!(id));
    Some(plan)
}

fn tagged(segments: Option<String>, provider: &str) -> Vec<Value> {
    let parsed: Vec<Value> = segments
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    parsed
        .into_iter()
        .map(|mut segment| {
            if let Some(object) = segment.as_object_mut() {
                object.insert("provider".into(), json!(provider));
            }
            segment
        })
        .collect()
}

fn body(input: &Input, id: &str) -> Option<String> {
    let value = input.responses.get(id)?;
    (!value.is_null()).then(|| value.to_string())
}

fn episode_args(ctx: &Context) -> Option<(String, i64, i64)> {
    Some((ctx.imdb_id.clone()?, ctx.season?, ctx.episode?))
}

fn pending(input: &Input) -> Vec<Value> {
    let ctx = &input.ctx;
    let providers = &input.providers;
    let mut wanted: Vec<Value> = Vec::new();
    let mut ask = |id: &str, plan: Option<String>| {
        if !input.responses.contains_key(id)
            && let Some(request) = request(id, plan)
        {
            wanted.push(request);
        }
    };

    if let Some((imdb_id, season, episode)) = episode_args(ctx) {
        let args = json!({ "imdbId": imdb_id, "season": season, "episode": episode }).to_string();
        if providers.intro_db {
            ask("introdb", intro_db_segments_plan_json(&args));
        }
        if providers.skip_db {
            ask("skipdb", skipdb_segments_plan_json(&args));
        }
    }

    if providers.the_intro_db {
        let args = json!({
            "tmdbId": ctx.tmdb_id,
            "imdbId": ctx.imdb_id,
            "season": ctx.season,
            "episode": ctx.episode,
            "durationMs": ctx.duration_ms,
        })
        .to_string();
        ask("theintrodb", the_introdb_media_plan_json(&args));
    }

    if let (Some(key), Some(tmdb_id)) = (&providers.public_meta_db_key, ctx.tmdb_id) {
        let query = json!({
            "tmdb_id": tmdb_id,
            "media_type": ctx.media_type,
            "season": ctx.season,
            "episode": ctx.episode,
        })
        .to_string();
        let plan = publicmetadb_skips_url(&query).map(|url| {
            json!({
                "url": url,
                "method": "GET",
                "headers": { "Authorization": publicmetadb_bearer(key) },
                "body": null,
            })
            .to_string()
        });
        ask("publicmetadb", plan);
    }

    let wants_anime_skip = providers.anime_skip_client_id.is_some();
    if ctx.anime && (providers.ani_skip || wants_anime_skip) {
        if let Some(title) = &ctx.title {
            let args = json!({ "title": title, "field": "id idMal" }).to_string();
            ask("anilist", anilist_media_id_plan_json(&args));
        }
        chained_anime(input, &mut ask);
    }
    wanted
}

fn chained_anime(input: &Input, ask: &mut impl FnMut(&str, Option<String>)) {
    let ctx = &input.ctx;
    let episode = ctx.episode.unwrap_or(1);
    let Some(anilist) = body(input, "anilist") else {
        return;
    };
    if input.providers.ani_skip
        && let Some(mal_id) = anilist_mal_id_json(&anilist).and_then(|raw| raw.parse::<i64>().ok())
    {
        let args = json!({ "malId": mal_id, "episode": episode }).to_string();
        ask("aniskip", aniskip_segments_plan_json(&args));
    }
    let Some(client_id) = input.providers.anime_skip_client_id.as_deref() else {
        return;
    };
    let Some(anilist_id) = anilist_id_json(&anilist).and_then(|raw| raw.parse::<i64>().ok()) else {
        return;
    };
    let args = json!({ "clientId": client_id, "anilistId": anilist_id }).to_string();
    ask("animeskip-show", anime_skip_find_show_plan_json(&args));
    let Some(show_id) = body(input, "animeskip-show").and_then(|raw| anime_skip_show_id_json(&raw))
    else {
        return;
    };
    let show_id: String = serde_json::from_str(&show_id).unwrap_or_default();
    let args = json!({ "clientId": client_id, "showId": show_id }).to_string();
    ask(
        "animeskip-episodes",
        anime_skip_find_episodes_plan_json(&args),
    );
    let Some(episode_id) = body(input, "animeskip-episodes")
        .and_then(|raw| match_anime_skip_episode_id(&raw, ctx.season.unwrap_or(0), episode))
    else {
        return;
    };
    let args = json!({ "clientId": client_id, "episodeId": episode_id }).to_string();
    ask(
        "animeskip-times",
        anime_skip_find_timestamps_plan_json(&args),
    );
}

fn merged(input: &Input) -> Vec<Value> {
    let mut sources: Vec<Vec<Value>> = Vec::new();
    sources.push(tagged(
        body(input, "introdb").and_then(|raw| parse_intro_db_segments_json(&raw)),
        "IntroDB",
    ));
    sources.push(tagged(
        body(input, "skipdb").and_then(|raw| parse_skipdb_segments_json(&raw)),
        "SkipDB",
    ));
    sources.push(tagged(
        body(input, "theintrodb").and_then(|raw| {
            parse_the_introdb_segments_json(
                &json!({ "responseJson": raw, "durationMs": input.ctx.duration_ms }).to_string(),
            )
        }),
        "TheIntroDB",
    ));
    sources.push(tagged(
        body(input, "publicmetadb").and_then(|raw| parse_publicmetadb_segments_json(&raw)),
        "PublicMetaDB",
    ));
    sources.push(tagged(
        body(input, "aniskip").and_then(|raw| parse_aniskip_results_json(&raw)),
        "AniSkip",
    ));
    sources.push(tagged(
        body(input, "animeskip-times").and_then(|raw| parse_anime_skip_results_json(&raw)),
        "Anime-Skip",
    ));
    if input.providers.chapters
        && let Some(duration) = input.ctx.duration_ms.filter(|d| *d > 0)
        && !input.chapters.is_null()
    {
        let chapters = chapter_skip_segments_json(&input.chapters.to_string(), duration);
        sources.push(serde_json::from_str(&chapters).unwrap_or_default());
    }
    let sources: Vec<Value> = sources.into_iter().map(Value::Array).collect();
    merge_intro_segments_json(&Value::Array(sources).to_string())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub(crate) fn player_segments_step_json(input: &str) -> Option<String> {
    let input: Input = serde_json::from_str(input).ok()?;
    let requests = pending(&input);
    let output = json!({
        "requests": requests,
        "segments": merged(&input),
        "done": requests.is_empty(),
    });
    serde_json::to_string(&output).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(input: Value) -> Value {
        serde_json::from_str(&player_segments_step_json(&input.to_string()).unwrap()).unwrap()
    }

    fn ids(step: &Value) -> Vec<String> {
        step["requests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|request| request["id"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn episode_asks_every_keyless_provider_in_parallel() {
        let first = step(json!({
            "ctx": { "imdbId": "tt1", "tmdbId": 5, "mediaType": "tv", "season": 1, "episode": 2 },
        }));
        assert_eq!(ids(&first), ["introdb", "skipdb", "theintrodb"]);
        assert_eq!(first["done"], false);
    }

    #[test]
    fn anime_chain_walks_anilist_then_aniskip_then_anime_skip() {
        let mut responses = Map::new();
        let ctx = json!({
            "imdbId": "tt1", "season": 1, "episode": 3, "title": "Frieren", "anime": true,
        });
        let providers = json!({
            "introDb": false, "skipDb": false, "theIntroDb": false, "animeSkipClientId": "cid",
        });
        let run = |responses: &Map<String, Value>| {
            step(json!({ "ctx": ctx, "providers": providers, "responses": responses }))
        };
        assert_eq!(ids(&run(&responses)), ["anilist"]);
        responses.insert(
            "anilist".into(),
            json!({ "data": { "Media": { "id": 154587, "idMal": 52991 } } }),
        );
        assert_eq!(ids(&run(&responses)), ["aniskip", "animeskip-show"]);
        responses.insert("aniskip".into(), Value::Null);
        responses.insert(
            "animeskip-show".into(),
            json!({ "data": { "findShowsByExternalId": [{ "id": "show1" }] } }),
        );
        assert_eq!(ids(&run(&responses)), ["animeskip-episodes"]);
        responses.insert(
            "animeskip-episodes".into(),
            json!({ "data": { "findEpisodesByShowId": [
                { "id": "ep3", "season": "1", "number": "3", "absoluteNumber": "3" }
            ] } }),
        );
        assert_eq!(ids(&run(&responses)), ["animeskip-times"]);
        responses.insert(
            "animeskip-times".into(),
            json!({ "data": { "findTimestampsByEpisodeId": [
                { "at": 5.0, "type": { "name": "Intro" } },
                { "at": 95.0, "type": { "name": "Canon" } },
            ] } }),
        );
        let last = run(&responses);
        assert_eq!(last["done"], true);
        assert_eq!(last["segments"][0]["provider"], "Anime-Skip");
        assert_eq!(last["segments"][0]["startTime"], 5000);
    }

    #[test]
    fn providers_and_chapters_merge_into_one_sorted_list() {
        let result = step(json!({
            "ctx": { "imdbId": "tt1", "season": 1, "episode": 1, "durationMs": 3_000_000 },
            "providers": { "skipDb": false, "theIntroDb": false },
            "chapters": [
                { "startTime": 0, "title": "Opening" },
                { "startTime": 90_000, "title": "Episode" },
            ],
            "responses": {
                "introdb": { "segments": [{ "type": "recap", "startTime": 100_000, "endTime": 120_000 }] },
            },
        }));
        let kinds: Vec<&str> = result["segments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["type"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, ["intro", "recap"]);
    }
}
