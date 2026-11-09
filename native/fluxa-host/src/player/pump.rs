use super::*;

pub(super) fn fall_back_to_sources(state: &mut RendererState) {
    if state
        .player
        .as_ref()
        .is_some_and(PlayerSession::saved_stream_failed)
    {
        pick_another_source(state);
    }
}

pub(super) fn pick_another_source(state: &mut RendererState) {
    let Some(mut item) = state.player.as_ref().map(|player| player.meta.clone()) else {
        return;
    };
    if let Some(fields) = item.as_object_mut() {
        for key in [
            "lastStream",
            "lastStreamUrl",
            "lastStreamTitle",
            "lastStreamIndex",
        ] {
            fields.remove(key);
        }
        fields.insert("pickSource".to_owned(), true.into());
    }
    close(state);
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
}

pub(crate) fn pump(state: &mut RendererState) {
    fall_back_to_sources(state);
    if let Some((time, _)) = state
        .player
        .as_mut()
        .and_then(|player| player.scrub.take_if(|(_, at)| at.elapsed() >= SCRUB_COMMIT))
    {
        command(state, VideoCommand::SeekTo(time));
    }
    if let Some(Panel::Submit(submit)) = state
        .player
        .as_mut()
        .and_then(|player| player.panel.as_mut())
    {
        submit.poll();
    }
    if let Some(player) = state.player.as_mut() {
        player.cast.poll();
    }
    advance_shuffle(state);
    menu::tick_sleep(state);
    menu::tick_style(state);
    menu::tick_subtitle_lift(state);
    subtitle_sync::tick(state);
    external_audio::tick(state);
    let RendererState {
        shuffle,
        player,
        session,
        video,
        gpu,
        settings,
        core_snapshot,
        pending_native_actions,
        ..
    } = state;
    let Some(player) = player.as_mut() else {
        return;
    };
    let session = session.as_ref();
    let snapshot = match (session, core_snapshot.as_ref()) {
        (Some(session), _) => session.snapshot(),
        (None, Some(snapshot)) => snapshot.clone(),
        (None, None) => return,
    };
    player.passthrough = video.as_ref().is_some_and(|video| video.passthrough());
    if player.snapshot_logo.is_none() {
        player.snapshot_logo = detail_logo(&snapshot, &player.meta);
    }
    if player.stale.is_some() {
        if player.stale.as_ref() == snapshot.get("player") && Instant::now() < player.stale_until {
            if let Some(session) = session {
                refresh_display(player, session, settings, &snapshot);
            }
            return;
        }
        player.stale = None;
    }
    if !player.started && !session.is_some_and(|session| session.has_queued_dispatches()) {
        if let Some(command) = resolution_command(&snapshot, player) {
            let starts = command_starts_playback(&snapshot);
            match session {
                Some(session) => match session.dispatch(command) {
                    Ok(()) => player.started = starts,
                    Err(error) => player.error = Some(error),
                },
                None if player.dispatched.as_ref() != Some(&command) => {
                    player.dispatched = Some(command.clone());
                    pending_native_actions.push(NativeAction::CoreCommand { command });
                    player.started = starts;
                }
                None => {}
            }
        }
    }
    let Some(session) = session else {
        load_resolved(player, video, gpu, settings, None, &snapshot);
        if let Some(video) = video.as_mut() {
            player.status = video.status();
        }
        player.episode_title = episode_title(&player.meta, &snapshot);
        return;
    };
    if player.trailer.is_some() {
        trailer::tick(player, session, video, gpu, &snapshot);
        if let Some(video) = video.as_mut() {
            player.status = video.status();
        }
        player.language =
            profile_language(snapshot.pointer("/profile/active").unwrap_or(&Value::Null));
        return;
    }
    poll_torrent(player, session, &snapshot);
    load_resolved(player, video, gpu, settings, Some(session), &snapshot);
    if let Some(command) = failover_command(player, &snapshot)
        && session.dispatch(command).is_ok()
    {
        player.error = None;
        player.loaded_url = None;
        player.started = false;
    }
    let mut advance = false;
    let mut restart_at = None;
    if let Some(video) = video.as_mut() {
        let mut status = video.status();
        if player.loaded_url.is_none() {
            status = VideoStatus::default();
        }
        log_status_change(&player.status, &status);
        player.had_frame |= status.has_frame;
        player.status = status;
        player.pin_volume();
        restart_at = recover::tick(player, &snapshot);
        player.subtitle = video.subtitle();
        if player.status.has_frame && player.loaded_url.is_some() && !player.tracks_restored {
            player.tracks_restored = true;
            menu::restore_tracks(player, video.as_mut(), settings);
            if let Some(at) = player.resume_at.take() {
                video.command(VideoCommand::SeekTo(at));
            }
            if std::mem::take(&mut player.start_paused) {
                video.command(VideoCommand::Mpv(vec![
                    "set".to_owned(),
                    "pause".to_owned(),
                    "yes".to_owned(),
                ]));
            }
        }
        if let Some(previews) = player.seek_previews.as_mut() {
            previews.tick(video.as_mut());
            if let Some(source) = previews.server_source() {
                player.trickplay = Some(trickplay::from_source(source));
            }
        }
        advance = overlay::tick(player, session, video.as_mut(), &snapshot);
        refresh_stats(player, video.as_mut(), &snapshot);
        let thumbnail = player
            .trickplay
            .as_mut()
            .and_then(|trickplay| trickplay.poll(session))
            .or_else(|| video.take_thumbnail());
        let thumbnail = match (thumbnail, gpu.as_ref()) {
            (Some(thumbnail), Some(gpu)) => player.filmstrip.accept(thumbnail, &gpu.egui_context),
            (thumbnail, _) => thumbnail,
        };
        if let Some(time) = player.filmstrip.next() {
            match player
                .trickplay
                .as_mut()
                .filter(|trickplay| trickplay.usable())
            {
                Some(trickplay) => trickplay.request(time),
                None => video.request_thumbnail(time),
            }
        }
        if let (Some(thumbnail), Some(gpu)) = (thumbnail, gpu.as_ref()) {
            let image = egui::ColorImage::from_rgba_unmultiplied(thumbnail.size, &thumbnail.rgba);
            match player.thumbnail.as_mut() {
                Some((time, texture)) => {
                    texture.set(image, egui::TextureOptions::LINEAR);
                    *time = thumbnail.time;
                }
                None => {
                    player.thumbnail = Some((
                        thumbnail.time,
                        gpu.egui_context.load_texture(
                            "fluxa-seek-thumbnail",
                            image,
                            egui::TextureOptions::LINEAR,
                        ),
                    ))
                }
            }
        }
    }
    if !player.status.has_frame {
        let preload = player
            .torrent_link
            .as_ref()
            .and(player.torrent_status.as_ref())
            .and_then(|status| status.get("preload"))
            .and_then(Value::as_f64)
            .filter(|percent| *percent > 0.0)
            .map(|percent| (percent / 100.0) as f32);
        if let Some(progress) = preload.or(player.status.buffering) {
            player.load_progress = player.load_progress.max(progress.clamp(0.045, 1.0));
        }
    }
    refresh_display(player, session, settings, &snapshot);
    tick_warnings(player);
    if shuffle.is_none() && !player.local {
        tick_recommendations(player, session, settings, &snapshot);
    }
    publish_media(player, video);
    if let Some(at) = restart_at {
        recover::restart(state, at, &snapshot);
        return;
    }
    let ended = state
        .player
        .as_ref()
        .is_some_and(|player| player.status.ended);
    let autoplays = state
        .player
        .as_ref()
        .is_some_and(|player| player.overlay.autoplays());
    if advance || (ended && autoplays) {
        play_next(state);
    } else if ended {
        close(state);
    }
}

pub(super) fn refresh_display(
    player: &mut PlayerSession,
    session: &SessionHandle,
    settings: &SettingsModel,
    snapshot: &Value,
) {
    player.episode_title = episode_title(&player.meta, snapshot);
    player.episode_image = episode_image(&player.meta, snapshot);
    player.season_episode = season_episode(&player.meta, snapshot);
    player.synopsis = synopsis(&player.meta, snapshot);
    player.stream_addons = stream_addon_names(snapshot);
    player.pending_addons = player.stream_addons.clone();
    seek_logo(player, session, snapshot);
    if !player.warnings_requested {
        player.warnings_requested = true;
        player.language =
            profile_language(snapshot.pointer("/profile/active").unwrap_or(&Value::Null));
        player.description = player
            .meta
            .get("description")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(|text| {
                if !settings.bool_value("shortenSynopsis") {
                    return text.to_owned();
                }
                core_value("shortenSynopsis", json!({"text": text}))
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .unwrap_or_else(|| text.to_owned())
            });
        player.warnings_rx =
            content_warning_url(&player.meta, snapshot).map(|url| session.fetch_json(url));
    }
    player.poll_streams(session);
}

pub(super) fn publish_media(player: &mut PlayerSession, video: &mut Option<Box<dyn VideoBackend>>) {
    let Some(video) = video.as_mut().filter(|_| player.status.has_frame) else {
        return;
    };
    let speed = if player.speed_held { FAST_SPEED } else { 1.0 };
    let has_next = player.overlay.upcoming().is_some();
    let key = format!(
        "{}|{:?}|{}|{}|{}|{}",
        player.title(),
        player.episode_title,
        player.status.paused,
        player.status.duration as i64,
        has_next,
        speed
    );
    let position = player.status.position;
    let unchanged = player.media_sent.as_ref().is_some_and(|(at, sent, from)| {
        let advance = if player.status.paused {
            0.0
        } else {
            at.elapsed().as_secs_f64() * speed
        };
        *sent == key && (from + advance - position).abs() < MEDIA_DRIFT
    });
    if unchanged {
        return;
    }
    let poster = player
        .meta
        .get("poster")
        .and_then(Value::as_str)
        .filter(|url| url.starts_with("http"));
    let input = json!({
        "title": player.title(),
        "subtitle": player.episode_title,
        "poster": poster,
        "paused": player.status.paused,
        "buffering": player.status.buffering.is_some(),
        "position": position,
        "duration": player.status.duration,
        "speed": speed,
        "hasNext": has_next,
    });
    if let Some(plan) = core_value("playerMediaSessionPlan", input) {
        video.media_session(&plan);
    }
    player.media_sent = Some((Instant::now(), key, position));
}
