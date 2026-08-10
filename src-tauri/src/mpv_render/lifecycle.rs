use super::*;

impl MpvClientHandle {
    fn new_internal(thumbnail: bool) -> Result<(Self, MpvRenderState), String> {
        let api = MpvApi::load()?;
        let handle = unsafe { (api.mpv_create)() };
        if handle.is_null() {
            return Err("mpv_create returned null".to_string());
        }
        let api = Arc::new(api);
        let frame_state = Arc::new(MpvFrameState::new());

        let client = Self {
            api: api.clone(),
            handle,
            frame_state: frame_state.clone(),
            log_ring: std::collections::VecDeque::new(),
            pending_unpause: false,
            pending_seek_seconds: None,
            current_url: None,
            next_async_command_id: AtomicU64::new(2000),
        };
        let render = MpvRenderState {
            api: api.clone(),
            handle,
            render_context: ptr::null_mut(),
            buffer: Vec::new(),
            width: 0,
            height: 0,
            frame_state,
        };

        if thumbnail {
            client.set_option("terminal", "no")?;
            client.set_option("config", "no")?;
            client.set_option("vo", "libmpv")?;
            client.set_option("ao", "null")?;
            client.set_option("audio", "no")?;
            client.set_option("idle", "yes")?;
            if let Err(error) = client.set_option("osc", "no") {
                log::warn!("set_option(osc, no) failed (mpv build without lua/OSC?): {error}");
            }
            client.set_option("osd-level", "0")?;
            client.set_option("hr-seek", "yes")?;
            client.set_option("pause", "yes")?;
            client.set_option("cache", "yes")?;
            client.set_option("cache-secs", "2")?;
            client.set_option("demuxer-max-bytes", "8MiB")?;
            client.set_option("demuxer-readahead-secs", "1")?;
        } else {
            client.set_option("terminal", "no")?;
            client.set_option("config", "no")?;
            client.set_option("vo", "libmpv")?;
            #[cfg(target_os = "windows")]
            client.set_option("gpu-api", "opengl")?;
            client.set_option("idle", "yes")?;
            client.set_option("keep-open", "yes")?;
            if let Err(error) = client.set_option("osc", "no") {
                log::warn!("set_option(osc, no) failed (mpv build without lua/OSC?): {error}");
            }
            client.set_option("osd-level", "0")?;
            client.set_option("osd-bar", "no")?;
            client.set_option("input-default-bindings", "yes")?;

            // GPU decode — auto-safe tries every platform API (VAAPI/NVDEC/DXVA2/VideoToolbox)
            // and falls back to software decode silently if none is available.
            client.set_option("hwdec", "auto-safe")?;
            client.set_option("hwdec-codecs", "all")?;

            client.set_option("video-sync", "audio")?;
            client.set_option("display-fps-override", "60")?;

            // Start playback immediately without waiting for the cache to fill.
            // cache-pause-initial=yes (default in many MPV builds) causes MPV to pause
            // at the beginning until demuxer-readahead-secs worth of data is buffered,
            // which looks like a frozen seekbar with no video or audio.
            client.set_option("cache-pause-initial", "no")?;

            // Network stream buffering (important for HLS/DASH/torrent)
            client.set_option("cache", "yes")?;
            client.set_option("cache-secs", "30")?;
            client.set_option("demuxer-max-bytes", "150MiB")?;
            client.set_option("demuxer-readahead-secs", "10")?;

            // Lower audio latency and proper app name for PulseAudio/PipeWire
            client.set_option("audio-buffer", "0.2")?;
            client.set_option("audio-client-name", "fluxa")?;
        }

        let init_result = unsafe { (client.api.mpv_initialize)(client.handle) };
        if init_result < 0 {
            let message = client.api.error_string(init_result);
            unsafe { (client.api.mpv_terminate_destroy)(client.handle) };
            return Err(format!("mpv_initialize failed: {message}"));
        }

        if thumbnail {
            return Ok((client, render));
        }

        let level = CString::new("info").unwrap();
        let log_result = unsafe { (client.api.mpv_request_log_messages)(client.handle, level.as_ptr()) };
        if log_result < 0 {
            log::warn!(
                "mpv: failed to enable info logging: {}",
                client.api.error_string(log_result)
            );
        } else {
            log::info!("mpv: info logging enabled");
        }

        let pause_name = CString::new("pause").unwrap();
        let observe_result = unsafe {
            (client.api.mpv_observe_property)(
                client.handle,
                PAUSE_OBSERVE_ID,
                pause_name.as_ptr(),
                MPV_FORMAT_FLAG,
            )
        };
        if observe_result < 0 {
            log::warn!(
                "mpv: failed to observe pause property: {}",
                client.api.error_string(observe_result)
            );
        }

        Ok((client, render))
    }

    pub fn new() -> Result<(Self, MpvRenderState), String> {
        Self::new_internal(false)
    }

    pub fn new_thumbnail() -> Result<(Self, MpvRenderState), String> {
        let (client, mut render) = Self::new_internal(true)?;
        render.create_software_context()?;
        Ok((client, render))
    }

    pub fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        self.frame_state.loaded.store(false, Ordering::Release);
        self.current_url = Some(url.to_string());
        self.log_ring.clear();
        self.frame_state.frames_rendered.store(0, Ordering::Relaxed);
        self.frame_state
            .first_frame_presented
            .store(false, Ordering::Release);
        self.pending_unpause = true;
        self.pending_seek_seconds = start_at.filter(|&s| s > 0).map(|s| s as f64);
        self.frame_state
            .pending_seek_active
            .store(self.pending_seek_seconds.is_some(), Ordering::Release);
        self.frame_state
            .waiting_for_seek_restart
            .store(false, Ordering::Release);
        self.frame_state
            .frame_ready_to_restore_audio
            .store(false, Ordering::Release);
        #[cfg(target_os = "windows")]
        {
            let already_muted_until_first_frame = self
                .frame_state
                .muted_until_first_frame
                .load(Ordering::Acquire);
            let restore_mute = if already_muted_until_first_frame {
                self.frame_state.restore_mute_value.load(Ordering::Acquire)
            } else {
                self.get_string_property("mute").as_deref() == Some("yes")
            };
            self.frame_state
                .muted_until_first_frame
                .store(true, Ordering::Release);
            self.frame_state
                .restore_mute_value
                .store(restore_mute, Ordering::Release);
            self.command_string("set mute yes")?;
        }
        self.command(&["loadfile", url, "replace"])?;
        self.command_string("set pause yes")?;
        self.frame_state.loaded.store(true, Ordering::Release);
        Ok(())
    }

    pub fn load_thumbnail(&mut self, url: &str) -> Result<(), String> {
        self.frame_state.loaded.store(false, Ordering::Release);
        self.command(&["loadfile", url, "replace"])?;
        self.frame_state.loaded.store(true, Ordering::Release);
        Ok(())
    }

    pub fn first_frame_presented(&self) -> bool {
        self.frame_state.first_frame_presented.load(Ordering::Acquire)
    }

    pub fn seek_to(&self, time_pos: f64) -> Result<(), String> {
        self.command_string(&format!("seek {time_pos:.3} absolute+exact"))
    }

    /// Dispatch a command originating from the UI. Startup resume/unpause work
    /// must never run later and overwrite an explicit user action.
    pub fn user_command(&mut self, command: &str) -> Result<(), String> {
        let command = command.trim();
        if command == "cycle pause" || command.starts_with("set pause ") {
            self.pending_unpause = false;
        }
        if command.starts_with("seek ") || command.starts_with("set time-pos ") {
            self.pending_seek_seconds = None;
            self.frame_state
                .pending_seek_active
                .store(false, Ordering::Release);
            self.frame_state
                .waiting_for_seek_restart
                .store(false, Ordering::Release);
        }
        self.command_string(command)
    }

    pub fn add_subtitle(
        &self,
        url: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), String> {
        let title = title.unwrap_or("Subtitle");
        match language.filter(|value| !value.is_empty()) {
            Some(language) => self.command_args(&["sub-add", url, "auto", title, language]),
            None => self.command_args(&["sub-add", url, "auto", title]),
        }
    }

    pub fn title(&self) -> Option<String> {
        self.get_string_property("media-title")
    }
}

impl Drop for MpvClientHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                (self.api.mpv_terminate_destroy)(self.handle);
            }
        }
    }
}

impl Drop for MpvRenderState {
    fn drop(&mut self) {
        unsafe {
            if !self.render_context.is_null() {
                (self.api.mpv_render_context_free)(self.render_context);
            }
        }
    }
}

impl MpvThumbnailRenderer {
    pub fn new() -> Result<Self, String> {
        let (client, render) = MpvClientHandle::new_thumbnail()?;
        Ok(Self { client, render })
    }

    pub fn load_thumbnail(&mut self, url: &str) -> Result<(), String> {
        self.client.load_thumbnail(url)
    }

    pub fn query_property(&self, name: &str) -> Option<String> {
        self.client.query_property(name)
    }

    pub fn seek_to(&self, time_pos: f64) -> Result<(), String> {
        self.client.seek_to(time_pos)
    }

    pub fn render_thumbnail(&mut self, width: i32, height: i32) -> Result<Vec<u8>, String> {
        self.render.render_thumbnail(width, height)
    }
}
