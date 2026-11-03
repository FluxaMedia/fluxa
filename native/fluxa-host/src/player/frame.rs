use super::*;

pub(crate) fn upload_frame(state: &mut RendererState) {
    let RendererState {
        player, video, gpu, ..
    } = state;
    let (Some(player), Some(video), Some(gpu)) = (player.as_mut(), video.as_mut(), gpu.as_mut())
    else {
        return;
    };
    if let Some(view) = video.render(&gpu.device) {
        gpu.set_hdr_video(Some(view.clone()));
        if let Some(old) = player.texture.take() {
            gpu.egui_renderer.free_texture(&old);
        }
        player.texture = Some(gpu.egui_renderer.register_native_texture(
            &gpu.device,
            &view,
            wgpu::FilterMode::Linear,
        ));
    }
}

pub(super) fn refresh_stats(
    player: &mut PlayerSession,
    video: &mut dyn VideoBackend,
    snapshot: &Value,
) {
    let interval = if player.stats_visible {
        stats::REFRESH
    } else {
        stats::SAMPLE
    };
    if !player.stats_visible {
        player.stats.clear();
    }
    if player.stats_at.is_some_and(|at| at.elapsed() < interval) {
        return;
    }
    player.stats_at = Some(Instant::now());
    let stats = video.stats();
    let playing = stats
        .as_ref()
        .map(PlaybackStats::playing)
        .unwrap_or_default();
    let stream = overlay::current_stream(player, snapshot).unwrap_or(&Value::Null);
    player.playing_badges = crate::stream_badges::playing_badges(stream, &playing);
    let Some(stats) = stats else {
        return;
    };
    if player.stats_visible {
        player.stats = stats.sections();
    }
    player.last_stats = Some(stats);
}

pub(super) fn remember_summary(state: &mut RendererState, player: &PlayerSession) {
    let live = state.video.as_mut().and_then(|video| video.stats());
    let Some(stats) = live
        .filter(|stats| stats.video_codec.is_some())
        .or_else(|| player.last_stats.clone())
    else {
        return;
    };
    let watched = (player.status.position > 0.0).then(|| {
        let position = fluxa_ui::format_time(player.status.position);
        match player.status.duration {
            duration if duration > 0.0 => {
                format!("{position} / {}", fluxa_ui::format_time(duration))
            }
            _ => position,
        }
    });
    let details = vec![
        ("settings.summary_title_label", Some(player.title())),
        ("settings.summary_watched", watched),
    ];
    state.settings.playback_summary = stats.summary(details);
}
