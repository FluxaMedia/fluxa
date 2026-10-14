use super::*;
use fluxa_ui::{DownloadStatus, NODE_DOWNLOAD_BASE};
use std::time::{SystemTime, UNIX_EPOCH};

const NODE_SPAN: u64 = 4000;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if !(NODE_DOWNLOAD_BASE..NODE_DOWNLOAD_BASE + NODE_SPAN).contains(&node) {
        return false;
    }
    let offset = node - NODE_DOWNLOAD_BASE;
    let Some(row) = state.library.download_rows().nth((offset / 2) as usize) else {
        return true;
    };
    let action = if offset % 2 == 1 {
        json!({"type": "offlineDownloadRemoveRequested", "id": row.id})
    } else if matches!(
        row.status,
        DownloadStatus::Downloading | DownloadStatus::Queued
    ) {
        json!({"type": "offlineDownloadPauseRequested", "id": row.id})
    } else {
        json!({"type": "offlineDownloadResumeRequested", "id": row.id, "nowMs": now_ms()})
    };
    if let Some(session) = state.session.as_ref()
        && let Err(error) = session.dispatch(action)
    {
        host_log(format!("download action failed: {error}"));
    }
    true
}

pub(crate) fn request(state: &RendererState, meta: Value, stream: Value, video: Option<Value>) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let video_id = video
        .as_ref()
        .and_then(|video| video.get("id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let profile = session.active_profile();
    let action = json!({
        "type": "offlineDownloadRequested",
        "meta": meta,
        "stream": stream,
        "videoId": video_id,
        "video": video,
        "profileId": profile.get("id"),
        "language": profile_language(&profile),
        "nowMs": now_ms(),
    });
    if let Err(error) = session.dispatch(action) {
        host_log(format!("download request failed: {error}"));
    }
}

fn any_active(snapshot: &Value) -> bool {
    snapshot
        .pointer("/offline/items")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|item| {
                matches!(item["status"].as_str(), Some("downloading" | "queued"))
            })
        })
}

pub(crate) fn active(state: &RendererState) -> bool {
    state
        .session
        .as_ref()
        .is_some_and(|session| any_active(&session.snapshot()))
}

pub(crate) fn request_close(state: &mut RendererState) -> bool {
    if !active(state) || crate::card_menu::quit_downloads_open(state) {
        return true;
    }
    crate::card_menu::open_quit_downloads(state);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_and_finished_downloads_do_not_block_quitting() {
        let snapshot = json!({"offline": {"items": [
            {"status": "paused"}, {"status": "completed"}, {"status": "failed"}
        ]}});
        assert!(!any_active(&snapshot));
    }

    #[test]
    fn a_running_download_asks_before_quitting() {
        let snapshot = json!({"offline": {"items": [{"status": "completed"}, {"status": "downloading"}]}});
        assert!(any_active(&snapshot));
    }
}
