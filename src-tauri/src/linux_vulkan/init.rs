use super::*;

impl VulkanContext {
    pub fn new(native_surface: NativeSurface, width: i32, height: i32) -> Result<Self, String> {
        let module = dlopen_first(&["libvulkan.so.1", "libvulkan.so"])
            .ok_or("libvulkan.so.1 not found (no Vulkan-capable driver installed?)")?;

        let get_instance_proc_addr: PfnGetInstanceProcAddr =
            unsafe { std::mem::transmute(dlsym_typed(module, "vkGetInstanceProcAddr")?) };
        let create_instance: PfnCreateInstance = unsafe {
            std::mem::transmute(get_instance_proc(
                get_instance_proc_addr,
                ptr::null_mut(),
                "vkCreateInstance",
            )?)
        };

        let app_name = CString::new("fluxa-desktop").unwrap();
        let app_info = VkApplicationInfo {
            s_type: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: 0,
            p_engine_name: app_name.as_ptr(),
            engine_version: 0,
            api_version: VK_API_VERSION_1_3,
        };
        let surface_ext_name = match native_surface {
            NativeSurface::Xlib { .. } => "VK_KHR_xlib_surface",
            NativeSurface::Wayland { .. } => "VK_KHR_wayland_surface",
        };
        let extensions = [
            CString::new("VK_KHR_surface").unwrap(),
            CString::new(surface_ext_name).unwrap(),
            CString::new("VK_EXT_swapchain_colorspace").unwrap(),
        ];
        let extension_ptrs: Vec<*const i8> = extensions.iter().map(|e| e.as_ptr()).collect();
        let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        let want_validation = std::env::var_os("FLUXA_VULKAN_VALIDATION").is_some();
        let layer_ptrs: Vec<*const i8> = if want_validation {
            vec![validation_layer.as_ptr()]
        } else {
            vec![]
        };
        let instance_create_info = VkInstanceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: 0,
            p_application_info: &app_info,
            enabled_layer_count: layer_ptrs.len() as u32,
            pp_enabled_layer_names: layer_ptrs.as_ptr(),
            enabled_extension_count: extension_ptrs.len() as u32,
            pp_enabled_extension_names: extension_ptrs.as_ptr(),
        };

        let mut instance: VkInstance = ptr::null_mut();
        let result = unsafe { create_instance(&instance_create_info, ptr::null(), &mut instance) };
        if result != VK_SUCCESS {
            return Err(format!("vkCreateInstance failed: VkResult {result}"));
        }

        macro_rules! iproc {
            ($name:expr_2021) => {
                unsafe {
                    std::mem::transmute(get_instance_proc(get_instance_proc_addr, instance, $name)?)
                }
            };
        }
        let destroy_instance: PfnDestroyInstance = iproc!("vkDestroyInstance");
        let enumerate_physical_devices: PfnEnumeratePhysicalDevices =
            iproc!("vkEnumeratePhysicalDevices");
        let get_physical_device_properties: PfnGetPhysicalDeviceProperties =
            iproc!("vkGetPhysicalDeviceProperties");
        let get_queue_family_properties: PfnGetPhysicalDeviceQueueFamilyProperties =
            iproc!("vkGetPhysicalDeviceQueueFamilyProperties");
        let get_surface_support_khr: PfnGetPhysicalDeviceSurfaceSupportKHR =
            iproc!("vkGetPhysicalDeviceSurfaceSupportKHR");
        let get_surface_capabilities_khr: PfnGetPhysicalDeviceSurfaceCapabilitiesKHR =
            iproc!("vkGetPhysicalDeviceSurfaceCapabilitiesKHR");
        let get_surface_formats_khr: PfnGetPhysicalDeviceSurfaceFormatsKHR =
            iproc!("vkGetPhysicalDeviceSurfaceFormatsKHR");
        let get_surface_present_modes_khr: PfnGetPhysicalDeviceSurfacePresentModesKHR =
            iproc!("vkGetPhysicalDeviceSurfacePresentModesKHR");
        let destroy_surface_khr: PfnDestroySurfaceKHR = iproc!("vkDestroySurfaceKHR");
        let create_device: PfnCreateDevice = iproc!("vkCreateDevice");
        let enumerate_device_extension_properties: PfnEnumerateDeviceExtensionProperties =
            iproc!("vkEnumerateDeviceExtensionProperties");

        let mut surface: VkSurfaceKHR = 0;
        let mut owned_xlib_display: *mut c_void = ptr::null_mut();
        let result = match native_surface {
            NativeSurface::Xlib { display, window } => {
                let create_xlib_surface_khr: PfnCreateXlibSurfaceKHR =
                    iproc!("vkCreateXlibSurfaceKHR");
                // The render loop runs on its own thread, but GTK's Display
                // connection is not thread-safe (XInitThreads is never called).
                // A private connection keeps Mesa's WSI traffic off GTK's.
                owned_xlib_display = unsafe { x11::xlib::XOpenDisplay(ptr::null()) as *mut c_void };
                let dpy = if owned_xlib_display.is_null() {
                    display
                } else {
                    owned_xlib_display
                };
                let create_info = VkXlibSurfaceCreateInfoKHR {
                    s_type: VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
                    p_next: ptr::null(),
                    flags: 0,
                    dpy,
                    window,
                };
                unsafe {
                    create_xlib_surface_khr(instance, &create_info, ptr::null(), &mut surface)
                }
            }
            NativeSurface::Wayland {
                display,
                surface: wl_surface,
            } => {
                let create_wayland_surface_khr: PfnCreateWaylandSurfaceKHR =
                    iproc!("vkCreateWaylandSurfaceKHR");
                let create_info = VkWaylandSurfaceCreateInfoKHR {
                    s_type: VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
                    p_next: ptr::null(),
                    flags: 0,
                    display,
                    surface: wl_surface,
                };
                unsafe {
                    create_wayland_surface_khr(instance, &create_info, ptr::null(), &mut surface)
                }
            }
        };
        if result != VK_SUCCESS {
            unsafe { destroy_instance(instance, ptr::null()) };
            return Err(format!("vkCreate*SurfaceKHR failed: VkResult {result}"));
        }

        let mut device_count: u32 = 0;
        unsafe { enumerate_physical_devices(instance, &mut device_count, ptr::null_mut()) };
        if device_count == 0 {
            unsafe {
                destroy_surface_khr(instance, surface, ptr::null());
                destroy_instance(instance, ptr::null());
            }
            return Err("vkEnumeratePhysicalDevices found no GPUs".into());
        }
        let mut physical_devices = vec![ptr::null_mut(); device_count as usize];
        unsafe {
            enumerate_physical_devices(instance, &mut device_count, physical_devices.as_mut_ptr())
        };

        const VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU: u32 = 2;
        const VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU: u32 = 1;
        const VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU: u32 = 3;
        fn device_type_rank(device_type: u32) -> u32 {
            match device_type {
                VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU => 0,
                VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU => 1,
                VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU => 2,
                _ => 3,
            }
        }

        let mut candidates: Vec<(VkPhysicalDevice, u32, u32)> = Vec::new();
        for &phys_device in &physical_devices {
            let mut family_count: u32 = 0;
            unsafe { get_queue_family_properties(phys_device, &mut family_count, ptr::null_mut()) };
            let mut families: Vec<VkQueueFamilyProperties> = (0..family_count)
                .map(|_| VkQueueFamilyProperties {
                    queue_flags: 0,
                    queue_count: 0,
                    timestamp_valid_bits: 0,
                    min_image_transfer_granularity: [0; 3],
                })
                .collect();
            unsafe {
                get_queue_family_properties(phys_device, &mut family_count, families.as_mut_ptr())
            };
            for (index, family) in families.iter().enumerate() {
                if family.queue_flags & VK_QUEUE_GRAPHICS_BIT == 0 {
                    continue;
                }
                let mut present_supported: u32 = 0;
                unsafe {
                    get_surface_support_khr(
                        phys_device,
                        index as u32,
                        surface,
                        &mut present_supported,
                    )
                };
                if present_supported != 0 {
                    let mut props = [0u8; 1024];
                    unsafe {
                        get_physical_device_properties(
                            phys_device,
                            props.as_mut_ptr() as *mut c_void,
                        )
                    };
                    let device_type = u32::from_ne_bytes(props[16..20].try_into().unwrap());
                    candidates.push((phys_device, index as u32, device_type));
                    break;
                }
            }
        }
        candidates.sort_by_key(|&(_, _, device_type)| device_type_rank(device_type));
        let chosen = candidates
            .first()
            .map(|&(phys_device, queue_family_index, _)| (phys_device, queue_family_index));
        let Some((phys_device, queue_family_index)) = chosen else {
            unsafe {
                destroy_surface_khr(instance, surface, ptr::null());
                destroy_instance(instance, ptr::null());
            }
            return Err(
                "no Vulkan queue family supports both graphics and presenting to this window"
                    .to_string(),
            );
        };

        let queue_priority: f32 = 1.0;
        let queue_create_info = VkDeviceQueueCreateInfo {
            s_type: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: 0,
            queue_family_index,
            queue_count: 1,
            p_queue_priorities: &queue_priority,
        };
        let mut ext_count: u32 = 0;
        unsafe {
            enumerate_device_extension_properties(
                phys_device,
                ptr::null(),
                &mut ext_count,
                ptr::null_mut(),
            )
        };
        let mut supported_extensions = vec![
            VkExtensionProperties {
                extension_name: [0; 256],
                spec_version: 0,
            };
            ext_count as usize
        ];
        unsafe {
            enumerate_device_extension_properties(
                phys_device,
                ptr::null(),
                &mut ext_count,
                supported_extensions.as_mut_ptr(),
            )
        };
        let device_extensions: Vec<CString> = supported_extensions
            .iter()
            .filter_map(|ext| {
                let nul = ext.extension_name.iter().position(|&b| b == 0)?;
                CString::new(&ext.extension_name[..nul]).ok()
            })
            .collect();
        let device_extension_ptrs: Vec<*const i8> =
            device_extensions.iter().map(|e| e.as_ptr()).collect();
        let mut synchronization2_features = VkPhysicalDeviceSynchronization2Features {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            p_next: ptr::null_mut(),
            synchronization2: 1,
        };
        let mut timeline_semaphore_features = VkPhysicalDeviceTimelineSemaphoreFeatures {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES,
            p_next: &mut synchronization2_features as *mut _ as *mut c_void,
            timeline_semaphore: 1,
        };
        let device_create_info = VkDeviceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            p_next: &mut timeline_semaphore_features as *mut _ as *const c_void,
            flags: 0,
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: device_extension_ptrs.len() as u32,
            pp_enabled_extension_names: device_extension_ptrs.as_ptr(),
            p_enabled_features: ptr::null(),
        };
        let mut device: VkDevice = ptr::null_mut();
        let result =
            unsafe { create_device(phys_device, &device_create_info, ptr::null(), &mut device) };
        if result != VK_SUCCESS {
            unsafe {
                destroy_surface_khr(instance, surface, ptr::null());
                destroy_instance(instance, ptr::null());
            }
            return Err(format!("vkCreateDevice failed: VkResult {result}"));
        }

        macro_rules! dproc {
            ($name:expr_2021) => {
                unsafe {
                    std::mem::transmute(get_instance_proc(get_instance_proc_addr, instance, $name)?)
                }
            };
        }
        let destroy_device: PfnDestroyDevice = dproc!("vkDestroyDevice");
        let get_device_queue: PfnGetDeviceQueue = dproc!("vkGetDeviceQueue");
        let create_swapchain_khr: PfnCreateSwapchainKHR = dproc!("vkCreateSwapchainKHR");
        let destroy_swapchain_khr: PfnDestroySwapchainKHR = dproc!("vkDestroySwapchainKHR");
        let get_swapchain_images_khr: PfnGetSwapchainImagesKHR = dproc!("vkGetSwapchainImagesKHR");
        let acquire_next_image_khr: PfnAcquireNextImageKHR = dproc!("vkAcquireNextImageKHR");
        let queue_present_khr: PfnQueuePresentKHR = dproc!("vkQueuePresentKHR");
        let create_semaphore: PfnCreateSemaphore = dproc!("vkCreateSemaphore");
        let destroy_semaphore: PfnDestroySemaphore = dproc!("vkDestroySemaphore");
        let create_fence: PfnCreateFence = dproc!("vkCreateFence");
        let destroy_fence: PfnDestroyFence = dproc!("vkDestroyFence");
        let wait_for_fences: PfnWaitForFences = dproc!("vkWaitForFences");
        let reset_fences: PfnResetFences = dproc!("vkResetFences");
        let device_wait_idle: PfnDeviceWaitIdle = dproc!("vkDeviceWaitIdle");
        let create_command_pool: PfnCreateCommandPool = dproc!("vkCreateCommandPool");
        let destroy_command_pool: PfnDestroyCommandPool = dproc!("vkDestroyCommandPool");
        let allocate_command_buffers: PfnAllocateCommandBuffers =
            dproc!("vkAllocateCommandBuffers");
        let reset_command_buffer: PfnResetCommandBuffer = dproc!("vkResetCommandBuffer");
        let begin_command_buffer: PfnBeginCommandBuffer = dproc!("vkBeginCommandBuffer");
        let end_command_buffer: PfnEndCommandBuffer = dproc!("vkEndCommandBuffer");
        let cmd_pipeline_barrier: PfnCmdPipelineBarrier = dproc!("vkCmdPipelineBarrier");
        let queue_submit: PfnQueueSubmit = dproc!("vkQueueSubmit");

        let mut queue: VkQueue = ptr::null_mut();
        unsafe { get_device_queue(device, queue_family_index, 0, &mut queue) };

        let fns = VkFns {
            get_instance_proc_addr,
            destroy_instance,
            get_physical_device_surface_capabilities_khr: get_surface_capabilities_khr,
            get_physical_device_surface_formats_khr: get_surface_formats_khr,
            get_physical_device_surface_present_modes_khr: get_surface_present_modes_khr,
            destroy_surface_khr,
            destroy_device,
            create_swapchain_khr,
            destroy_swapchain_khr,
            get_swapchain_images_khr,
            acquire_next_image_khr,
            queue_present_khr,
            create_semaphore,
            destroy_semaphore,
            create_fence,
            destroy_fence,
            wait_for_fences,
            reset_fences,
            device_wait_idle,
            create_command_pool,
            destroy_command_pool,
            allocate_command_buffers,
            reset_command_buffer,
            begin_command_buffer,
            end_command_buffer,
            cmd_pipeline_barrier,
            queue_submit,
        };

        let mut ctx = Self {
            fns,
            instance,
            phys_device,
            device,
            queue,
            queue_family_index,
            surface,
            swapchain: 0,
            images: Vec::new(),
            image_format: VK_FORMAT_B8G8R8A8_UNORM,
            image_usage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            extent: VkExtent2D {
                width: width.max(2) as u32,
                height: height.max(2) as u32,
            },
            acquire_semaphore: 0,
            render_done_semaphore: 0,
            transition_semaphore: 0,
            in_flight_fence: 0,
            command_pool: ptr::null_mut(),
            command_buffer: ptr::null_mut(),
            hdr: AtomicBool::new(false),
            enabled_device_extensions: device_extensions,
            owned_xlib_display,
        };
        ctx.create_swapchain(width.max(2) as u32, height.max(2) as u32)?;
        ctx.create_semaphores()?;
        ctx.create_command_buffer()?;
        Ok(ctx)
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            (self.fns.device_wait_idle)(self.device);
            if self.acquire_semaphore != 0 {
                (self.fns.destroy_semaphore)(self.device, self.acquire_semaphore, ptr::null());
            }
            if self.render_done_semaphore != 0 {
                (self.fns.destroy_semaphore)(self.device, self.render_done_semaphore, ptr::null());
            }
            if self.transition_semaphore != 0 {
                (self.fns.destroy_semaphore)(self.device, self.transition_semaphore, ptr::null());
            }
            if self.in_flight_fence != 0 {
                (self.fns.destroy_fence)(self.device, self.in_flight_fence, ptr::null());
            }
            if !self.command_pool.is_null() {
                (self.fns.destroy_command_pool)(self.device, self.command_pool, ptr::null());
            }
            if self.swapchain != 0 {
                (self.fns.destroy_swapchain_khr)(self.device, self.swapchain, ptr::null());
            }
            (self.fns.destroy_device)(self.device, ptr::null());
            (self.fns.destroy_surface_khr)(self.instance, self.surface, ptr::null());
            (self.fns.destroy_instance)(self.instance, ptr::null());
            if !self.owned_xlib_display.is_null() {
                x11::xlib::XCloseDisplay(self.owned_xlib_display as *mut x11::xlib::Display);
            }
        }
    }
}
