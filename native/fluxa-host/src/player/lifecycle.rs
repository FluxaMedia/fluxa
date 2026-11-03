use super::*;

pub(super) fn log_status_change(old: &VideoStatus, new: &VideoStatus) {
    if new.has_frame && !old.has_frame {
        host_log(format!("player first frame, duration={:.1}s", new.duration));
    }
    if new.paused != old.paused {
        host_log(format!(
            "player {} at {:.1}s",
            if new.paused { "paused" } else { "resumed" },
            new.position
        ));
    }
    if new.stalled != old.stalled {
        host_log(format!(
            "player {} at {:.1}s, buffered to {:.1}s",
            if new.stalled { "stalled" } else { "recovered" },
            new.position,
            new.buffered
        ));
    }
    if new.error.is_some() && new.error != old.error {
        host_log(format!(
            "player error at {:.1}s: {}",
            new.position,
            new.error.as_deref().unwrap_or_default()
        ));
    }
}

pub(crate) fn leave(state: &mut RendererState) {
    let desktop = state.home.form_factor == fluxa_ui::UiFormFactorJson::Desktop;
    if desktop
        && state
            .player
            .as_ref()
            .is_some_and(|player| player.can_minimize())
    {
        minimize(state);
    } else {
        close(state);
    }
}

pub(crate) fn close(state: &mut RendererState) {
    let Some(player) = state.player.take() else {
        return;
    };
    host_log(format!(
        "player closed at {:.1}s of {:.1}s",
        player.status.position, player.status.duration
    ));
    remember_summary(state, &player);
    if let Some(session) = state.session.as_ref() {
        let snapshot = session.snapshot();
        if player.overlay.scrobbling() {
            scrobble(session, &player, &snapshot, "stop");
        }
        let tracks = state
            .video
            .as_mut()
            .map(|video| video.tracks())
            .unwrap_or_default();
        if !player.local {
            overlay::finish(&player, session, &snapshot, &tracks);
        }
        if let Some(link) = player.torrent_link.clone() {
            session.deactivate_torrent(link);
        }
    }
    if let Some(video) = state.video.as_mut() {
        video.stop();
    }
    if let Some(gpu) = state.gpu.as_mut() {
        if let Some(texture) = player.texture {
            gpu.egui_renderer.free_texture(&texture);
        }
        gpu.set_hdr_wanted(false);
    }
    crate::reset_ui(state);
}

pub(crate) fn video_belongs_to(video: &str, meta: &Value) -> bool {
    let id = meta.get("id").and_then(Value::as_str).unwrap_or_default();
    id.is_empty()
        || video == id
        || video
            .strip_prefix(id)
            .is_some_and(|rest| rest.starts_with(':'))
}

pub(super) fn save_progress(session: &SessionHandle, player: &PlayerSession, snapshot: &Value) {
    if player.local || player.status.duration <= 0.0 || player.status.position < 5.0 {
        return;
    }
    let stream = overlay::current_stream(player, snapshot);
    let command = json!({
        "type": "savePlaybackProgressRequested",
        "profile": session.active_profile(),
        "meta": player.meta,
        "timeOffset": player.status.position as i64,
        "duration": player.status.duration as i64,
        "lastVideoId": snapshot
            .pointer("/player/currentVideoId")
            .filter(|video| video_belongs_to(video.as_str().unwrap_or_default(), &player.meta)),
        "lastEpisodeName": player.episode_title,
        "lastEpisodeThumbnail": player.episode_image,
        "lastEpisodeOverview": player.synopsis,
        "lastStreamIndex": snapshot.pointer("/player/currentStreamIndex"),
        "lastStreamUrl": stream.and_then(|stream| stream.get("playableUrl").or_else(|| stream.get("url"))),
        "lastStreamTitle": stream.and_then(|stream| stream.get("title").or_else(|| stream.get("name"))),
    });
    if let Err(error) = session.dispatch(command) {
        host_log(format!("core dispatch failed: {error}"));
    }
}

pub(super) fn scrobble(
    session: &SessionHandle,
    player: &PlayerSession,
    snapshot: &Value,
    action: &str,
) {
    let item_id = snapshot
        .pointer("/player/currentVideoId")
        .or_else(|| player.meta.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if player.local || item_id.is_empty() || player.status.duration <= 0.0 {
        return;
    }
    let command = json!({
        "type": "scrobbleRequested",
        "token": "",
        "metaType": player.meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
        "itemId": item_id,
        "progress": player.status.position / player.status.duration * 100.0,
        "actionName": action,
        "profile": session.active_profile(),
        "meta": player.meta,
    });
    if let Err(error) = session.dispatch(command) {
        host_log(format!("core dispatch failed: {error}"));
    }
}
