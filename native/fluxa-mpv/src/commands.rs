use super::*;

impl MpvClientHandle {
    pub fn command_string(&self, command: &str) -> Result<(), String> {
        let c_command = CString::new(command).map_err(|error| error.to_string())?;
        let result = unsafe { (self.api.mpv_command_string)(self.handle, c_command.as_ptr()) };
        if result < 0 {
            Err(format!(
                "mpv command failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn command(&self, args: &[&str]) -> Result<(), String> {
        let c_args = args
            .iter()
            .map(|arg| CString::new(*arg).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut raw_args = c_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        raw_args.push(ptr::null());

        let result = unsafe { (self.api.mpv_command)(self.handle, raw_args.as_ptr()) };
        if result < 0 {
            Err(format!(
                "mpv command failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn command_args(&self, args: &[&str]) -> Result<(), String> {
        let c_args = args
            .iter()
            .map(|arg| CString::new(*arg).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut raw_args = c_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        raw_args.push(ptr::null());

        let id = self.next_async_command_id.fetch_add(1, Ordering::Relaxed);
        let result = unsafe { (self.api.mpv_command_async)(self.handle, id, raw_args.as_ptr()) };
        if result < 0 {
            Err(format!(
                "mpv async command failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn set_log_level(&self, level: &str) -> Result<(), String> {
        let c_level = CString::new(level).map_err(|error| error.to_string())?;
        let result = unsafe { (self.api.mpv_request_log_messages)(self.handle, c_level.as_ptr()) };
        if result < 0 {
            Err(self.api.error_string(result))
        } else {
            Ok(())
        }
    }

    pub fn thumbnail(&self, time: f64) -> Option<SeekThumbnail> {
        let time_arg = CString::new(format!("{time:.3}")).ok()?;
        let args = [c"thumbnail".as_ptr(), time_arg.as_ptr(), ptr::null()];
        let mut node = MpvNode {
            data: MpvNodeData { int64: 0 },
            format: 0,
        };
        let result = unsafe { (self.api.mpv_command_ret)(self.handle, args.as_ptr(), &mut node) };
        if result < 0 {
            return None;
        }
        let thumbnail = unsafe { read_thumbnail(&node) };
        unsafe { (self.api.mpv_free_node_contents)(&mut node) };
        thumbnail
    }

    pub fn thumbnail_progress(&self) -> Option<(usize, usize)> {
        let count = self.get_i64_property("thumbnail-info/count")?;
        let done = self.get_i64_property("thumbnail-info/done")?;
        Some((done.max(0) as usize, count.max(0) as usize))
    }

    pub fn reload_thumbnails(&self) -> Result<(), String> {
        self.command(&["thumbnail-reload"])
    }

    pub fn hold_start(&mut self, hold: bool) {
        self.hold_unpause = hold;
    }

    pub fn release_start(&mut self) {
        self.hold_unpause = false;
        if self.pending_unpause
            && !self
                .frame_state
                .waiting_for_seek_restart
                .load(Ordering::Acquire)
        {
            self.pending_unpause = false;
            let _ = self.command_args(&["set", "pause", "no"]);
        }
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<(), String> {
        let c_name = CString::new(name).map_err(|error| error.to_string())?;
        let c_value = CString::new(value).map_err(|error| error.to_string())?;
        let result = unsafe {
            (self.api.mpv_set_option_string)(self.handle, c_name.as_ptr(), c_value.as_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv option '{name}' failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn set_http_headers(&self, headers: &[(String, String)]) -> Result<(), String> {
        if headers.is_empty() {
            return self.set_option("http-header-fields", "");
        }
        let joined = headers
            .iter()
            .map(|(key, value)| format!("{key}: {value}").replace(',', "\\,"))
            .collect::<Vec<_>>()
            .join(",");
        self.set_option("http-header-fields", &joined)
    }

    pub fn apply_options(&self, options: &[(String, String)]) -> Result<(), String> {
        {
            let mut policy = self.audio_policy.lock().unwrap();
            let previous_passthrough = policy.passthrough;
            for (name, value) in options {
                match name.trim().to_ascii_lowercase().as_str() {
                    "audio-spdif" => policy.passthrough = !value.trim().is_empty(),
                    "af" => policy.dsp = !value.trim().is_empty(),
                    _ => {}
                }
            }
            if policy.dsp {
                policy.passthrough = false;
            }
            if policy.passthrough && !previous_passthrough {
                policy.fallback_attempted = false;
            }
        }
        for (name, value) in options {
            let result = if name == "glsl-shaders" {
                if value.is_empty() {
                    self.command_args(&["change-list", name, "clr", ""])
                } else {
                    self.command_args(&["change-list", name, "set", value])
                }
            } else {
                self.command_args(&["set", name, value])
            };
            if let Err(error) = result {
                log::warn!("mpv preference skipped: {error}");
            }
        }
        Ok(())
    }
}

unsafe fn read_thumbnail(node: &MpvNode) -> Option<SeekThumbnail> {
    if node.format != MPV_FORMAT_NODE_MAP {
        return None;
    }
    let list = unsafe { node.data.list.as_ref()? };
    let (mut width, mut height, mut stride, mut time, mut exact) = (0, 0, 0, 0.0, false);
    let mut bytes: Option<&[u8]> = None;
    for index in 0..list.count.max(0) as usize {
        let key = unsafe { CStr::from_ptr(*list.keys.add(index)) }.to_bytes();
        let value = unsafe { &*list.values.add(index) };
        match key {
            b"w" => width = unsafe { value.data.int64 } as usize,
            b"h" => height = unsafe { value.data.int64 } as usize,
            b"stride" => stride = unsafe { value.data.int64 } as usize,
            b"time" => time = unsafe { value.data.double },
            b"exact" => exact = unsafe { value.data.flag } != 0,
            b"data" if value.format == MPV_FORMAT_BYTE_ARRAY => {
                let array = unsafe { value.data.bytes.as_ref()? };
                bytes = Some(unsafe { std::slice::from_raw_parts(array.data, array.size) });
            }
            _ => {}
        }
    }
    let bytes = bytes?;
    if width == 0 || height == 0 || stride < width * 4 || bytes.len() < stride * height {
        return None;
    }
    let mut rgba = Vec::with_capacity(width * height * 4);
    for row in bytes.chunks(stride).take(height) {
        for pixel in row[..width * 4].chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }
    Some(SeekThumbnail {
        width,
        height,
        time,
        exact,
        rgba,
    })
}
