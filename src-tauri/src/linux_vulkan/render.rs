use super::*;

impl VulkanContext {
    pub fn device_handles(&self) -> (*mut c_void, *mut c_void, *mut c_void, u32, u32, *mut c_void) {
        (
            self.instance,
            self.phys_device,
            self.device,
            self.queue_family_index,
            1,
            self.fns.get_instance_proc_addr as *mut c_void,
        )
    }

    pub fn image_usage(&self) -> u32 {
        self.image_usage
    }

    pub fn is_hdr(&self) -> bool {
        self.hdr.load(Ordering::Acquire)
    }

    pub fn enabled_device_extension_ptrs(&self) -> Vec<*const i8> {
        self.enabled_device_extensions
            .iter()
            .map(|e| e.as_ptr())
            .collect()
    }

    pub fn render_and_present<F>(&mut self, render: F) -> Result<(), String>
    where
        F: FnMut(u64, i32, u32, u32, u64, u64) -> Result<i32, String>,
    {
        let result = self.render_and_present_inner(render);
        if result.is_err() {
            self.recover_after_render_error();
        }
        result
    }

    fn recover_after_render_error(&mut self) {
        unsafe {
            (self.fns.device_wait_idle)(self.device);
            for sem in [
                self.acquire_semaphore,
                self.render_done_semaphore,
                self.transition_semaphore,
            ] {
                if sem != 0 {
                    (self.fns.destroy_semaphore)(self.device, sem, ptr::null());
                }
            }
            if self.in_flight_fence != 0 {
                (self.fns.destroy_fence)(self.device, self.in_flight_fence, ptr::null());
            }
        }
        self.acquire_semaphore = 0;
        self.render_done_semaphore = 0;
        self.transition_semaphore = 0;
        self.in_flight_fence = 0;
        let extent = self.extent;
        let _ = self.create_semaphores();
        let _ = self.create_swapchain(extent.width, extent.height);
    }

    fn render_and_present_inner<F>(&mut self, mut render: F) -> Result<(), String>
    where
        F: FnMut(u64, i32, u32, u32, u64, u64) -> Result<i32, String>,
    {
        unsafe {
            let wait_result =
                (self.fns.wait_for_fences)(self.device, 1, &self.in_flight_fence, 1, u64::MAX);
            if wait_result != VK_SUCCESS {
                return Err(format!("vkWaitForFences failed: {wait_result}"));
            }
            let reset_result = (self.fns.reset_fences)(self.device, 1, &self.in_flight_fence);
            if reset_result != VK_SUCCESS {
                return Err(format!("vkResetFences failed: {reset_result}"));
            }
        }
        let mut image_index: u32 = 0;
        let result = unsafe {
            (self.fns.acquire_next_image_khr)(
                self.device,
                self.swapchain,
                100_000_000,
                self.acquire_semaphore,
                ptr::null_mut(),
                &mut image_index,
            )
        };
        if result == VK_TIMEOUT {
            return Ok(());
        }
        if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            return Err(format!("vkAcquireNextImageKHR failed: VkResult {result}"));
        }

        let image = self.images[image_index as usize];
        let out_layout = render(
            image,
            self.image_format,
            self.extent.width,
            self.extent.height,
            self.acquire_semaphore,
            self.render_done_semaphore,
        )?;

        unsafe {
            (self.fns.reset_command_buffer)(self.command_buffer, 0);
            let begin_info = VkCommandBufferBeginInfo {
                s_type: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
                p_inheritance_info: ptr::null(),
            };
            let result = (self.fns.begin_command_buffer)(self.command_buffer, &begin_info);
            if result != VK_SUCCESS {
                return Err(format!("vkBeginCommandBuffer failed: VkResult {result}"));
            }
            let barrier = VkImageMemoryBarrier {
                s_type: VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
                p_next: ptr::null(),
                src_access_mask: VK_ACCESS_MEMORY_READ_BIT | VK_ACCESS_MEMORY_WRITE_BIT,
                dst_access_mask: 0,
                old_layout: out_layout,
                new_layout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
                src_queue_family_index: VK_QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: VK_QUEUE_FAMILY_IGNORED,
                image,
                subresource_range: VkImageSubresourceRange {
                    aspect_mask: VK_IMAGE_ASPECT_COLOR_BIT,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
            };
            (self.fns.cmd_pipeline_barrier)(
                self.command_buffer,
                VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
                VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
                0,
                0,
                ptr::null(),
                0,
                ptr::null(),
                1,
                &barrier,
            );
            let result = (self.fns.end_command_buffer)(self.command_buffer);
            if result != VK_SUCCESS {
                return Err(format!("vkEndCommandBuffer failed: VkResult {result}"));
            }

            let wait_stage: u32 = VK_PIPELINE_STAGE_ALL_COMMANDS_BIT;
            let submit_info = VkSubmitInfo {
                s_type: VK_STRUCTURE_TYPE_SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: 1,
                p_wait_semaphores: &self.render_done_semaphore,
                p_wait_dst_stage_mask: &wait_stage,
                command_buffer_count: 1,
                p_command_buffers: &self.command_buffer,
                signal_semaphore_count: 1,
                p_signal_semaphores: &self.transition_semaphore,
            };
            let result = (self.fns.queue_submit)(self.queue, 1, &submit_info, self.in_flight_fence);
            if result != VK_SUCCESS {
                return Err(format!("vkQueueSubmit failed: VkResult {result}"));
            }
        }

        let present_info = VkPresentInfoKHR {
            s_type: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.transition_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.swapchain,
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };
        let result = unsafe { (self.fns.queue_present_khr)(self.queue, &present_info) };
        if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            return Err(format!("vkQueuePresentKHR failed: VkResult {result}"));
        }
        Ok(())
    }
}
