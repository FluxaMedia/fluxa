use super::*;

fn audio_passthrough_failure_text(details: &[String], message: &str) -> bool {
    let text = details
        .iter()
        .map(|line| line.to_ascii_lowercase())
        .chain(std::iter::once(message.to_ascii_lowercase()))
        .collect::<Vec<_>>();
    let output_mentioned = |line: &str| {
        line.contains("audio output")
            || line.contains("audio driver")
            || line.contains("audiotrack")
            || line.contains("spdif")
            || line.starts_with("ao:")
            || line.starts_with("ao/")
    };
    let failure_mentioned = |line: &str| {
        line.contains("failed")
            || line.contains("error")
            || line.contains("could not")
            || line.contains("cannot")
            || line.contains("unsupported")
            || line.contains("not supported")
            || line.contains("unavailable")
            || line.contains("refused")
    };
    text.iter().any(|line| output_mentioned(line) && failure_mentioned(line))
}

impl MpvClientHandle {
    fn audio_output_mode(&self) -> String {
        let policy = *self.audio_policy.lock().unwrap();
        audio_output_mode_for_policy(policy).to_string()
    }

    fn is_audio_passthrough_failure(&self, details: &[String], message: &str) -> bool {
        let policy = *self.audio_policy.lock().unwrap();
        if !policy.passthrough || policy.dsp || policy.fallback_attempted {
            return false;
        }
        audio_passthrough_failure_text(details, message)
    }

    fn retry_audio_as_pcm(&mut self) -> Result<(), String> {
        let url = self
            .current_url
            .clone()
            .ok_or_else(|| "no active media for PCM audio fallback".to_string())?;
        let position = self
            .get_string_property("time-pos")
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(|value| value.round() as u64);
        {
            let mut policy = self.audio_policy.lock().unwrap();
            policy.fallback_attempted = true;
            policy.passthrough = false;
        }
        // `audio-spdif` is a runtime property in MPV. Clear it before the
        // replacement load so the same media item cannot re-enter bitstream
        // output while the retry is being prepared.
        self.command_string("set audio-spdif \"\"")?;
        self.load(&url, position)
    }

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
                        // A sink that refuses bitstream kills audio without ending
                        // the file, so the end-of-file fallback below never sees it.
                        let ao_line = format!("{prefix}: {text}");
                        if msg.log_level <= 30 && self.is_audio_passthrough_failure(&[], &ao_line) {
                            match self.retry_audio_as_pcm() {
                                Ok(()) => log::warn!(
                                    "mpv audio output rejected passthrough; retrying as decoded PCM"
                                ),
                                Err(error) => {
                                    log::warn!("mpv PCM audio fallback failed: {error}")
                                }
                            }
                        }
                    }
                }
                MPV_EVENT_END_FILE if !event.data.is_null() => {
                    let end_file = unsafe { &*(event.data as *const MpvEventEndFile) };
                    if end_file.reason == MPV_END_FILE_REASON_ERROR {
                        let details = self.error_log_details();
                        let base_message = self.api.error_string(end_file.error);
                        if self.is_audio_passthrough_failure(&details, &base_message) {
                            match self.retry_audio_as_pcm() {
                                Ok(()) => {
                                    log::warn!(
                                        "mpv passthrough failed; transparently retrying as decoded PCM"
                                    );
                                    continue;
                                }
                                Err(error) => {
                                    log::warn!("mpv PCM audio retry failed: {error}");
                                }
                            }
                        }
                        let mut message = base_message;
                        if !details.is_empty() {
                            message.push('\n');
                            message.push_str(&details.join("\n"));
                        }
                        let url = self.current_url.clone();
                        let error_code = end_file.error;
                        crate::diagnostics::report_global_with_scope(
                            message.clone(),
                            sentry::Level::Error,
                            move |scope| {
                                scope.set_tag("mpv.error_code", error_code);
                                if let Some(url) = &url {
                                    scope.set_extra("mpv.url", url.clone().into());
                                }
                                if !details.is_empty() {
                                    scope.set_extra("mpv.log_tail", details.join("\n").into());
                                }
                            },
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
                    if event.reply_userdata == TRACK_LIST_OBSERVE_ID {
                        *self.track_list_cache.lock().unwrap() = None;
                    }
                    if (STATIC_OBSERVE_BASE..STATIC_OBSERVE_BASE + STATIC_OBSERVE_PROPERTIES.len() as u64)
                        .contains(&event.reply_userdata)
                    {
                        let index = (event.reply_userdata - STATIC_OBSERVE_BASE) as usize;
                        let value = if property.format == MPV_FORMAT_STRING && !property.data.is_null() {
                            let value = unsafe { *(property.data as *const *const c_char) };
                            (!value.is_null()).then(|| unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() })
                        } else {
                            None
                        };
                        self.static_properties.insert(STATIC_OBSERVE_PROPERTIES[index].to_string(), value);
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
        if let Some(status) = *self.track_list_cache.lock().unwrap() {
            return status;
        }
        let count = self
            .get_i64_property("track-list/count")
            .unwrap_or(0)
            .max(0) as usize;
        if count == 0 {
            let status = (false, false);
            *self.track_list_cache.lock().unwrap() = Some(status);
            return status;
        }
        let mut has_video_track = false;
        for index in 0..count {
            if self
                .get_string_property(&format!("track-list/{index}/type"))
                .as_deref()
                == Some("video")
            {
                has_video_track = true;
                break;
            }
        }
        let status = (has_video_track, true);
        *self.track_list_cache.lock().unwrap() = Some(status);
        status
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
            audio_output_mode: Some(self.audio_output_mode()),
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

    pub fn fast_position_status(&self) -> PlayerPositionStatus {
        PlayerPositionStatus {
            time_pos: self.get_string_property("time-pos"),
            percent_pos: self.get_string_property("percent-pos"),
            cache_speed: self.get_string_property("cache-speed"),
            demuxer_cache_duration: self.get_string_property("demuxer-cache-duration"),
            frame_drop_count: None,
            decoder_frame_drop_count: None,
            avsync: None,
            video_bitrate: None,
            audio_bitrate: None,
            mistimed_frame_count: None,
            vo_delayed_frame_count: None,
            paused_for_cache: self.get_string_property("paused-for-cache"),
            cache_buffering_state: self.get_string_property("cache-buffering-state"),
            seeking: self.get_string_property("seeking"),
            frames_rendered: self.frame_state.frames_rendered.load(Ordering::Relaxed),
        }
    }

    pub fn stats_position_status(&self) -> PlayerPositionStatus {
        let mut status = self.fast_position_status();
        status.frame_drop_count = self.get_string_property("frame-drop-count");
        status.decoder_frame_drop_count = self.get_string_property("decoder-frame-drop-count");
        status.avsync = self.get_string_property("avsync");
        status.video_bitrate = self.get_string_property("video-bitrate");
        status.audio_bitrate = self.get_string_property("audio-bitrate");
        status.mistimed_frame_count = self.get_string_property("mistimed-frame-count");
        status.vo_delayed_frame_count = self.get_string_property("vo-delayed-frame-count");
        status
    }

    pub fn cached_static_status(&self) -> PlayerStaticStatus {
        let property = |name: &str| self.static_properties.get(name).cloned().flatten();
        let (has_video_track, track_list_ready) = self.track_list_status();
        PlayerStaticStatus {
            loaded: self.frame_state.loaded.load(Ordering::Acquire),
            path: property("path"),
            media_title: property("media-title"),
            duration: property("duration"),
            pause: property("pause"),
            mute: property("mute"),
            volume: property("volume"),
            core_idle: property("core-idle"),
            eof_reached: property("eof-reached"),
            vo_configured: property("vo-configured"),
            video_codec: property("video-codec"),
            video_format: property("video-format"),
            width: property("width"),
            height: property("height"),
            hwdec_current: property("hwdec-current"),
            fps: property("estimated-vf-fps"),
            audio_codec: property("audio-codec-name"),
            audio_samplerate: property("audio-params/samplerate"),
            audio_channels: property("audio-params/channels"),
            audio_output_mode: Some(self.audio_output_mode()),
            color_primaries: property("video-params/primaries"),
            color_matrix: property("video-params/colormatrix"),
            color_gamma: property("video-params/gamma"),
            video_out_primaries: property("video-out-params/primaries"),
            video_out_matrix: property("video-out-params/colormatrix"),
            video_out_gamma: property("video-out-params/gamma"),
            sig_peak: property("video-params/sig-peak"),
            hdr_active: false,
            container_fps: property("container-fps"),
            display_fps: property("display-fps"),
            file_format: property("file-format"),
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

#[cfg(test)]
mod tests {
    use super::{audio_output_mode_for_policy, audio_passthrough_failure_text};
    use crate::mpv_render::MpvAudioPolicyState;

    #[test]
    fn recognizes_audio_sink_bitstream_failures_only() {
        assert!(audio_passthrough_failure_text(
            &["AO: [audiotrack] spdif format not supported".to_string()],
            "error opening audio output"
        ));
        assert!(!audio_passthrough_failure_text(
            &["audio output is ready".to_string()],
            "playback ended"
        ));
        assert!(!audio_passthrough_failure_text(
            &["video decoder failed".to_string()],
            "decoder error"
        ));
        assert!(!audio_passthrough_failure_text(
            vec![
                "audio output is ready".to_string(),
                "video decoder failed".to_string(),
            ].as_slice(),
            "decoder error"
        ));
    }

    #[test]
    fn reports_runtime_audio_output_path() {
        assert_eq!(
            audio_output_mode_for_policy(MpvAudioPolicyState {
                passthrough: true,
                dsp: false,
                fallback_attempted: false,
            }),
            "passthrough"
        );
        assert_eq!(
            audio_output_mode_for_policy(MpvAudioPolicyState {
                passthrough: false,
                dsp: false,
                fallback_attempted: true,
            }),
            "pcm_fallback"
        );
        assert_eq!(
            audio_output_mode_for_policy(MpvAudioPolicyState {
                passthrough: false,
                dsp: true,
                fallback_attempted: false,
            }),
            "pcm_dsp"
        );
    }
}
