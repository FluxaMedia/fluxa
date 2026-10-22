use super::super::*;
use crate::runtime::EffectKind;
use serde_json::{Value, json};

fn act(engine: &mut HeadlessEngine, action: Value) -> Vec<crate::runtime::EffectEnvelope> {
    let action: AppAction = serde_json::from_value(action).unwrap();
    engine.apply(action).effects
}

fn request(engine: &mut HeadlessEngine, episode: u32) -> Vec<crate::runtime::EffectEnvelope> {
    act(
        engine,
        json!({
            "type": "offlineDownloadRequested",
            "meta": {"id": "tt1", "type": "series", "name": "Show", "logo": "https://x.test/logo.png", "background": "https://x.test/bg.jpg"},
            "stream": {"url": "https://cdn.test/a.mkv"},
            "videoId": format!("tt1:1:{episode}"),
            "video": {"season": 1, "episode": episode, "title": "Pilot"},
            "nowMs": 1000,
        }),
    )
}

fn item(engine: &HeadlessEngine, index: usize) -> Value {
    serde_json::to_value(&engine.state.offline.items[index]).unwrap()
}

#[test]
fn request_creates_item_with_series_artwork_and_starts_effect() {
    let mut engine = HeadlessEngine::new(Default::default());
    let effects = request(&mut engine, 1);
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].kind, EffectKind::EnqueueOfflineDownload);
    let item = item(&engine, 0);
    assert_eq!(item["status"], "downloading");
    assert_eq!(item["season"], 1);
    assert_eq!(item["episode"], 1);
    assert_eq!(item["logo"], "https://x.test/logo.png");
}

#[test]
fn requesting_the_same_episode_twice_does_not_restart_it() {
    let mut engine = HeadlessEngine::new(Default::default());
    request(&mut engine, 1);
    assert!(request(&mut engine, 1).is_empty());
    assert_eq!(engine.state.offline.items.len(), 1);
}

#[test]
fn progress_derives_percent_speed_and_eta() {
    let mut engine = HeadlessEngine::new(Default::default());
    request(&mut engine, 1);
    act(
        &mut engine,
        json!({"type": "offlineDownloadProgressed", "id": "tt1|tt1:1:1", "bytesDone": 1_000_000, "bytesTotal": 10_000_000, "nowMs": 2000}),
    );
    let item = item(&engine, 0);
    assert_eq!(item["percent"], 10.0);
    assert_eq!(item["speedBps"], 1_000_000.0);
    assert_eq!(item["etaMs"], 9000);
    assert_eq!(item["elapsedMs"], 1000);
}

#[test]
fn non_http_stream_fails_without_an_effect() {
    let mut engine = HeadlessEngine::new(Default::default());
    let effects = act(
        &mut engine,
        json!({
            "type": "offlineDownloadRequested",
            "meta": {"id": "tt2", "type": "movie", "name": "Film"},
            "stream": {"externalUrl": "https://example.test/watch"},
        }),
    );
    assert!(effects.is_empty());
    let item = item(&engine, 0);
    assert_eq!(item["status"], "failed");
    assert_eq!(item["error"], "unsupported_source");
}

#[test]
fn pause_stops_the_task_and_ignores_late_progress() {
    let mut engine = HeadlessEngine::new(Default::default());
    request(&mut engine, 1);
    let effects = act(
        &mut engine,
        json!({"type": "offlineDownloadPauseRequested", "id": "tt1|tt1:1:1"}),
    );
    assert_eq!(effects[0].kind, EffectKind::ControlOfflineDownload);
    assert_eq!(effects[0].payload["command"], "pause");
    act(
        &mut engine,
        json!({"type": "offlineDownloadProgressed", "id": "tt1|tt1:1:1", "bytesDone": 5, "nowMs": 3000}),
    );
    assert_eq!(item(&engine, 0)["bytesDone"], 0);
    let resumed = act(
        &mut engine,
        json!({"type": "offlineDownloadResumeRequested", "id": "tt1|tt1:1:1", "nowMs": 4000}),
    );
    assert_eq!(resumed[0].kind, EffectKind::EnqueueOfflineDownload);
    assert_eq!(item(&engine, 0)["status"], "downloading");
}

#[test]
fn restored_downloads_come_back_paused() {
    let mut engine = HeadlessEngine::new(Default::default());
    act(
        &mut engine,
        json!({"type": "offlineDownloadsRestored", "items": [
            {"id": "a", "status": "downloading", "speedBps": 5.0, "url": "https://x.test/a"},
            {"id": "b", "status": "completed", "path": "/d/b.mkv"}
        ]}),
    );
    assert_eq!(item(&engine, 0)["status"], "paused");
    assert_eq!(item(&engine, 0)["speedBps"], 0.0);
    assert_eq!(item(&engine, 1)["status"], "completed");
}

#[test]
fn remove_drops_the_item_and_asks_the_shell_to_delete_files() {
    let mut engine = HeadlessEngine::new(Default::default());
    request(&mut engine, 1);
    let effects = act(
        &mut engine,
        json!({"type": "offlineDownloadRemoveRequested", "id": "tt1|tt1:1:1"}),
    );
    assert_eq!(effects[0].payload["command"], "remove");
    assert!(engine.state.offline.items.is_empty());
}

#[test]
fn total_speed_is_sampled_once_per_second_and_drops_when_paused() {
    let mut engine = HeadlessEngine::new(Default::default());
    request(&mut engine, 1);
    request(&mut engine, 2);
    for (now, done) in [
        (2000u64, 1_000_000u64),
        (2400, 1_400_000),
        (3100, 2_100_000),
    ] {
        act(
            &mut engine,
            json!({"type": "offlineDownloadProgressed", "id": "tt1|tt1:1:1", "bytesDone": done, "nowMs": now}),
        );
    }
    let state = serde_json::to_value(&*engine.state.offline).unwrap();
    assert_eq!(state["history"].as_array().unwrap().len(), 2);
    assert_eq!(state["totalSpeedBps"], 1_000_000.0);
    act(
        &mut engine,
        json!({"type": "offlineDownloadPauseRequested", "id": "tt1|tt1:1:1"}),
    );
    let state = serde_json::to_value(&*engine.state.offline).unwrap();
    assert_eq!(state["totalSpeedBps"], 0.0);
}

#[test]
fn download_dir_setting_is_captured_per_item_and_sent_on_remove() {
    let mut engine = HeadlessEngine::new(Default::default());
    engine.state.settings.values["downloadDir"] = json!("/mnt/media");
    let effects = request(&mut engine, 1);
    assert_eq!(effects[0].payload["dir"], "/mnt/media");
    let effects = act(
        &mut engine,
        json!({"type": "offlineDownloadRemoveRequested", "id": "tt1|tt1:1:1"}),
    );
    assert_eq!(effects[0].payload["dir"], "/mnt/media");
}

#[test]
fn storage_report_exposes_location_and_volumes() {
    let mut engine = HeadlessEngine::new(Default::default());
    let effects = act(&mut engine, json!({"type": "offlineStorageRequested"}));
    assert_eq!(effects[0].kind, EffectKind::ReadDownloadStorage);
    let result: EffectResultInput = serde_json::from_value(json!({
        "effectId": effects[0].id, "status": "ok",
        "value": {
            "location": {"path": "/d", "label": "Default", "freeBytes": 5, "totalBytes": 10},
            "volumes": [{"path": "/d", "label": "Default", "freeBytes": 5, "totalBytes": 10}]
        }
    }))
    .unwrap();
    engine.complete(result);
    let state = serde_json::to_value(&*engine.state.offline).unwrap();
    assert_eq!(state["location"]["freeBytes"], 5);
    assert_eq!(state["volumes"].as_array().unwrap().len(), 1);
}

#[test]
fn torrent_stream_is_enqueued_with_its_torrent_url() {
    let mut engine = HeadlessEngine::new(Default::default());
    let effects = act(
        &mut engine,
        json!({
            "type": "offlineDownloadRequested",
            "meta": {"id": "tt1", "type": "movie", "name": "Film"},
            "stream": {"infoHash": "abc123", "fileIdx": 2},
            "nowMs": 1000,
        }),
    );
    assert_eq!(effects[0].kind, EffectKind::EnqueueOfflineDownload);
    assert_eq!(effects[0].payload["url"], "stremio://torrent/abc123/2");
}
