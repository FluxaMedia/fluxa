use super::*;

pub(super) fn parse_range(
    value: Option<&HeaderValue>,
    length: u64,
) -> Result<Option<(u64, u64)>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    let raw = value.to_str().map_err(|_| ())?;
    let spec = raw.strip_prefix("bytes=").ok_or(())?;
    if spec.contains(',') || length == 0 {
        return Err(());
    }
    let (start, end) = spec.split_once('-').ok_or(())?;
    if start.is_empty() {
        let suffix_len = end.parse::<u64>().map_err(|_| ())?;
        if suffix_len == 0 {
            return Err(());
        }
        let start = length.saturating_sub(suffix_len);
        return Ok(Some((start, length.saturating_sub(1))));
    }
    let start = start.parse::<u64>().map_err(|_| ())?;
    if start >= length {
        return Err(());
    }
    let end = if end.is_empty() {
        length.saturating_sub(1)
    } else {
        let end = end.parse::<u64>().map_err(|_| ())?;
        if end < start {
            return Err(());
        }
        end.min(length.saturating_sub(1))
    };
    Ok(Some((start, end)))
}

pub(super) fn insert_header(headers: &mut HeaderMap, key: &'static str, value: String) {
    if let Ok(value) = HeaderValue::from_str(&value) {
        headers.insert(key, value);
    }
}

pub(super) fn largest_file_id(details: &TorrentDetailsResponse) -> Option<usize> {
    details
        .files
        .as_ref()?
        .iter()
        .enumerate()
        .max_by_key(|(_, file)| file.length)
        .map(|(idx, _)| idx)
}

pub(super) async fn prioritize_stream_file(
    state: &EngineState,
    torrent_id: usize,
    file_id: usize,
    role: FileRole,
) {
    // Keep one current video file plus active auxiliary files. Moving through
    // a season pack replaces the old video instead of accumulating episodes;
    // subtitles remain selectable without evicting that primary video.
    let (only_files, previous_primary) = state
        .runtime
        .lock()
        .map(|mut runtime| {
            let files = &mut runtime.prioritized_files;
            let focus = files.entry(torrent_id).or_default();
            let previous = focus.primary_video;
            let selected = update_file_focus(focus, file_id, role);
            let replaced_primary = matches!(role, FileRole::Video)
                .then_some(previous)
                .flatten()
                .filter(|previous| *previous != file_id);
            (selected, replaced_primary)
        })
        .unwrap_or((None, None));
    if let Some(previous_file) = previous_primary {
        let _ = state
            .api
            .api_clear_streaming_window(TorrentIdOrHash::Id(torrent_id), previous_file);
        if let Ok(mut runtime) = state.runtime.lock() {
            runtime
                .playback_windows
                .remove(&(torrent_id, previous_file));
            if let Some(session) = runtime
                .playback_sessions
                .remove(&(torrent_id, previous_file))
            {
                session.cancel.cancel();
            }
        }
    }
    let Some(only_files) = only_files else {
        return;
    };
    let _ = state
        .api
        .api_torrent_action_update_only_files(TorrentIdOrHash::Id(torrent_id), &only_files)
        .await;
}

pub(super) fn update_file_focus(
    focus: &mut TorrentFileFocus,
    file_id: usize,
    role: FileRole,
) -> Option<HashSet<usize>> {
    match role {
        FileRole::Video if focus.primary_video == Some(file_id) => return None,
        FileRole::Video => {
            // A new episode starts a new playback focus. Its prior subtitles
            // must not keep consuming piece-picker attention.
            focus.primary_video = Some(file_id);
            focus.auxiliary_files.clear();
        }
        FileRole::Subtitle | FileRole::Auxiliary if !focus.auxiliary_files.insert(file_id) => {
            return None;
        }
        FileRole::Subtitle | FileRole::Auxiliary => {}
    }
    let mut selected = focus.auxiliary_files.clone();
    if let Some(primary) = focus.primary_video {
        selected.insert(primary);
    }
    Some(selected)
}

pub(super) fn lookup_known_link(state: &EngineState, link: Option<&str>) -> Option<usize> {
    let link = link?.trim();
    state.runtime.lock().ok()?.known_links.get(link).copied()
}

pub(super) fn remember_link(state: &EngineState, link: &str, id: usize) {
    if let Ok(mut runtime) = state.runtime.lock() {
        let links = &mut runtime.known_links;
        if links.len() >= 64 {
            links.clear();
        }
        links.insert(link.to_string(), id);
    }
}

pub(super) fn error_response(message_status: StatusCode, message: impl Into<String>) -> Response {
    (message_status, Json(json!({ "error": message.into() }))).into_response()
}

pub(super) fn request_authorized(
    state: &EngineState,
    remote_addr: SocketAddr,
    access_token: Option<&str>,
) -> bool {
    remote_addr.ip().is_loopback()
        || (!state.access_token.is_empty()
            && access_token.is_some_and(|token| token == state.access_token.as_str()))
}

pub(super) fn range_not_satisfiable_response(length: u64) -> Response {
    let mut headers = HeaderMap::new();
    insert_header(&mut headers, "Content-Range", format!("bytes */{length}"));
    (
        StatusCode::RANGE_NOT_SATISFIABLE,
        headers,
        Json(json!({ "error": "range not satisfiable" })),
    )
        .into_response()
}
