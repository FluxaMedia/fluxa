use super::*;

pub(super) fn command_starts_playback(snapshot: &Value) -> bool {
    snapshot
        .pointer("/player/currentStreams")
        .and_then(Value::as_array)
        .is_some_and(|streams| !streams.is_empty())
}

pub(super) fn resolution_command(snapshot: &Value, player: &mut PlayerSession) -> Option<Value> {
    let meta = &player.meta;
    if let Some(streams) = snapshot
        .pointer("/player/currentStreams")
        .and_then(Value::as_array)
        .filter(|streams| !streams.is_empty())
    {
        let index = snapshot
            .pointer("/player/currentStreamIndex")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0) as usize;
        let stream = streams.get(index).cloned().unwrap_or(Value::Null);
        let url = snapshot
            .pointer("/player/currentUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| playback_url(&stream, meta))?;
        return Some(json!({
            "type": "playerResolvePlaybackRequested",
            "url": url,
            "stream": stream,
            "currentVideoId": snapshot.pointer("/player/currentVideoId").cloned().unwrap_or(Value::Null),
            "title": player.title(),
        }));
    }
    if snapshot
        .pointer("/player/pendingStreamLoad")
        .is_some_and(Value::is_object)
    {
        return None;
    }
    let streams = snapshot
        .pointer("/player/directPlaybackTarget/streams")
        .and_then(Value::as_array)?;
    if streams.is_empty() {
        player.error = Some("No streams were found for this title".to_owned());
        return None;
    }
    let Some(content_id) = meta.get("id").and_then(Value::as_str) else {
        player.error = Some("This title has no content id".to_owned());
        return None;
    };
    let video_id = meta
        .get("lastVideoId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .unwrap_or(content_id);
    let profile = snapshot
        .pointer("/profile/active")
        .cloned()
        .unwrap_or(Value::Null);
    let mode = profile
        .get("streamSourceSelectionMode")
        .and_then(Value::as_str)
        .unwrap_or("manual");
    if mode == "manual" && player.chosen.is_none() {
        if player.sources.is_none() {
            player.sources = Some(streams.clone());
        }
        return None;
    }
    let (initial_index, saved_url) = match player.chosen {
        Some(index) => (json!(index), Value::Null),
        None => (
            meta.get("lastStreamIndex").cloned().unwrap_or(json!(0)),
            meta.get("lastStreamUrl").cloned().unwrap_or(Value::Null),
        ),
    };
    Some(json!({
        "type": "playerLoadStreamsRequested",
        "contentType": meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
        "id": video_id,
        "currentVideoId": video_id,
        "initialVideoId": video_id,
        "initialStreams": streams,
        "initialStreamIndex": initial_index,
        "savedUrl": saved_url,
        "savedTitle": meta.get("lastStreamTitle").cloned().unwrap_or(Value::Null),
        "sourceSelectionMode": mode,
        "regexPattern": profile.get("streamSourceRegexPattern"),
        "title": meta.get("name").or_else(|| meta.get("title")),
        "originalName": meta.get("originalName"),
        "year": meta_year(meta),
        "language": profile_language(&profile),
        "profile": profile,
    }))
}

pub(super) fn playback_url(stream: &Value, meta: &Value) -> Option<String> {
    let plan = core_value(
        "playbackPreparePlan",
        json!({"stream": stream, "meta": meta}),
    )?;
    let mode = plan.get("mode").and_then(Value::as_str)?;
    matches!(mode, "direct" | "torrent" | "external")
        .then(|| plan.get("url").and_then(Value::as_str))
        .flatten()
        .filter(|url| !url.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn poll_torrent(
    player: &mut PlayerSession,
    session: &fluxa_effects::SessionHandle,
    snapshot: &Value,
) {
    while let Some(status) = player.torrent_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        player.torrent_status = Some(status);
    }
    let stream = snapshot
        .pointer("/player/currentStreamIndex")
        .and_then(Value::as_u64)
        .and_then(|index| {
            snapshot
                .pointer("/player/currentStreams")?
                .as_array()?
                .get(index as usize)
        });
    let Some(link) =
        stream.and_then(|stream| stream_policy::stream_magnet_link_json(&stream.to_string()))
    else {
        return;
    };
    if player.torrent_link.as_deref() == Some(link.as_str()) {
        return;
    }
    let file_id = stream
        .and_then(|stream| stream.get("fileIdx").or_else(|| stream.get("fileIndex")))
        .and_then(Value::as_u64)
        .map(|index| index as usize);
    player.torrent_link = Some(link.clone());
    player.torrent_status = None;
    player.torrent_rx = Some(session.poll_torrent_status(link, file_id));
}

fn meta_year(meta: &Value) -> Value {
    let year = meta.get("year");
    let parsed = year
        .and_then(Value::as_i64)
        .or_else(|| year.and_then(Value::as_str)?.get(..4)?.parse().ok());
    parsed.map_or(Value::Null, |year| json!(year))
}
