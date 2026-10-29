use super::*;
use fluxa_ui::{
    DownloadStatus, NODE_DOWNLOAD_BACK, NODE_DOWNLOAD_BASE, NODE_DOWNLOAD_GROUP_BASE,
    NODE_DOWNLOAD_LOCATION, format_bytes, localized,
};
use web_time::{SystemTime, UNIX_EPOCH};

const NODE_SPAN: u64 = 4000;
const STORAGE_POLL: Duration = Duration::from_secs(5);

pub(crate) fn tick(state: &mut RendererState) {
    if state.route != Route::Library || !state.library_downloads {
        state.storage_polled = None;
        return;
    }
    if state
        .storage_polled
        .is_some_and(|at| at.elapsed() < STORAGE_POLL)
    {
        return;
    }
    state.storage_polled = Some(Instant::now());
    if let Some(session) = state.session.as_ref()
        && let Err(error) = session.dispatch(json!({"type": "offlineStorageRequested"}))
    {
        host_log(format!("storage request failed: {error}"));
    }
}

fn open_location_picker(state: &mut RendererState) {
    if let Some(picker) = state.folder_picker.as_ref() {
        if let Some(path) = picker() {
            crate::input::apply_choice(
                state,
                Route::Library,
                "downloadDir",
                path.to_string_lossy().into_owned(),
            );
        }
        return;
    }
    let language = state.settings.language().to_owned();
    let summary = &state.library.downloads_summary;
    let free = |bytes: u64| {
        localized("downloads.volume_free", &language).replace("%s", &format_bytes(bytes))
    };
    let mut options = Vec::new();
    for (index, volume) in summary.volumes.iter().enumerate() {
        let name = if index == 0 {
            localized("downloads.location_default", &language)
        } else {
            volume.label.clone()
        };
        let value = if index == 0 {
            String::new()
        } else {
            volume.path.clone()
        };
        options.push((value, format!("{name} · {}", free(volume.free_bytes))));
    }
    if options.is_empty() {
        options.push((
            String::new(),
            localized("downloads.location_default", &language),
        ));
    }
    let selected = state
        .settings
        .value("downloadDir")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    card_menu::open_choice(
        state,
        fluxa_ui::ChoiceRequest {
            key: "downloadDir",
            title: localized("downloads.location_title", &language),
            options,
            selected,
        },
    );
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

pub(crate) fn close_group(state: &mut RendererState) -> bool {
    if state.route != Route::Library
        || !state.library_downloads
        || state.library_download_group.is_none()
    {
        return false;
    }
    state.library_download_group = None;
    state.screen_scroll_offsets.remove(&Route::Library);
    refresh_library_view(state);
    true
}

fn open_group(state: &mut RendererState, index: usize) {
    state.library_download_group = state
        .library
        .downloads
        .get(index)
        .map(|group| group.id.clone());
    state.screen_scroll_offsets.remove(&Route::Library);
    refresh_library_view(state);
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if node == NODE_DOWNLOAD_BACK {
        close_group(state);
        return true;
    }
    if node >= NODE_DOWNLOAD_GROUP_BASE && node < NODE_DOWNLOAD_GROUP_BASE + NODE_SPAN {
        open_group(state, (node - NODE_DOWNLOAD_GROUP_BASE) as usize);
        return true;
    }
    if node == NODE_DOWNLOAD_LOCATION {
        open_location_picker(state);
        return true;
    }
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
            items
                .iter()
                .any(|item| matches!(item["status"].as_str(), Some("downloading" | "queued")))
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
        let snapshot =
            json!({"offline": {"items": [{"status": "completed"}, {"status": "downloading"}]}});
        assert!(any_active(&snapshot));
    }
}
