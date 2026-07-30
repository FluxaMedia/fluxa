use super::*;

impl MpvRenderer {
    pub fn new() -> Result<Self, String> {
        let api = MpvApi::load()?;
        let handle = unsafe { (api.mpv_create)() };
        if handle.is_null() {
            return Err("mpv_create returned null".to_string());
        }

        let renderer = Self {
            api,
            handle,
            render_context: ptr::null_mut(),
            buffer: Vec::new(),
            width: 0,
            height: 0,
            loaded: false,
            log_ring: std::collections::VecDeque::new(),
            frames_rendered: 0,
            first_frame_presented: false,
            pending_unpause: false,
            pending_seek_seconds: None,
            waiting_for_seek_restart: false,
            frame_ready_to_restore_audio: false,
            #[cfg(target_os = "windows")]
            muted_until_first_frame: false,
            #[cfg(target_os = "windows")]
            restore_mute: None,
            current_url: None,
        };

        renderer.set_option("terminal", "no")?;
        renderer.set_option("config", "no")?;
        renderer.set_option("vo", "libmpv")?;
        #[cfg(target_os = "windows")]
        renderer.set_option("gpu-api", "opengl")?;
        renderer.set_option("idle", "yes")?;
        renderer.set_option("keep-open", "yes")?;
        if let Err(error) = renderer.set_option("osc", "no") {
            log::warn!("set_option(osc, no) failed (mpv build without lua/OSC?): {error}");
        }
        renderer.set_option("osd-level", "0")?;
        renderer.set_option("osd-bar", "no")?;
        renderer.set_option("input-default-bindings", "yes")?;

        // GPU decode — auto-safe tries every platform API (VAAPI/NVDEC/DXVA2/VideoToolbox)
        // and falls back to software decode silently if none is available.
        renderer.set_option("hwdec", "auto-safe")?;
        renderer.set_option("hwdec-codecs", "all")?;

        renderer.set_option("video-sync", "audio")?;
        renderer.set_option("display-fps-override", "60")?;

        // Start playback immediately without waiting for the cache to fill.
        // cache-pause-initial=yes (default in many MPV builds) causes MPV to pause
        // at the beginning until demuxer-readahead-secs worth of data is buffered,
        // which looks like a frozen seekbar with no video or audio.
        renderer.set_option("cache-pause-initial", "no")?;

        // Network stream buffering (important for HLS/DASH/torrent)
        renderer.set_option("cache", "yes")?;
        renderer.set_option("cache-secs", "30")?;
        renderer.set_option("demuxer-max-bytes", "150MiB")?;
        renderer.set_option("demuxer-readahead-secs", "10")?;

        // Lower audio latency and proper app name for PulseAudio/PipeWire
        renderer.set_option("audio-buffer", "0.2")?;
        renderer.set_option("audio-client-name", "fluxa")?;

        let init_result = unsafe { (renderer.api.mpv_initialize)(renderer.handle) };
        if init_result < 0 {
            let message = renderer.api.error_string(init_result);
            unsafe { (renderer.api.mpv_terminate_destroy)(renderer.handle) };
            return Err(format!("mpv_initialize failed: {message}"));
        }

        let level = CString::new("info").unwrap();
        let log_result =
            unsafe { (renderer.api.mpv_request_log_messages)(renderer.handle, level.as_ptr()) };
        if log_result < 0 {
            log::warn!(
                "mpv: failed to enable info logging: {}",
                renderer.api.error_string(log_result)
            );
        } else {
            log::info!("mpv: info logging enabled");
        }

        Ok(renderer)
    }

    pub fn new_thumbnail() -> Result<Self, String> {
        let api = MpvApi::load()?;
        let handle = unsafe { (api.mpv_create)() };
        if handle.is_null() {
            return Err("mpv_create returned null".to_string());
        }

        let mut renderer = Self {
            api,
            handle,
            render_context: ptr::null_mut(),
            buffer: Vec::new(),
            width: 0,
            height: 0,
            loaded: false,
            log_ring: std::collections::VecDeque::new(),
            frames_rendered: 0,
            first_frame_presented: false,
            pending_unpause: false,
            pending_seek_seconds: None,
            waiting_for_seek_restart: false,
            frame_ready_to_restore_audio: false,
            #[cfg(target_os = "windows")]
            muted_until_first_frame: false,
            #[cfg(target_os = "windows")]
            restore_mute: None,
            current_url: None,
        };

        renderer.set_option("terminal", "no")?;
        renderer.set_option("config", "no")?;
        renderer.set_option("vo", "libmpv")?;
        renderer.set_option("ao", "null")?;
        renderer.set_option("audio", "no")?;
        renderer.set_option("idle", "yes")?;
        if let Err(error) = renderer.set_option("osc", "no") {
            log::warn!("set_option(osc, no) failed (mpv build without lua/OSC?): {error}");
        }
        renderer.set_option("osd-level", "0")?;
        renderer.set_option("hr-seek", "yes")?;
        renderer.set_option("pause", "yes")?;
        renderer.set_option("cache", "yes")?;
        renderer.set_option("cache-secs", "10")?;
        renderer.set_option("demuxer-max-bytes", "50MiB")?;
        renderer.set_option("demuxer-readahead-secs", "2")?;

        let init_result = unsafe { (renderer.api.mpv_initialize)(renderer.handle) };
        if init_result < 0 {
            let message = renderer.api.error_string(init_result);
            unsafe { (renderer.api.mpv_terminate_destroy)(renderer.handle) };
            return Err(format!("mpv_initialize failed: {message}"));
        }

        renderer.create_software_context()?;

        Ok(renderer)
    }

    pub fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        self.loaded = false;
        self.current_url = Some(url.to_string());
        self.log_ring.clear();
        self.frames_rendered = 0;
        self.first_frame_presented = false;
        self.pending_unpause = true;
        self.pending_seek_seconds = start_at.filter(|&s| s > 0).map(|s| s as f64);
        self.waiting_for_seek_restart = false;
        self.frame_ready_to_restore_audio = false;
        #[cfg(target_os = "windows")]
        {
            let restore_mute = if self.muted_until_first_frame {
                self.restore_mute.unwrap_or(false)
            } else {
                self.get_string_property("mute").as_deref() == Some("yes")
            };
            self.muted_until_first_frame = true;
            self.restore_mute = Some(restore_mute);
            self.command_string("set mute yes")?;
        }
        self.command(&["loadfile", url, "replace"])?;
        self.command_string("set pause yes")?;
        self.loaded = true;
        Ok(())
    }

    pub fn load_thumbnail(&mut self, url: &str) -> Result<(), String> {
        self.loaded = false;
        self.command(&["loadfile", url, "replace"])?;
        self.loaded = true;
        Ok(())
    }

    pub fn first_frame_presented(&self) -> bool {
        self.first_frame_presented
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
            self.waiting_for_seek_restart = false;
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

impl Drop for MpvRenderer {
    fn drop(&mut self) {
        unsafe {
            if !self.render_context.is_null() {
                (self.api.mpv_render_context_free)(self.render_context);
            }
            if !self.handle.is_null() {
                (self.api.mpv_terminate_destroy)(self.handle);
            }
        }
    }
}
