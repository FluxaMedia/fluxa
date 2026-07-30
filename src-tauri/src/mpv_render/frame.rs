use super::*;

impl MpvRenderer {
    pub fn render_thumbnail(&mut self, width: i32, height: i32) -> Result<Vec<u8>, String> {
        if !self.loaded {
            return Err("not loaded".to_string());
        }
        if self.render_context.is_null() {
            self.create_software_context()?;
        }
        let width = width.clamp(2, 1920);
        let height = height.clamp(2, 1080);
        self.ensure_buffer(width, height);

        let mut size = [width, height];
        let format = CString::new("rgb0").unwrap();
        let mut stride = (width as usize) * 4;
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_FORMAT,
                data: format.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_STRIDE,
                data: (&mut stride as *mut usize).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_POINTER,
                data: self.buffer.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (thumbnail) failed: {}",
                self.api.error_string(result)
            ));
        }

        for alpha in self.buffer.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }

        Ok(self.buffer.clone())
    }

    pub fn render_frame(&mut self, width: i32, height: i32) -> Result<PlayerFrame, String> {
        if !self.loaded {
            return Err("player has not loaded media yet".to_string());
        }
        if self.render_context.is_null() {
            self.create_software_context()?;
        }
        let width = width.clamp(2, 1920);
        let height = height.clamp(2, 1080);
        self.ensure_buffer(width, height);

        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        let mut size = [width, height];
        let format = CString::new("rgb0").unwrap();
        let mut stride = (width as usize) * 4;
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_FORMAT,
                data: format.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_STRIDE,
                data: (&mut stride as *mut usize).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_POINTER,
                data: self.buffer.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (sw) failed: {}",
                self.api.error_string(result)
            ));
        }

        for alpha in self.buffer.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }

        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }

        Ok(PlayerFrame {
            width,
            height,
            pixels_base64: general_purpose::STANDARD.encode(&self.buffer),
        })
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn render_opengl_frame(&mut self, width: i32, height: i32) -> Result<(), String> {
        if self.render_context.is_null() {
            self.create_opengl_context()?;
        }

        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        // Linux/GTK: query the offscreen FBO that GTK's GLArea binds.
        // Windows/macOS: render into the default framebuffer (FBO 0).
        #[cfg(target_os = "linux")]
        let fbo_id = query_draw_fbo();
        #[cfg(not(target_os = "linux"))]
        let fbo_id: c_int = 0;

        let mut fbo = MpvOpenGlFbo {
            fbo: fbo_id,
            width: width.max(2),
            height: height.max(2),
            internal_format: 0,
        };
        let mut flip_y: c_int = 1;
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_OPENGL_FBO,
                data: (&mut fbo as *mut MpvOpenGlFbo).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_FLIP_Y,
                data: (&mut flip_y as *mut c_int).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render failed: {}",
                self.api.error_string(result)
            ));
        }
        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }
        Ok(())
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn render_vulkan_frame(&mut self, image: &mut VulkanTargetImage) -> Result<(), String> {
        if self.render_context.is_null() {
            return Err("vulkan render context not created".to_string());
        }
        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        let mut ffi_image = MpvVulkanImageFfi {
            image: image.image,
            format: image.format,
            w: image.w,
            h: image.h,
            usage: image.usage,
            layout: image.layout,
            wait_semaphore: image.wait_semaphore,
            signal_semaphore: image.signal_semaphore,
        };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_VULKAN_IMAGE,
                data: (&mut ffi_image as *mut MpvVulkanImageFfi).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        image.layout = ffi_image.layout;
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (vulkan) failed: {}",
                self.api.error_string(result)
            ));
        }
        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn render_d3d11_frame(&mut self, tex: *mut c_void) -> Result<(), String> {
        if self.render_context.is_null() {
            return Err("d3d11 render context not created".to_string());
        }
        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        // mpv_d3d11_fbo contains only the ID3D11Texture2D pointer. The
        // texture itself supplies dimensions and format to the renderer.
        let mut target = MpvD3d11Target { tex };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_D3D11_TARGET,
                data: (&mut target as *mut MpvD3d11Target).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (d3d11) failed: {}",
                self.api.error_string(result)
            ));
        }
        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }
        Ok(())
    }

    /// Call right after the buffer swap completes.
    pub fn report_swap(&mut self) {
        if self.render_context.is_null() {
            return;
        }
        unsafe {
            (self.api.mpv_render_context_report_swap)(self.render_context);
        }
        if self.frame_ready_to_restore_audio
            && self.pending_seek_seconds.is_none()
            && !self.waiting_for_seek_restart
        {
            self.first_frame_presented = true;
        }
        self.restore_audio_after_first_presented_frame();
    }

    fn restore_audio_after_first_presented_frame(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if !self.muted_until_first_frame
                || !self.frame_ready_to_restore_audio
                || self.pending_seek_seconds.is_some()
                || self.waiting_for_seek_restart
            {
                return;
            }
            self.frame_ready_to_restore_audio = false;
            self.muted_until_first_frame = false;
            let mute = if self.restore_mute.take().unwrap_or(false) {
                "yes"
            } else {
                "no"
            };
            let _ = self.command_args(&["set", "mute", mute]);
        }
    }

    pub(super) fn restore_audio_only(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if !self.muted_until_first_frame
                || self.pending_seek_seconds.is_some()
                || self.waiting_for_seek_restart
            {
                return;
            }
            let (has_video_track, track_list_ready) = self.track_list_status();
            if track_list_ready && !has_video_track {
                self.muted_until_first_frame = false;
                let mute = if self.restore_mute.take().unwrap_or(false) {
                    "yes"
                } else {
                    "no"
                };
                let _ = self.command_args(&["set", "mute", mute]);
            }
        }
    }
}
