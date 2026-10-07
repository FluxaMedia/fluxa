use std::sync::mpsc::Receiver;

use serde_json::{Map, Value, json};

use super::{PlayerSession, overlay};
use crate::{RendererState, core_value, host_log, profile_language};

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Status {
    #[default]
    Idle,
    Loading,
    NoKey,
    Unavailable,
    Ready,
}

#[derive(Default)]
pub(super) struct XRay {
    pub(super) status: Status,
    key: Option<String>,
    ctx: Value,
    cast: Vec<Value>,
    marks: Map<String, Value>,
    rows: Value,
    rows_at: Option<(i64, u64)>,
    version: u64,
    find_rx: Option<Receiver<Option<Value>>>,
    credits_rx: Option<Receiver<Option<Value>>>,
    series_rx: Option<Receiver<Option<Value>>>,
    credits_done: Option<Option<Value>>,
    series_done: Option<Option<Value>>,
}

const RECORD_VERSION: i64 = 4;

impl XRay {
    pub(super) fn people(&self, group: &str) -> Vec<&Value> {
        self.rows
            .get(group)
            .and_then(Value::as_array)
            .map(|people| people.iter().collect())
            .unwrap_or_default()
    }
}

fn storage_key(ctx: &Value) -> Option<String> {
    let id = match (ctx["imdbId"].as_str(), ctx["tmdbId"].as_i64()) {
        (Some(imdb), _) => imdb.to_owned(),
        (None, Some(tmdb)) => format!("tmdb{tmdb}"),
        _ => return None,
    };
    let part = |key: &str| ctx[key].as_i64().unwrap_or(0);
    Some(format!("xray:{id}:{}:{}", part("season"), part("episode")))
}

fn request_credits(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let api_key = state
        .settings
        .str_value("tmdbApiKey")
        .unwrap_or_default()
        .trim()
        .to_owned();
    if api_key.is_empty() {
        player.xray.status = Status::NoKey;
        return;
    }
    let mut args = player.xray.ctx.clone();
    args["apiKey"] = json!(api_key);
    args["language"] = json!(profile_language(
        session
            .snapshot()
            .pointer("/profile/active")
            .unwrap_or(&Value::Null)
    ));
    let plan = core_value("playerXrayCreditsPlan", args);
    let executor = session.executor();
    let url_of = |key: &str| plan.as_ref()?.get(key)?.as_str().map(str::to_owned);
    if let Some(url) = url_of("url") {
        player.xray.credits_rx = Some(executor.fetch_json(url));
        player.xray.series_rx = url_of("seriesUrl").map(|url| executor.fetch_json(url));
        player.xray.status = Status::Loading;
    } else if let Some(url) = url_of("findUrl") {
        player.xray.find_rx = Some(executor.fetch_json(url));
        player.xray.status = Status::Loading;
    } else {
        player.xray.status = Status::Unavailable;
    }
}

fn episode_from_id(player: &PlayerSession, snapshot: &Value) -> Option<(i64, i64)> {
    let id = snapshot
        .pointer("/player/currentVideoId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .or_else(|| player.meta.get("lastVideoId").and_then(Value::as_str))?;
    let mut parts = id.rsplit(':');
    let episode = parts.next()?.parse().ok()?;
    let season = parts.next()?.parse().ok()?;
    Some((season, episode))
}

pub(super) fn load(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let snapshot = session.snapshot();
    let Some(mut ctx) = overlay::lookup_context(player, &snapshot) else {
        player.xray.status = Status::Unavailable;
        return;
    };
    let numbers = player
        .season_episode
        .or_else(|| episode_from_id(player, &snapshot));
    if let (Some((season, episode)), "tv") =
        (numbers, ctx["mediaType"].as_str().unwrap_or_default())
    {
        ctx["season"] = json!(season);
        ctx["episode"] = json!(episode);
    }
    let Some(key) = storage_key(&ctx) else {
        player.xray.status = Status::Unavailable;
        return;
    };
    let current = player.xray.key.as_deref() == Some(key.as_str());
    if current && matches!(player.xray.status, Status::Ready | Status::Loading) {
        return;
    }
    let stored = session.storage().read_json(&key).ok().flatten();
    player.xray = XRay {
        key: Some(key),
        ctx,
        ..Default::default()
    };
    if let Some(stored) = stored {
        if stored["v"].as_i64() == Some(RECORD_VERSION) {
            player.xray.cast = stored["cast"].as_array().cloned().unwrap_or_default();
        }
        player.xray.marks = stored["marks"].as_object().cloned().unwrap_or_default();
    }
    if player.xray.cast.is_empty() {
        request_credits(state);
    } else {
        player.xray.status = Status::Ready;
    }
}

fn save(session: &crate::SessionHandle, xray: &XRay) {
    let Some(key) = xray.key.as_deref() else {
        return;
    };
    let value = json!({ "v": RECORD_VERSION, "cast": xray.cast, "marks": xray.marks });
    if let Err(error) = session.storage().write_json(key, &value) {
        host_log(format!("X-Ray write failed: {error}"));
    }
}

pub(super) fn pump(state: &mut RendererState) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let xray = &mut player.xray;
    let found = xray.find_rx.as_ref().and_then(|rx| rx.try_recv().ok());
    if let Some(find) = found {
        xray.find_rx = None;
        let id = find.and_then(|find| {
            core_value(
                "playerXrayFindId",
                json!({ "find": find, "mediaType": xray.ctx["mediaType"] }),
            )
        });
        match id.and_then(|id| id["tmdbId"].as_i64()) {
            Some(id) => {
                xray.ctx["tmdbId"] = json!(id);
                request_credits(state);
            }
            None => xray.status = Status::Unavailable,
        }
        return;
    }
    if let Some(credits) = xray.credits_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        xray.credits_rx = None;
        xray.credits_done = Some(credits);
    }
    if let Some(series) = xray.series_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        xray.series_rx = None;
        xray.series_done = Some(series);
    }
    if xray.credits_rx.is_none() && xray.series_rx.is_none() && xray.credits_done.is_some() {
        let credits = xray.credits_done.take().flatten();
        let series = xray.series_done.take().flatten();
        let cast = core_value(
            "playerXrayCast",
            json!({ "credits": credits, "series": series }),
        )
        .and_then(|value| value["cast"].as_array().cloned())
        .filter(|cast| !cast.is_empty());
        match cast {
            Some(cast) => {
                xray.cast = cast;
                xray.status = Status::Ready;
                if let Some(session) = state.session.as_ref()
                    && let Some(player) = state.player.as_ref()
                {
                    save(session, &player.xray);
                }
            }
            None => xray.status = Status::Unavailable,
        }
        return;
    }
    refresh(player);
}

fn refresh(player: &mut PlayerSession) {
    if player.xray.status != Status::Ready {
        return;
    }
    let position_ms = (player.status.position * 1000.0) as i64;
    let at = (position_ms / 1000, player.xray.version);
    if player.xray.rows_at == Some(at) {
        return;
    }
    player.xray.rows_at = Some(at);
    player.xray.rows = core_value(
        "playerXrayRows",
        json!({
            "cast": player.xray.cast,
            "marks": player.xray.marks,
            "positionMs": position_ms,
        }),
    )
    .unwrap_or(Value::Null);
}

pub(super) fn toggle(state: &mut RendererState, person_id: i64) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let position_ms = (player.status.position * 1000.0) as i64;
    let updated = core_value(
        "playerXrayToggle",
        json!({
            "marks": player.xray.marks,
            "personId": person_id,
            "positionMs": position_ms,
        }),
    );
    let Some(marks) = updated.and_then(|value| value["marks"].as_object().cloned()) else {
        return;
    };
    player.xray.marks = marks;
    player.xray.version += 1;
    refresh(player);
    if let Some(session) = state.session.as_ref() {
        save(session, &player.xray);
    }
}
