use super::*;

impl MpvClientHandle {
    fn error_log_details(&self) -> Vec<String> {
        const MPV_LOG_LEVEL_WARN: c_int = 30;
        const MAX_LINES: usize = 6;
        const MAX_LINE_LEN: usize = 220;

        let mut skip_indices = std::collections::HashSet::new();
        for (i, (_, text)) in self.log_ring.iter().enumerate() {
            let Some(disp_filename) = text
                .strip_prefix("Can not open external file ")
                .and_then(|rest| rest.strip_suffix('.'))
            else {
                continue;
            };
            skip_indices.insert(i);
            if i > 0 {
                if let Some((_, prev_text)) = self.log_ring.get(i - 1) {
                    if prev_text == &format!("Failed to open {disp_filename}.") {
                        skip_indices.insert(i - 1);
                    }
                }
            }
        }

        let mut lines: Vec<String> = Vec::new();
        for (i, (level, text)) in self.log_ring.iter().enumerate().rev() {
            if *level > MPV_LOG_LEVEL_WARN {
                continue;
            }
            if skip_indices.contains(&i) {
                continue;
            }
            let mut line = text.clone();
            if line.len() > MAX_LINE_LEN {
                let mut cut = MAX_LINE_LEN;
                while !line.is_char_boundary(cut) {
                    cut -= 1;
                }
                line.truncate(cut);
                line.push('…');
            }
            if lines.contains(&line) {
                continue;
            }
            lines.push(line);
            if lines.len() >= MAX_LINES {
                break;
            }
        }
        lines.reverse();
        lines
    }

    pub fn poll_events(&mut self) -> Vec<PlayerEvent> {
        let mut events = Vec::new();
        loop {
            let raw = unsafe { (self.api.mpv_wait_event)(self.handle, 0.0) };
            if raw.is_null() {
                break;
            }
            let event = unsafe { &*raw };
            match event.event_id {
                MPV_EVENT_NONE => break,
                MPV_EVENT_LOG_MESSAGE if !event.data.is_null() => {
                    let msg = unsafe { &*(event.data as *const MpvEventLogMessage) };
                    let prefix = if msg.prefix.is_null() {
                        "unknown".into()
                    } else {
                        unsafe { CStr::from_ptr(msg.prefix) }.to_string_lossy()
                    };
                    let level = if msg.level.is_null() {
                        "unknown".into()
                    } else {
                        unsafe { CStr::from_ptr(msg.level) }.to_string_lossy()
                    };
                    let text = unsafe { CStr::from_ptr(msg.text) }.to_string_lossy();
                    let text = text.trim_end();
                    if !text.is_empty() {
                        match msg.log_level {
                            10 | 20 => log::error!("mpv [{prefix}] [{level}]: {text}"),
                            30 => log::warn!("mpv [{prefix}] [{level}]: {text}"),
                            40 => log::info!("mpv [{prefix}] [{level}]: {text}"),
                            _ => log::debug!("mpv [{prefix}] [{level}]: {text}"),
                        }
                        if self.log_ring.len() >= 40 {
                            self.log_ring.pop_front();
                        }
                        self.log_ring.push_back((msg.log_level, text.to_string()));
                    }
                }
                MPV_EVENT_END_FILE if !event.data.is_null() => {
                    let end_file = unsafe { &*(event.data as *const MpvEventEndFile) };
                    if end_file.reason == MPV_END_FILE_REASON_ERROR {
                        let mut message = self.api.error_string(end_file.error);
                        let details = self.error_log_details();
                        if !details.is_empty() {
                            message.push('\n');
                            message.push_str(&details.join("\n"));
                        }
                        let url = self.current_url.clone();
                        sentry::with_scope(
                            |scope| {
                                scope.set_tag("mpv.error_code", end_file.error);
                                if let Some(url) = &url {
                                    scope.set_extra("mpv.url", url.clone().into());
                                }
                                if !details.is_empty() {
                                    scope.set_extra("mpv.log_tail", details.join("\n").into());
                                }
                            },
                            || sentry::capture_message(&message, sentry::Level::Error),
                        );
                        events.push(PlayerEvent::EndFile {
                            eof: false,
                            error: Some(message),
                        });
                    } else if end_file.reason == MPV_END_FILE_REASON_EOF {
                        events.push(PlayerEvent::EndFile {
                            eof: true,
                            error: None,
                        });
                    }
                }
                MPV_EVENT_PROPERTY_CHANGE if !event.data.is_null() => {
                    let property = unsafe { &*(event.data as *const MpvEventProperty) };
                    if event.reply_userdata == PAUSE_OBSERVE_ID
                        && property.format == MPV_FORMAT_FLAG
                        && !property.data.is_null()
                    {
                        let paused = unsafe { *(property.data as *const c_int) } != 0;
                        events.push(PlayerEvent::PauseChanged(paused));
                    }
                }
                MPV_EVENT_COMMAND_REPLY if event.error < 0 => {
                    log::debug!(
                        "mpv async command failed: {}",
                        self.api.error_string(event.error)
                    );
                }
                MPV_EVENT_PLAYBACK_RESTART => {
                    if let Some(secs) = self.pending_seek_seconds.take() {
                        self.frame_state
                            .pending_seek_active
                            .store(false, Ordering::Release);
                        self.frame_state
                            .waiting_for_seek_restart
                            .store(true, Ordering::Release);
                        self.frame_state
                            .frame_ready_to_restore_audio
                            .store(false, Ordering::Release);
                        let _ = self.command_args(&["seek", &format!("{secs:.3}"), "absolute+exact"]);
                    } else if self
                        .frame_state
                        .waiting_for_seek_restart
                        .load(Ordering::Acquire)
                    {
                        self.frame_state
                            .waiting_for_seek_restart
                            .store(false, Ordering::Release);
                        self.frame_state
                            .frame_ready_to_restore_audio
                            .store(false, Ordering::Release);
                    }
                    if self.pending_unpause
                        && !self.frame_state.waiting_for_seek_restart.load(Ordering::Acquire)
                    {
                        self.pending_unpause = false;
                        let _ = self.command_args(&["set", "pause", "no"]);
                    }
                }
                _ => {}
            }
        }
        self.restore_audio_only();
        events
    }

    fn restore_audio_only(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if self.pending_seek_seconds.is_some()
                || self.frame_state.waiting_for_seek_restart.load(Ordering::Acquire)
                || !self.frame_state.muted_until_first_frame.load(Ordering::Acquire)
            {
                return;
            }
            let (has_video_track, track_list_ready) = self.track_list_status();
            if track_list_ready
                && !has_video_track
                && self
                    .frame_state
                    .muted_until_first_frame
                    .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                let mute = if self.frame_state.restore_mute_value.load(Ordering::Acquire) {
                    "yes"
                } else {
                    "no"
                };
                let _ = self.command_args(&["set", "mute", mute]);
            }
        }
    }

    pub(super) fn track_list_status(&self) -> (bool, bool) {
        let count = self
            .get_i64_property("track-list/count")
            .unwrap_or(0)
            .max(0) as usize;
        if count == 0 {
            return (false, false);
        }
        for index in 0..count {
            if self
                .get_string_property(&format!("track-list/{index}/type"))
                .as_deref()
                == Some("video")
            {
                return (true, true);
            }
        }
        (false, true)
    }

    pub fn status(&self) -> PlayerStatus {
        let (has_video_track, track_list_ready) = self.track_list_status();
        PlayerStatus {
            loaded: self.frame_state.loaded.load(Ordering::Acquire),
            path: self.get_string_property("path"),
            media_title: self.get_string_property("media-title"),
            time_pos: self.get_string_property("time-pos"),
            duration: self.get_string_property("duration"),
            percent_pos: self.get_string_property("percent-pos"),
            pause: self.get_string_property("pause"),
            mute: self.get_string_property("mute"),
            volume: self.get_string_property("volume"),
            core_idle: self.get_string_property("core-idle"),
            eof_reached: self.get_string_property("eof-reached"),
            vo_configured: self.get_string_property("vo-configured"),
            video_codec: self.get_string_property("video-codec"),
            video_format: self.get_string_property("video-format"),
            width: self.get_string_property("width"),
            height: self.get_string_property("height"),
            cache_speed: self.get_string_property("cache-speed"),
            demuxer_cache_duration: self.get_string_property("demuxer-cache-duration"),
            hwdec_current: self.get_string_property("hwdec-current"),
            fps: self.get_string_property("estimated-vf-fps"),
            frame_drop_count: self.get_string_property("frame-drop-count"),
            decoder_frame_drop_count: self.get_string_property("decoder-frame-drop-count"),
            avsync: self.get_string_property("avsync"),
            video_bitrate: self.get_string_property("video-bitrate"),
            audio_bitrate: self.get_string_property("audio-bitrate"),
            audio_codec: self.get_string_property("audio-codec-name"),
            audio_samplerate: self.get_string_property("audio-params/samplerate"),
            audio_channels: self.get_string_property("audio-params/channels"),
            color_primaries: self.get_string_property("video-params/primaries"),
            color_matrix: self.get_string_property("video-params/colormatrix"),
            color_gamma: self.get_string_property("video-params/gamma"),
            video_out_primaries: self.get_string_property("video-out-params/primaries"),
            video_out_matrix: self.get_string_property("video-out-params/colormatrix"),
            video_out_gamma: self.get_string_property("video-out-params/gamma"),
            sig_peak: self.get_string_property("video-params/sig-peak"),
            hdr_active: false,
            container_fps: self.get_string_property("container-fps"),
            display_fps: self.get_string_property("display-fps"),
            mistimed_frame_count: self.get_string_property("mistimed-frame-count"),
            vo_delayed_frame_count: self.get_string_property("vo-delayed-frame-count"),
            paused_for_cache: self.get_string_property("paused-for-cache"),
            cache_buffering_state: self.get_string_property("cache-buffering-state"),
            seeking: self.get_string_property("seeking"),
            file_format: self.get_string_property("file-format"),
            frames_rendered: self.frame_state.frames_rendered.load(Ordering::Relaxed),
            first_frame_presented: self.frame_state.first_frame_presented.load(Ordering::Acquire),
            has_video_track,
            track_list_ready,
            resuming: self.pending_seek_seconds.is_some()
                || self.frame_state.waiting_for_seek_restart.load(Ordering::Acquire),
        }
    }

    pub fn track_options(&self, track_type: &str) -> Vec<PlayerTrackOption> {
        let count = self
            .get_i64_property("track-list/count")
            .unwrap_or(0)
            .max(0) as usize;
        log::warn!(
            "mpv track query: requested={track_type:?}, loaded={}, count={count}, path={:?}",
            self.frame_state.loaded.load(Ordering::Acquire),
            self.get_string_property("path")
        );
        let mut tracks = Vec::new();
        for index in 0..count {
            let kind = self.get_string_property(&format!("track-list/{index}/type"));
            log::warn!(
                "mpv track query: index={index}, type={kind:?}, id={:?}, title={:?}, lang={:?}, codec={:?}, selected={:?}, external={:?}",
                self.get_i64_property(&format!("track-list/{index}/id")),
                self.get_string_property(&format!("track-list/{index}/title")),
                self.get_string_property(&format!("track-list/{index}/lang")),
                self.get_string_property(&format!("track-list/{index}/codec")),
                self.get_flag_property(&format!("track-list/{index}/selected")),
                self.get_flag_property(&format!("track-list/{index}/external")),
            );
            let Some(kind) = kind else {
                continue;
            };
            if kind != track_type {
                continue;
            }
            let id = self
                .get_i64_property(&format!("track-list/{index}/id"))
                .map(|value| value.to_string())
                .unwrap_or_else(|| (index + 1).to_string());
            let title = self
                .get_string_property(&format!("track-list/{index}/title"))
                .filter(|value| !value.trim().is_empty());
            let lang = self
                .get_string_property(&format!("track-list/{index}/lang"))
                .filter(|value| !value.trim().is_empty());
            let external_filename = self
                .get_string_property(&format!("track-list/{index}/external-filename"))
                .and_then(|value| {
                    std::path::Path::new(&value)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(str::to_string)
                })
                .filter(|value| !value.trim().is_empty());
            let external = self
                .get_flag_property(&format!("track-list/{index}/external"))
                .unwrap_or(false);
            let codec = self
                .get_string_property(&format!("track-list/{index}/codec"))
                .filter(|value| !value.trim().is_empty());
            let selected = self
                .get_flag_property(&format!("track-list/{index}/selected"))
                .unwrap_or(false);
            let fallback = if track_type == "audio" {
                format!("Audio {}", tracks.len() + 1)
            } else {
                format!("Subtitle {}", tracks.len() + 1)
            };
            let source = if external {
                title.clone().or_else(|| external_filename.clone())
            } else {
                title.clone()
            };
            let format = if track_type == "audio" {
                let channels = self
                    .get_string_property(&format!("track-list/{index}/demux-channel-count"))
                    .and_then(|value| value.parse::<u32>().ok());
                let channel_label = audio_channel_label(channels);
                match (channel_label, codec.as_deref()) {
                    (Some(ch), Some(c)) => Some(format!("{ch} · {}", c.to_uppercase())),
                    (Some(ch), None) => Some(ch),
                    (None, Some(c)) => Some(c.to_uppercase()),
                    (None, None) => None,
                }
            } else {
                codec.as_deref().map(subtitle_format_label)
            };
            tracks.push(PlayerTrackOption {
                id,
                label: title
                    .clone()
                    .or(external_filename)
                    .or(lang.clone())
                    .unwrap_or(fallback),
                selected,
                lang,
                source,
                external,
                format,
            });
        }
        tracks
    }

    pub fn query_property(&self, name: &str) -> Option<String> {
        self.get_string_property(name)
    }

    fn get_i64_property(&self, name: &str) -> Option<i64> {
        let c_name = CString::new(name).ok()?;
        let mut value = 0i64;
        let result = unsafe {
            (self.api.mpv_get_property)(
                self.handle,
                c_name.as_ptr(),
                4,
                (&mut value as *mut i64).cast(),
            )
        };
        (result >= 0).then_some(value)
    }

    fn get_flag_property(&self, name: &str) -> Option<bool> {
        let c_name = CString::new(name).ok()?;
        let mut value = 0i32;
        let result = unsafe {
            (self.api.mpv_get_property)(
                self.handle,
                c_name.as_ptr(),
                3,
                (&mut value as *mut i32).cast(),
            )
        };
        (result >= 0).then_some(value != 0)
    }
    pub(super) fn get_string_property(&self, name: &str) -> Option<String> {
        let c_name = CString::new(name).ok()?;
        let mut value: *mut c_char = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_get_property)(
                self.handle,
                c_name.as_ptr(),
                1,
                (&mut value as *mut *mut c_char).cast(),
            )
        };
        if result < 0 || value.is_null() {
            return None;
        }
        let text = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
        unsafe { (self.api.mpv_free)(value.cast()) };
        Some(text)
    }
}
