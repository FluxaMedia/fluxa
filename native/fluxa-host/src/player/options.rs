use super::*;

pub(crate) fn seek(state: &mut RendererState, direction: f64) {
    let step = state
        .player
        .as_ref()
        .map_or(DEFAULT_SEEK_STEP, PlayerSession::seek_step);
    command(state, VideoCommand::Seek(direction * step));
}

pub(crate) fn command(state: &mut RendererState, command: VideoCommand) {
    if !crate::watch_together::intercept(state, &command) {
        return;
    }
    if let Some(player) = state.player.as_mut() {
        player.touch();
        match &command {
            VideoCommand::Seek(delta) => player.toast_seek(*delta),
            VideoCommand::ToggleMute => player.toast("muted", f64::from(!player.status.muted)),
            VideoCommand::SetSpeed(rate) => player.toast("speed", *rate),
            _ => {}
        }
    }
    if let Some(video) = state.video.as_mut() {
        video.command(command);
    }
}

pub(crate) fn apply_options(model: &mut PlayerModel, settings: &SettingsModel) {
    let on = |key: &str| settings.bool_value(key);
    let options = fluxa_ui::PlayerOptions {
        title: on("playerShowTitle"),
        up_next: on("playerShowUpNextCard"),
        audio_subtitles: on("playerShowAudioSubtitles"),
        speed: on("playerShowSpeed"),
        episodes: on("playerShowEpisodes"),
        next_episode: on("playerShowNextEpisode"),
        settings: on("playerShowSettings"),
        volume: on("playerShowVolume"),
        fullscreen: on("playerShowFullscreen"),
        cast: on("playerShowCast"),
        mark_segment: on("playerShowMarkSegment"),
        segments: on("playerShowSegments"),
        chapters: on("playerShowChapters"),
        gestures: on("playerGestures"),
        double_tap: on("playerDoubleTapSeek"),
        center_controls: on("playerCenterControls"),
    };
    model.boost = on("playerAudioBoost");
    model.show_pause_info &= on("pauseMetadataOverlayEnabled");
    if !on("contentWarningsEnabled") {
        model.warnings.clear();
        model.warnings_elapsed = None;
    }
    if !options.up_next {
        model.next_episode = None;
    }
    model.options = options;
}

pub(crate) const UPSCALING_MODES: [&str; 6] = ["a", "b", "c", "aa", "bb", "ca"];

pub(crate) fn upscaling(settings: &SettingsModel) -> &str {
    if settings
        .str_value("animeUpscalingMode")
        .is_none_or(|mode| mode == "off")
    {
        return "off";
    }
    match settings.str_value("animeUpscalingModePreset") {
        Some(mode) if UPSCALING_MODES.contains(&mode) => mode,
        _ => "a",
    }
}

pub(super) const FITS: [&str; 4] = ["fit", "fill", "stretch", "original"];

pub(crate) fn video_fit(settings: &SettingsModel) -> &'static str {
    FITS.into_iter()
        .find(|fit| settings.str_value("playerVideoFit") == Some(*fit))
        .unwrap_or(FITS[0])
}

pub(crate) fn step_fit(state: &mut RendererState, direction: i32, wrap: bool) {
    let current = FITS
        .iter()
        .position(|fit| *fit == video_fit(&state.settings))
        .unwrap_or(0) as i32;
    let count = FITS.len() as i32;
    let next = if wrap {
        (current + direction).rem_euclid(count)
    } else {
        (current + direction).clamp(0, count - 1)
    };
    if next == current {
        return;
    }
    let value = json!(FITS[next as usize]);
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert("playerVideoFit".to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(crate::NativeAction::SettingsChange {
            key: "playerVideoFit".to_owned(),
            value,
        });
    command(state, VideoCommand::Fit(FITS[next as usize].to_owned()));
    if let Some(player) = state.player.as_mut() {
        let label = fluxa_ui::localized(
            &format!("player.fit_{}", FITS[next as usize]),
            &player.language,
        );
        if let Some(plan) = core_value("playerToastPlan", json!({"kind": "fit", "label": label})) {
            player.toast = Some((plan, Instant::now()));
        }
    }
}

pub(super) fn apply_subtitle_style(video: &mut dyn VideoBackend, settings: &Value) {
    let Some(Value::Array(options)) = core_value("subtitleStylePlan", settings.clone()) else {
        return;
    };
    for option in options {
        if let (Some(name), Some(value)) = (option[0].as_str(), option[1].as_str()) {
            video.command(VideoCommand::Mpv(vec![
                "set".to_owned(),
                name.to_owned(),
                value.to_owned(),
            ]));
        }
    }
}

pub(super) fn motion_profile(settings: &SettingsModel) -> String {
    match settings.str_value("frameInterpolationMode") {
        Some("display_resample") => "motion-resample",
        Some("smooth") => "smooth-motion",
        _ => "",
    }
    .to_owned()
}

pub(super) fn shader_chain(settings: &SettingsModel) -> String {
    let mode = upscaling(settings);
    if mode == "off" {
        return String::new();
    }
    let tier = settings
        .str_value("animeUpscalingQuality")
        .unwrap_or("anime4k_m");
    core_value("anime4kProfile", json!({"tier": tier, "mode": mode}))
        .and_then(|profile| serde_json::from_value(profile).ok())
        .unwrap_or_default()
}

pub(crate) fn toggle_upscaling(state: &mut RendererState) {
    let next = if upscaling(&state.settings) == "off" {
        "a"
    } else {
        "off"
    };
    set_upscaling(state, next);
}

pub(crate) fn step_upscaling_mode(state: &mut RendererState, direction: i32) {
    if upscaling(&state.settings) == "off" {
        return;
    }
    let current = UPSCALING_MODES
        .iter()
        .position(|mode| *mode == upscaling(&state.settings))
        .unwrap_or(0) as i32;
    let next =
        UPSCALING_MODES[(current + direction).rem_euclid(UPSCALING_MODES.len() as i32) as usize];
    set_upscaling(state, next);
}

pub(crate) fn previous_episode(state: &mut RendererState) {
    menu::previous(state);
}

pub(crate) fn open_submit(state: &mut RendererState) {
    if let Some(player) = state.player.as_mut() {
        submit::open(player);
    }
}

pub(crate) fn set_upscaling(state: &mut RendererState, next: &str) {
    let mut changes = vec![(
        "animeUpscalingMode",
        json!(if next == "off" { "off" } else { "auto" }),
    )];
    if next != "off" {
        changes.push(("animeUpscalingModePreset", json!(next)));
    }
    for (key, value) in changes {
        if let Some(values) = state.settings.values.as_object_mut() {
            values.insert(key.to_owned(), value.clone());
        }
        state
            .pending_native_actions
            .push(crate::NativeAction::SettingsChange {
                key: key.to_owned(),
                value,
            });
    }
    let chain = shader_chain(&state.settings);
    command(state, VideoCommand::Shaders(chain));
    if let Some(player) = state.player.as_mut()
        && let Some(plan) = core_value(
            "playerToastPlan",
            json!({"kind": "anime4k", "value": if next == "off" { 0.0 } else { 1.0 }}),
        )
    {
        player.toast = Some((plan, Instant::now()));
    }
}
