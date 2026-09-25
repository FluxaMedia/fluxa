use super::*;

impl MpvRenderState {
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn needs_vulkan_context(&self) -> bool {
        self.render_context.is_null()
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn vulkan_frame_ready(&self) -> bool {
        if self.render_context.is_null() {
            return false;
        }
        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };
        update_flags & MPV_RENDER_UPDATE_FRAME != 0
    }

    #[cfg(target_os = "windows")]
    pub fn needs_d3d11_context(&self) -> bool {
        self.render_context.is_null()
    }

    pub fn reset_render_context(&mut self) {
        if self.render_context.is_null() {
            return;
        }
        unsafe {
            (self.api.mpv_render_context_free)(self.render_context);
        }
        self.render_context = ptr::null_mut();
        self.frame_state
            .frame_ready_to_restore_audio
            .store(false, Ordering::Release);
    }

    pub fn set_icc_profile(&self, data: &[u8]) -> Result<(), String> {
        if self.render_context.is_null() {
            return Err("render context not created yet".to_string());
        }
        let byte_array = MpvByteArray {
            data: data.as_ptr(),
            size: data.len(),
        };
        let param = MpvRenderParam {
            param_type: MPV_RENDER_PARAM_ICC_PROFILE,
            data: (&byte_array as *const MpvByteArray) as *mut c_void,
        };
        let result =
            unsafe { (self.api.mpv_render_context_set_parameter)(self.render_context, param) };
        if result < 0 {
            Err(format!(
                "failed to set ICC profile: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub(super) fn create_software_context(&mut self) -> Result<(), String> {
        let api_type = CString::new("sw").unwrap();
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv software render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv software render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn create_vulkan_context(
        &mut self,
        instance: *mut c_void,
        phys_device: *mut c_void,
        device: *mut c_void,
        queue_graphics_index: u32,
        queue_graphics_count: u32,
        get_proc_address: *mut c_void,
        enabled_extension_ptrs: &[*const c_char],
    ) -> Result<(), String> {
        let api_type = CString::new("vulkan").unwrap();
        let mut init_params = MpvVulkanInitParams {
            instance,
            phys_device,
            device,
            get_proc_address,
            queue_graphics_index,
            queue_graphics_count,
            enabled_extensions: enabled_extension_ptrs.as_ptr(),
            num_enabled_extensions: enabled_extension_ptrs.len() as i32,
        };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_VULKAN_INIT_PARAMS,
                data: (&mut init_params as *mut MpvVulkanInitParams).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv Vulkan render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv Vulkan render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    #[cfg(target_os = "windows")]
    pub fn create_d3d11_context(&mut self, device: *mut c_void) -> Result<(), String> {
        let api_type = CString::new("d3d11").unwrap();
        let mut init_params = MpvD3d11InitParams { device };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_D3D11_INIT_PARAMS,
                data: (&mut init_params as *mut MpvD3d11InitParams).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv D3D11 render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv D3D11 render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    pub(super) fn ensure_buffer(&mut self, width: i32, height: i32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        self.buffer
            .resize((width as usize) * (height as usize) * 4, 0);
    }
}
