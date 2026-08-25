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
