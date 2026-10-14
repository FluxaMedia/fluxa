use crate::headless_engine::helpers::normalize_error;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const SPEED_SMOOTHING: f64 = 0.3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DownloadStatus {
    #[default]
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct DownloadItem {
    id: String,
    meta_id: String,
    content_type: String,
    name: String,
    poster: Option<String>,
    background: Option<String>,
    logo: Option<String>,
    video_id: Option<String>,
    season: Option<u32>,
    episode: Option<u32>,
    episode_title: Option<String>,
    thumbnail: Option<String>,
    stream: Value,
    url: String,
    profile_id: Option<String>,
    status: DownloadStatus,
    bytes_done: u64,
    bytes_total: Option<u64>,
    percent: f64,
    speed_bps: f64,
    eta_ms: Option<u64>,
    elapsed_ms: u64,
    last_tick_ms: u64,
    path: Option<String>,
    error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OfflineState {
    pub(crate) items: Vec<DownloadItem>,
    last_request: Value,
    last_enqueued: Value,
    error: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnqueueOfflineDownloadPayload {
    id: String,
    url: String,
    stream: Value,
    meta: Value,
    video_id: Option<String>,
    video: Option<Value>,
    subtitle: Option<Value>,
    profile_id: Option<String>,
    language: Option<String>,
}

fn text(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn number(value: &Value, key: &str) -> Option<u32> {
    value
        .get(key)
        .and_then(|number| {
            number
                .as_u64()
                .or_else(|| number.as_str().and_then(|text| text.parse().ok()))
        })
        .and_then(|number| u32::try_from(number).ok())
}

fn is_http(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn item_mut<'a>(engine: &'a mut HeadlessEngine, id: &str) -> Option<&'a mut DownloadItem> {
    engine
        .state
        .offline
        .items
        .iter_mut()
        .find(|item| item.id == id)
}

fn start_effect(engine: &mut HeadlessEngine, item: &DownloadItem) -> EffectEnvelope {
    let generation = engine.bump_generation(GenerationKey::Offline);
    let payload = EnqueueOfflineDownloadPayload {
        id: item.id.clone(),
        url: item.url.clone(),
        stream: item.stream.clone(),
        meta: json!({"id": item.meta_id, "name": item.name}),
        video_id: item.video_id.clone(),
        video: None,
        subtitle: None,
        profile_id: item.profile_id.clone(),
        language: None,
    };
    engine.effect(EffectKind::EnqueueOfflineDownload, generation, payload)
}

fn control_effect(engine: &mut HeadlessEngine, id: &str, command: &str) -> EffectEnvelope {
    let generation = engine.bump_generation(GenerationKey::Offline);
    engine.effect(
        EffectKind::ControlOfflineDownload,
        generation,
        json!({"id": id, "command": command}),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch(
    engine: &mut HeadlessEngine,
    meta: Value,
    stream: Value,
    video_id: Option<String>,
    video: Option<Value>,
    subtitle: Option<Value>,
    profile_id: Option<String>,
    language: Option<String>,
    now_ms: u64,
) -> Vec<EffectEnvelope> {
    let url = text(&stream, &["url"]).unwrap_or_default();
    let meta_id = text(&meta, &["id"]).unwrap_or_default();
    let id = format!("{meta_id}|{}", video_id.clone().unwrap_or_default());

    engine.state.offline.last_request = json!({
        "meta": meta,
        "stream": stream,
        "videoId": video_id,
        "video": video,
        "subtitle": subtitle,
        "profileId": profile_id,
        "language": language,
    });

    if let Some(existing) = engine.state.offline.items.iter().find(|item| item.id == id) {
        match existing.status {
            DownloadStatus::Paused | DownloadStatus::Failed => return resume(engine, &id, now_ms),
            _ => return vec![],
        }
    }

    let video_ref = video.clone().unwrap_or(Value::Null);
    let mut item = DownloadItem {
        id: id.clone(),
        meta_id,
        content_type: text(&meta, &["type"]).unwrap_or_default(),
        name: text(&meta, &["name", "title"]).unwrap_or_default(),
        poster: text(&meta, &["poster"]),
        background: text(&meta, &["background"]),
        logo: text(&meta, &["logo"]),
        video_id,
        season: number(&video_ref, "season"),
        episode: number(&video_ref, "episode").or_else(|| number(&video_ref, "number")),
        episode_title: text(&video_ref, &["title", "name"]),
        thumbnail: text(&video_ref, &["thumbnail"]),
        stream,
        url,
        profile_id,
        status: DownloadStatus::Downloading,
        last_tick_ms: now_ms,
        ..Default::default()
    };

    if !is_http(&item.url) {
        item.status = DownloadStatus::Failed;
        item.error = Some("unsupported_source".to_owned());
        engine.state.offline.items.push(item);
        return vec![];
    }

    let effect = start_effect(engine, &item);
    engine.state.offline.items.push(item);
    vec![effect]
}

pub(crate) fn resume(engine: &mut HeadlessEngine, id: &str, now_ms: u64) -> Vec<EffectEnvelope> {
    let Some(item) = item_mut(engine, id) else {
        return vec![];
    };
    if !matches!(item.status, DownloadStatus::Paused | DownloadStatus::Failed) {
        return vec![];
    }
    if !is_http(&item.url) {
        return vec![];
    }
    item.status = DownloadStatus::Downloading;
    item.error = None;
    item.last_tick_ms = now_ms;
    item.speed_bps = 0.0;
    let item = item.clone();
    vec![start_effect(engine, &item)]
}

pub(crate) fn pause(engine: &mut HeadlessEngine, id: &str) -> Vec<EffectEnvelope> {
    let Some(item) = item_mut(engine, id) else {
        return vec![];
    };
    if !matches!(
        item.status,
        DownloadStatus::Downloading | DownloadStatus::Queued
    ) {
        return vec![];
    }
    item.status = DownloadStatus::Paused;
    item.speed_bps = 0.0;
    item.eta_ms = None;
    item.last_tick_ms = 0;
    vec![control_effect(engine, id, "pause")]
}

pub(crate) fn remove(engine: &mut HeadlessEngine, id: &str) -> Vec<EffectEnvelope> {
    let before = engine.state.offline.items.len();
    engine.state.offline.items.retain(|item| item.id != id);
    if engine.state.offline.items.len() == before {
        return vec![];
    }
    vec![control_effect(engine, id, "remove")]
}

pub(crate) fn progress(
    engine: &mut HeadlessEngine,
    id: &str,
    bytes_done: u64,
    bytes_total: Option<u64>,
    now_ms: u64,
) {
    let Some(item) = item_mut(engine, id) else {
        return;
    };
    if item.status != DownloadStatus::Downloading {
        return;
    }
    if item.last_tick_ms > 0 && now_ms > item.last_tick_ms {
        let elapsed = now_ms - item.last_tick_ms;
        item.elapsed_ms += elapsed;
        let instant = bytes_done.saturating_sub(item.bytes_done) as f64 * 1000.0 / elapsed as f64;
        item.speed_bps = if item.speed_bps <= 0.0 {
            instant
        } else {
            item.speed_bps * (1.0 - SPEED_SMOOTHING) + instant * SPEED_SMOOTHING
        };
    }
    item.last_tick_ms = now_ms;
    item.bytes_done = bytes_done;
    if bytes_total.is_some() {
        item.bytes_total = bytes_total;
    }
    match item.bytes_total.filter(|total| *total > 0) {
        Some(total) => {
            item.percent = (bytes_done as f64 / total as f64 * 100.0).min(100.0);
            item.eta_ms = (item.speed_bps > 1.0).then(|| {
                (total.saturating_sub(bytes_done) as f64 / item.speed_bps * 1000.0) as u64
            });
        }
        None => {
            item.percent = 0.0;
            item.eta_ms = None;
        }
    }
}

pub(crate) fn completed(
    engine: &mut HeadlessEngine,
    id: &str,
    path: String,
    bytes_total: Option<u64>,
    now_ms: u64,
) {
    let Some(item) = item_mut(engine, id) else {
        return;
    };
    if item.last_tick_ms > 0 && now_ms > item.last_tick_ms {
        item.elapsed_ms += now_ms - item.last_tick_ms;
    }
    let total = bytes_total.or(item.bytes_total).unwrap_or(item.bytes_done);
    item.status = DownloadStatus::Completed;
    item.bytes_done = total;
    item.bytes_total = Some(total);
    item.percent = 100.0;
    item.speed_bps = 0.0;
    item.eta_ms = Some(0);
    item.last_tick_ms = 0;
    item.path = Some(path);
    item.error = None;
}

pub(crate) fn failed(engine: &mut HeadlessEngine, id: &str, error: String) {
    let Some(item) = item_mut(engine, id) else {
        return;
    };
    if item.status != DownloadStatus::Downloading {
        return;
    }
    item.status = DownloadStatus::Failed;
    item.speed_bps = 0.0;
    item.eta_ms = None;
    item.last_tick_ms = 0;
    item.error = Some(error);
}

pub(crate) fn restored(engine: &mut HeadlessEngine, items: Vec<Value>) {
    let mut items = items
        .into_iter()
        .filter_map(|item| serde_json::from_value::<DownloadItem>(item).ok())
        .collect::<Vec<_>>();
    for item in &mut items {
        if matches!(
            item.status,
            DownloadStatus::Downloading | DownloadStatus::Queued
        ) {
            item.status = DownloadStatus::Paused;
        }
        item.speed_bps = 0.0;
        item.eta_ms = None;
        item.last_tick_ms = 0;
    }
    engine.state.offline.items = items;
}

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation == engine.state.runtime.get(GenerationKey::Offline) {
        if result.status.is_ok() {
            engine.state.offline.last_enqueued = result.value.clone();
            engine.state.offline.error = Value::Null;
        } else {
            engine.state.offline.error = normalize_error(result.error.clone());
        }
    }
    vec![]
}
