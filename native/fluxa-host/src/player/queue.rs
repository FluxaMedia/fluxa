use super::*;
use crate::Snapshot;

pub(crate) fn media_command(state: &mut RendererState, name: &str, value: f64) {
    if name == "fitStep" {
        step_fit(state, value as i32, false);
        return;
    }
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let Some(plan) = core_value(
        "playerMediaCommandPlan",
        json!({
            "command": name,
            "value": value,
            "paused": player.status.paused,
            "position": player.status.position,
            "duration": player.status.duration,
            "hasNext": player.overlay.upcoming().is_some(),
        }),
    ) else {
        return;
    };
    let amount = plan["value"].as_f64().unwrap_or_default();
    match plan["action"].as_str() {
        Some("togglePause") => command(state, VideoCommand::TogglePause),
        Some("seekBy") => command(state, VideoCommand::Seek(amount)),
        Some("seekTo") => command(state, VideoCommand::SeekTo(amount)),
        Some("next") => start_next(state, true),
        Some("close") => close(state),
        _ => {}
    }
}

pub(super) fn play_next(state: &mut RendererState) {
    start_next(state, false);
}

pub(super) fn current_binge_group(state: &RendererState) -> Option<String> {
    let snapshot = state.session.as_ref()?.snapshot();
    let index = snapshot.pointer("/player/currentStreamIndex")?.as_u64()? as usize;
    snapshot
        .pointer("/player/currentStreams")?
        .as_array()?
        .get(index)?
        .get("bingeGroup")?
        .as_str()
        .map(ToOwned::to_owned)
}

pub(super) fn start_next(state: &mut RendererState, any: bool) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let next = if any {
        player.overlay.upcoming()
    } else {
        player.overlay.next_video()
    };
    let Some(next) = next else {
        return;
    };
    let Some(id) = next.get("id").and_then(Value::as_str) else {
        return;
    };
    let mut item = player.meta.clone();
    let group = state
        .settings
        .bool_value("reuseBingeGroup")
        .then(|| current_binge_group(state))
        .flatten();
    if let Some(fields) = item.as_object_mut() {
        fields.retain(|key, _| {
            !key.starts_with("last") && key != "timeOffset" && key != "preferredBingeGroup"
        });
        fields.insert("lastVideoId".to_owned(), json!(id));
        if let Some(group) = group {
            fields.insert("preferredBingeGroup".to_owned(), json!(group));
        }
    }
    close(state);
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
}

pub(super) fn failover_command(player: &mut PlayerSession, snapshot: &Snapshot) -> Option<Value> {
    let failing = player.loaded_url.clone()?;
    let failed = !player.status.has_frame
        && (player.error.is_some() || player.status.error.is_some())
        && player.failed_url.as_deref() != Some(failing.as_str());
    let enabled = snapshot
        .pointer("/profile/active/autoRetryNextSource")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let streams = snapshot
        .pointer("/player/currentStreams")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    if !failed || !enabled || streams < 2 {
        return None;
    }
    player.failed_url = Some(failing);
    Some(json!({
        "type": "playerPlaybackFailed",
        "errorCode": player.error.clone().or_else(|| player.status.error.clone()),
        "title": player.title(),
        "failover": true,
    }))
}

pub(super) fn load_resolved(
    player: &mut PlayerSession,
    video: &mut Option<Box<dyn VideoBackend>>,
    gpu: &mut Option<crate::Gpu>,
    settings: &SettingsModel,
    session: Option<&SessionHandle>,
    snapshot: &Snapshot,
) {
    if player.loaded_url.is_none()
        && let Some(url) = snapshot
            .pointer("/player/resolvedUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .filter(|url| player.failed_url.as_deref() != Some(*url))
    {
        if let Some(device) = player.cast_target.take() {
            let resume = player.resume_at.take().unwrap_or(0.0);
            player.cast.start(device, url, &player.title(), resume);
            player.loaded_url = Some(url.to_owned());
            player.panel = Some(Panel::Cast);
            return;
        }
        match (video.as_mut(), gpu.as_mut()) {
            (Some(video), Some(gpu)) => {
                let trickplay = trickplay::from_snapshot(snapshot);
                let previews = player.seek_previews.get_or_insert_with(|| {
                    seek_previews::SeekPreviews::begin(
                        settings,
                        snapshot,
                        &player.meta,
                        session,
                        trickplay.is_some(),
                    )
                });
                if !previews.ready_to_load() {
                    return;
                }
                host_log(format!(
                    "loading player url host={}",
                    url.split("://")
                        .nth(1)
                        .and_then(|rest| rest.split(['/', '?']).next())
                        .unwrap_or("?")
                ));
                video.configure(&settings.values);
                video.thumbnails(&previews.initial_plan(url));
                #[cfg(not(target_arch = "wasm32"))]
                fluxa_effects::set_stream_headers(
                    url,
                    fluxa_core::player::stream_policy::selected_stream_headers(
                        &snapshot.view(&["player"]),
                    ),
                );
                gpu.set_hdr_wanted(true);
                video.load(&gpu.instance, &gpu.device, url);
                video.command(VideoCommand::Shaders(shader_chain(settings)));
                video.command(VideoCommand::Motion(motion_profile(settings)));
                video.command(VideoCommand::Fit(video_fit(settings).to_owned()));
                apply_subtitle_style(video.as_mut(), &settings.values);
                player.loaded_url = Some(url.to_owned());
                previews.loaded();
                player.trickplay = trickplay;
            }
            (None, _) => player.error = Some("No video backend on this platform".to_owned()),
            _ => {}
        }
    }
}
