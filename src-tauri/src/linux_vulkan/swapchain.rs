use super::*;

impl VulkanContext {
    pub(super) fn create_semaphores(&mut self) -> Result<(), String> {
        let info = VkSemaphoreCreateInfo {
            s_type: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: 0,
        };
        let mut acquire: VkSemaphore = 0;
        let mut render_done: VkSemaphore = 0;
        let mut transition: VkSemaphore = 0;
        let r1 =
            unsafe { (self.fns.create_semaphore)(self.device, &info, ptr::null(), &mut acquire) };
        let r2 = unsafe {
            (self.fns.create_semaphore)(self.device, &info, ptr::null(), &mut render_done)
        };
        let r3 = unsafe {
            (self.fns.create_semaphore)(self.device, &info, ptr::null(), &mut transition)
        };
        if r1 != VK_SUCCESS || r2 != VK_SUCCESS || r3 != VK_SUCCESS {
            return Err(format!("vkCreateSemaphore failed: {r1}/{r2}/{r3}"));
        }
        self.acquire_semaphore = acquire;
        self.render_done_semaphore = render_done;
        self.transition_semaphore = transition;

        let fence_info = VkFenceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: VK_FENCE_CREATE_SIGNALED_BIT,
        };
        let mut fence: VkFence = 0;
        let r4 =
            unsafe { (self.fns.create_fence)(self.device, &fence_info, ptr::null(), &mut fence) };
        if r4 != VK_SUCCESS {
            return Err(format!("vkCreateFence failed: {r4}"));
        }
        self.in_flight_fence = fence;
        Ok(())
    }

    pub(super) fn create_command_buffer(&mut self) -> Result<(), String> {
        let pool_info = VkCommandPoolCreateInfo {
            s_type: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
            queue_family_index: self.queue_family_index,
        };
        let mut pool: VkCommandPool = ptr::null_mut();
        let result = unsafe {
            (self.fns.create_command_pool)(self.device, &pool_info, ptr::null(), &mut pool)
        };
        if result != VK_SUCCESS {
            return Err(format!("vkCreateCommandPool failed: VkResult {result}"));
        }
        let alloc_info = VkCommandBufferAllocateInfo {
            s_type: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: pool,
            level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            command_buffer_count: 1,
        };
        let mut command_buffer: VkCommandBuffer = ptr::null_mut();
        let result = unsafe {
            (self.fns.allocate_command_buffers)(self.device, &alloc_info, &mut command_buffer)
        };
        if result != VK_SUCCESS {
            unsafe { (self.fns.destroy_command_pool)(self.device, pool, ptr::null()) };
            return Err(format!(
                "vkAllocateCommandBuffers failed: VkResult {result}"
            ));
        }
        self.command_pool = pool;
        self.command_buffer = command_buffer;
        Ok(())
    }

    pub(super) fn create_swapchain(&mut self, width: u32, height: u32) -> Result<(), String> {
        let mut caps = VkSurfaceCapabilitiesKHR {
            min_image_count: 0,
            max_image_count: 0,
            current_extent: VkExtent2D::default(),
            min_image_extent: VkExtent2D::default(),
            max_image_extent: VkExtent2D::default(),
            max_image_array_layers: 0,
            supported_transforms: 0,
            current_transform: 0,
            supported_composite_alpha: 0,
            supported_usage_flags: 0,
        };
        let result = unsafe {
            (self.fns.get_physical_device_surface_capabilities_khr)(
                self.phys_device,
                self.surface,
                &mut caps,
            )
        };
        if result != VK_SUCCESS {
            return Err(format!(
                "vkGetPhysicalDeviceSurfaceCapabilitiesKHR failed: {result}"
            ));
        }

        let mut format_count: u32 = 0;
        unsafe {
            (self.fns.get_physical_device_surface_formats_khr)(
                self.phys_device,
                self.surface,
                &mut format_count,
                ptr::null_mut(),
            )
        };
        let mut formats = vec![
            VkSurfaceFormatKHR {
                format: 0,
                color_space: 0
            };
            format_count as usize
        ];
        unsafe {
            (self.fns.get_physical_device_surface_formats_khr)(
                self.phys_device,
                self.surface,
                &mut format_count,
                formats.as_mut_ptr(),
            )
        };

        let hdr_format: Option<&VkSurfaceFormatKHR> = None;
        let (chosen_format, hdr) = if let Some(f) = hdr_format {
            (*f, true)
        } else {
            let f = formats
                .iter()
                .find(|f| {
                    f.format == VK_FORMAT_B8G8R8A8_UNORM
                        && f.color_space == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
                })
                .or_else(|| {
                    formats
                        .iter()
                        .find(|f| f.format == VK_FORMAT_B8G8R8A8_UNORM)
                })
                .or_else(|| formats.first())
                .copied()
                .unwrap_or(VkSurfaceFormatKHR {
                    format: VK_FORMAT_B8G8R8A8_UNORM,
                    color_space: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                });
            (f, false)
        };
        self.hdr.store(hdr, Ordering::Release);

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count {
            image_count = caps.max_image_count;
        }

        let extent = if caps.current_extent.width != u32::MAX {
            caps.current_extent
        } else {
            VkExtent2D { width, height }
        };

        let image_usage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT
            | (caps.supported_usage_flags & VK_IMAGE_USAGE_TRANSFER_DST_BIT);
        self.image_usage = image_usage;

        let mut mode_count: u32 = 0;
        unsafe {
            (self.fns.get_physical_device_surface_present_modes_khr)(
                self.phys_device,
                self.surface,
                &mut mode_count,
                ptr::null_mut(),
            )
        };
        let mut modes = vec![0i32; mode_count as usize];
        if mode_count > 0 {
            unsafe {
                (self.fns.get_physical_device_surface_present_modes_khr)(
                    self.phys_device,
                    self.surface,
                    &mut mode_count,
                    modes.as_mut_ptr(),
                )
            };
        }
        let present_mode = if modes.contains(&VK_PRESENT_MODE_MAILBOX_KHR) {
            VK_PRESENT_MODE_MAILBOX_KHR
        } else {
            VK_PRESENT_MODE_FIFO_KHR
        };

        let old_swapchain = self.swapchain;
        let create_info = VkSwapchainCreateInfoKHR {
            s_type: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: 0,
            surface: self.surface,
            min_image_count: image_count,
            image_format: chosen_format.format,
            image_color_space: chosen_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage,
            image_sharing_mode: VK_SHARING_MODE_EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            pre_transform: caps.current_transform,
            composite_alpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            present_mode,
            clipped: 1,
            old_swapchain,
        };

        let mut swapchain: VkSwapchainKHR = 0;
        let result = unsafe {
            (self.fns.create_swapchain_khr)(self.device, &create_info, ptr::null(), &mut swapchain)
        };
        if old_swapchain != 0 {
            unsafe { (self.fns.destroy_swapchain_khr)(self.device, old_swapchain, ptr::null()) };
        }
        if result != VK_SUCCESS {
            return Err(format!("vkCreateSwapchainKHR failed: VkResult {result}"));
        }

        let mut image_count_out: u32 = 0;
        unsafe {
            (self.fns.get_swapchain_images_khr)(
                self.device,
                swapchain,
                &mut image_count_out,
                ptr::null_mut(),
            )
        };
        let mut images = vec![0u64; image_count_out as usize];
        unsafe {
            (self.fns.get_swapchain_images_khr)(
                self.device,
                swapchain,
                &mut image_count_out,
                images.as_mut_ptr(),
            )
        };

        self.swapchain = swapchain;
        self.images = images;
        self.image_format = chosen_format.format;
        self.extent = extent;
        Ok(())
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), String> {
        let width = width.max(2) as u32;
        let height = height.max(2) as u32;
        if self.extent.width == width && self.extent.height == height {
            return Ok(());
        }
        unsafe { (self.fns.device_wait_idle)(self.device) };
        self.create_swapchain(width, height)
    }
}
