#![allow(dead_code)]

use std::ffi::{CString, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

pub type VkInstance = *mut c_void;
pub type VkPhysicalDevice = *mut c_void;
pub type VkDevice = *mut c_void;
pub type VkQueue = *mut c_void;
pub type VkSurfaceKHR = u64;
pub type VkSwapchainKHR = u64;
pub type VkSemaphore = u64;
pub type VkImage = u64;
pub type VkFence = u64;
pub type VkCommandPool = *mut c_void;
pub type VkCommandBuffer = *mut c_void;
pub type VkResult = i32;

pub const VK_SUCCESS: VkResult = 0;
pub const VK_ERROR_OUT_OF_DATE_KHR: VkResult = -1000001004;
pub const VK_SUBOPTIMAL_KHR: VkResult = 1000001003;
pub const VK_STRUCTURE_TYPE_APPLICATION_INFO: i32 = 0;
pub const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: i32 = 1;
pub const VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO: i32 = 2;
pub const VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO: i32 = 3;
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2: i32 = 2;
pub const VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO: i32 = 9;
pub const VK_STRUCTURE_TYPE_FENCE_CREATE_INFO: i32 = 8;
pub const VK_FENCE_CREATE_SIGNALED_BIT: u32 = 1;
pub const VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT: u32 = 2;
pub const VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR: i32 = 1000001000;
pub const VK_STRUCTURE_TYPE_PRESENT_INFO_KHR: i32 = 1000001001;
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES: i32 = 1000207000;
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES: i32 = 1000314007;
pub const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_2_FEATURES: i32 = 51;
pub const VK_STRUCTURE_TYPE_SUBMIT_INFO: i32 = 4;
pub const VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO: i32 = 39;
pub const VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO: i32 = 40;
pub const VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO: i32 = 42;
pub const VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER: i32 = 45;

pub const VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR: u32 = 0x1;
pub const VK_QUEUE_GRAPHICS_BIT: u32 = 0x1;
pub const VK_IMAGE_USAGE_TRANSFER_DST_BIT: u32 = 0x2;
pub const VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT: u32 = 0x10;
pub const VK_FORMAT_B8G8R8A8_UNORM: i32 = 44;
pub const VK_FORMAT_B8G8R8A8_SRGB: i32 = 50;
pub const VK_FORMAT_R16G16B16A16_SFLOAT: i32 = 97;
pub const VK_COLOR_SPACE_SRGB_NONLINEAR_KHR: i32 = 0;
pub const VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT: i32 = 1000104002;
pub const VK_SHARING_MODE_EXCLUSIVE: i32 = 0;
pub const VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR: u32 = 0x1;
pub const VK_PRESENT_MODE_FIFO_KHR: i32 = 2;
pub const VK_API_VERSION_1_0: u32 = 1 << 22;
pub const VK_API_VERSION_1_2: u32 = (1 << 22) | (2 << 12);
pub const VK_API_VERSION_1_3: u32 = (1 << 22) | (3 << 12);
pub const VK_API_VERSION_1_4: u32 = (1 << 22) | (4 << 12);
pub const VK_IMAGE_LAYOUT_PRESENT_SRC_KHR: i32 = 1000001002;
pub const VK_IMAGE_ASPECT_COLOR_BIT: u32 = 0x1;
pub const VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT: u32 = 0x00002000;
pub const VK_PIPELINE_STAGE_ALL_COMMANDS_BIT: u32 = 0x00010000;
pub const VK_ACCESS_MEMORY_READ_BIT: u32 = 0x00008000;
pub const VK_ACCESS_MEMORY_WRITE_BIT: u32 = 0x00010000;
pub const VK_COMMAND_BUFFER_LEVEL_PRIMARY: i32 = 0;
pub const VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT: u32 = 0x1;
pub const VK_QUEUE_FAMILY_IGNORED: u32 = 0xFFFFFFFF;

#[repr(C)]
pub struct VkApplicationInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub p_application_name: *const i8,
    pub application_version: u32,
    pub p_engine_name: *const i8,
    pub engine_version: u32,
    pub api_version: u32,
}

#[repr(C)]
pub struct VkInstanceCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub p_application_info: *const VkApplicationInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const i8,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const i8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VkExtent2D {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct VkSurfaceCapabilitiesKHR {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: VkExtent2D,
    pub min_image_extent: VkExtent2D,
    pub max_image_extent: VkExtent2D,
    pub max_image_array_layers: u32,
    pub supported_transforms: u32,
    pub current_transform: u32,
    pub supported_composite_alpha: u32,
    pub supported_usage_flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSurfaceFormatKHR {
    pub format: i32,
    pub color_space: i32,
}

#[repr(C)]
pub struct VkQueueFamilyProperties {
    pub queue_flags: u32,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: [u32; 3],
}

#[repr(C)]
#[derive(Clone)]
pub struct VkExtensionProperties {
    pub extension_name: [u8; 256],
    pub spec_version: u32,
}

#[repr(C)]
pub struct VkDeviceQueueCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub queue_family_index: u32,
    pub queue_count: u32,
    pub p_queue_priorities: *const f32,
}

#[repr(C)]
pub struct VkDeviceCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub queue_create_info_count: u32,
    pub p_queue_create_infos: *const VkDeviceQueueCreateInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const i8,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const i8,
    pub p_enabled_features: *const c_void,
}

#[repr(C)]
pub struct VkPhysicalDeviceFeatures2 {
    pub s_type: i32,
    pub p_next: *mut c_void,
    // VkPhysicalDeviceFeatures is 55 VkBool32 fields. We do not need to
    // inspect the legacy feature bits here, but the storage must be present
    // so the pNext chain has the correct Vulkan ABI layout.
    pub features: [u32; 55],
}

#[repr(C)]
pub struct VkPhysicalDeviceTimelineSemaphoreFeatures {
    pub s_type: i32,
    pub p_next: *mut c_void,
    pub timeline_semaphore: u32,
}

// libplacebo imports the required timeline feature through the Vulkan 1.2
// core feature chain. Keep this ABI-compatible with VkPhysicalDeviceVulkan12Features
// instead of advertising the promoted extension struct alone.
#[repr(C)]
#[derive(Default)]
pub struct VkPhysicalDeviceVulkan12Features {
    pub s_type: i32,
    pub p_next: *mut c_void,
    pub sampler_mirror_clamp_to_edge: u32,
    pub draw_indirect_count: u32,
    pub storage_buffer8_bit_access: u32,
    pub uniform_and_storage_buffer8_bit_access: u32,
    pub storage_push_constant8: u32,
    pub shader_buffer_int64_atomics: u32,
    pub shader_shared_int64_atomics: u32,
    pub shader_float16: u32,
    pub shader_int8: u32,
    pub descriptor_indexing: u32,
    pub shader_input_attachment_array_dynamic_indexing: u32,
    pub shader_uniform_texel_buffer_array_dynamic_indexing: u32,
    pub shader_storage_texel_buffer_array_dynamic_indexing: u32,
    pub shader_uniform_buffer_array_non_uniform_indexing: u32,
    pub shader_sampled_image_array_non_uniform_indexing: u32,
    pub shader_storage_buffer_array_non_uniform_indexing: u32,
    pub shader_storage_image_array_non_uniform_indexing: u32,
    pub shader_input_attachment_array_non_uniform_indexing: u32,
    pub shader_uniform_texel_buffer_array_non_uniform_indexing: u32,
    pub shader_storage_texel_buffer_array_non_uniform_indexing: u32,
    pub descriptor_binding_uniform_buffer_update_after_bind: u32,
    pub descriptor_binding_sampled_image_update_after_bind: u32,
    pub descriptor_binding_storage_image_update_after_bind: u32,
    pub descriptor_binding_storage_buffer_update_after_bind: u32,
    pub descriptor_binding_uniform_texel_buffer_update_after_bind: u32,
    pub descriptor_binding_storage_texel_buffer_update_after_bind: u32,
    pub descriptor_binding_update_unused_while_pending: u32,
    pub descriptor_binding_partially_bound: u32,
    pub descriptor_binding_variable_descriptor_count: u32,
    pub runtime_descriptor_array: u32,
    pub sampler_filter_minmax: u32,
    pub scalar_block_layout: u32,
    pub imageless_framebuffer: u32,
    pub uniform_buffer_standard_layout: u32,
    pub shader_subgroup_extended_types: u32,
    pub separate_depth_stencil_layouts: u32,
    pub host_query_reset: u32,
    pub timeline_semaphore: u32,
    pub buffer_device_address: u32,
    pub buffer_device_address_capture_replay: u32,
    pub buffer_device_address_multi_device: u32,
    pub vulkan_memory_model: u32,
    pub vulkan_memory_model_device_scope: u32,
    pub vulkan_memory_model_availability_visibility_chains: u32,
    pub shader_output_viewport_index: u32,
    pub shader_output_layer: u32,
    pub subgroup_broadcast_dynamic_id: u32,
}

#[repr(C)]
pub struct VkPhysicalDeviceSynchronization2Features {
    pub s_type: i32,
    pub p_next: *mut c_void,
    pub synchronization2: u32,
}

#[repr(C)]
pub struct VkSwapchainCreateInfoKHR {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub surface: VkSurfaceKHR,
    pub min_image_count: u32,
    pub image_format: i32,
    pub image_color_space: i32,
    pub image_extent: VkExtent2D,
    pub image_array_layers: u32,
    pub image_usage: u32,
    pub image_sharing_mode: i32,
    pub queue_family_index_count: u32,
    pub p_queue_family_indices: *const u32,
    pub pre_transform: u32,
    pub composite_alpha: u32,
    pub present_mode: i32,
    pub clipped: u32,
    pub old_swapchain: VkSwapchainKHR,
}

#[repr(C)]
pub struct VkSemaphoreCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
}

#[repr(C)]
pub struct VkPresentInfoKHR {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub wait_semaphore_count: u32,
    pub p_wait_semaphores: *const VkSemaphore,
    pub swapchain_count: u32,
    pub p_swapchains: *const VkSwapchainKHR,
    pub p_image_indices: *const u32,
    pub p_results: *mut VkResult,
}

#[repr(C)]
pub struct VkImageSubresourceRange {
    pub aspect_mask: u32,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

#[repr(C)]
pub struct VkImageMemoryBarrier {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub src_access_mask: u32,
    pub dst_access_mask: u32,
    pub old_layout: i32,
    pub new_layout: i32,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: VkImage,
    pub subresource_range: VkImageSubresourceRange,
}

#[repr(C)]
pub struct VkCommandPoolCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub queue_family_index: u32,
}

#[repr(C)]
pub struct VkCommandBufferAllocateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub command_pool: VkCommandPool,
    pub level: i32,
    pub command_buffer_count: u32,
}

#[repr(C)]
pub struct VkCommandBufferBeginInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub p_inheritance_info: *const c_void,
}

#[repr(C)]
pub struct VkSubmitInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub wait_semaphore_count: u32,
    pub p_wait_semaphores: *const VkSemaphore,
    pub p_wait_dst_stage_mask: *const u32,
    pub command_buffer_count: u32,
    pub p_command_buffers: *const VkCommandBuffer,
    pub signal_semaphore_count: u32,
    pub p_signal_semaphores: *const VkSemaphore,
}

pub type PfnGetInstanceProcAddr =
    unsafe extern "system" fn(instance: VkInstance, name: *const i8) -> *mut c_void;
pub type PfnEnumerateInstanceExtensionProperties =
    unsafe extern "system" fn(*const i8, *mut u32, *mut VkExtensionProperties) -> VkResult;
pub type PfnCreateInstance = unsafe extern "system" fn(
    *const VkInstanceCreateInfo,
    *const c_void,
    *mut VkInstance,
) -> VkResult;
pub type PfnDestroyInstance = unsafe extern "system" fn(VkInstance, *const c_void);
pub type PfnEnumeratePhysicalDevices =
    unsafe extern "system" fn(VkInstance, *mut u32, *mut VkPhysicalDevice) -> VkResult;
pub type PfnGetPhysicalDeviceProperties = unsafe extern "system" fn(VkPhysicalDevice, *mut c_void);
pub type PfnGetPhysicalDeviceFeatures2 = unsafe extern "system" fn(
    VkPhysicalDevice,
    *mut VkPhysicalDeviceFeatures2,
);
pub type PfnEnumerateDeviceExtensionProperties = unsafe extern "system" fn(
    VkPhysicalDevice,
    *const i8,
    *mut u32,
    *mut VkExtensionProperties,
) -> VkResult;
pub type PfnGetPhysicalDeviceQueueFamilyProperties =
    unsafe extern "system" fn(VkPhysicalDevice, *mut u32, *mut VkQueueFamilyProperties);
pub type PfnGetPhysicalDeviceSurfaceSupportKHR =
    unsafe extern "system" fn(VkPhysicalDevice, u32, VkSurfaceKHR, *mut u32) -> VkResult;
pub type PfnGetPhysicalDeviceSurfaceCapabilitiesKHR = unsafe extern "system" fn(
    VkPhysicalDevice,
    VkSurfaceKHR,
    *mut VkSurfaceCapabilitiesKHR,
) -> VkResult;
pub type PfnGetPhysicalDeviceSurfaceFormatsKHR = unsafe extern "system" fn(
    VkPhysicalDevice,
    VkSurfaceKHR,
    *mut u32,
    *mut VkSurfaceFormatKHR,
) -> VkResult;
pub type PfnDestroySurfaceKHR = unsafe extern "system" fn(VkInstance, VkSurfaceKHR, *const c_void);
pub type PfnCreateDevice = unsafe extern "system" fn(
    VkPhysicalDevice,
    *const VkDeviceCreateInfo,
    *const c_void,
    *mut VkDevice,
) -> VkResult;
pub type PfnDestroyDevice = unsafe extern "system" fn(VkDevice, *const c_void);
pub type PfnGetDeviceQueue = unsafe extern "system" fn(VkDevice, u32, u32, *mut VkQueue);
pub type PfnCreateSwapchainKHR = unsafe extern "system" fn(
    VkDevice,
    *const VkSwapchainCreateInfoKHR,
    *const c_void,
    *mut VkSwapchainKHR,
) -> VkResult;
pub type PfnDestroySwapchainKHR =
    unsafe extern "system" fn(VkDevice, VkSwapchainKHR, *const c_void);
pub type PfnGetSwapchainImagesKHR =
    unsafe extern "system" fn(VkDevice, VkSwapchainKHR, *mut u32, *mut VkImage) -> VkResult;
pub type PfnAcquireNextImageKHR = unsafe extern "system" fn(
    VkDevice,
    VkSwapchainKHR,
    u64,
    VkSemaphore,
    *mut c_void,
    *mut u32,
) -> VkResult;
pub type PfnQueuePresentKHR =
    unsafe extern "system" fn(VkQueue, *const VkPresentInfoKHR) -> VkResult;
pub type PfnCreateSemaphore = unsafe extern "system" fn(
    VkDevice,
    *const VkSemaphoreCreateInfo,
    *const c_void,
    *mut VkSemaphore,
) -> VkResult;
pub type PfnDestroySemaphore = unsafe extern "system" fn(VkDevice, VkSemaphore, *const c_void);
pub type PfnDeviceWaitIdle = unsafe extern "system" fn(VkDevice) -> VkResult;
pub type PfnCreateCommandPool = unsafe extern "system" fn(
    VkDevice,
    *const VkCommandPoolCreateInfo,
    *const c_void,
    *mut VkCommandPool,
) -> VkResult;
pub type PfnDestroyCommandPool = unsafe extern "system" fn(VkDevice, VkCommandPool, *const c_void);
pub type PfnAllocateCommandBuffers = unsafe extern "system" fn(
    VkDevice,
    *const VkCommandBufferAllocateInfo,
    *mut VkCommandBuffer,
) -> VkResult;
pub type PfnResetCommandBuffer = unsafe extern "system" fn(VkCommandBuffer, u32) -> VkResult;
pub type PfnBeginCommandBuffer =
    unsafe extern "system" fn(VkCommandBuffer, *const VkCommandBufferBeginInfo) -> VkResult;
pub type PfnEndCommandBuffer = unsafe extern "system" fn(VkCommandBuffer) -> VkResult;
pub type PfnCmdPipelineBarrier = unsafe extern "system" fn(
    VkCommandBuffer,
    u32,
    u32,
    u32,
    u32,
    *const c_void,
    u32,
    *const c_void,
    u32,
    *const VkImageMemoryBarrier,
);
pub type PfnQueueSubmit =
    unsafe extern "system" fn(VkQueue, u32, *const VkSubmitInfo, VkFence) -> VkResult;
pub type PfnEnumerateInstanceVersion = unsafe extern "system" fn(*mut u32) -> VkResult;
pub type PfnCreateFence = unsafe extern "system" fn(
    VkDevice,
    *const VkFenceCreateInfo,
    *const c_void,
    *mut VkFence,
) -> VkResult;
pub type PfnDestroyFence = unsafe extern "system" fn(VkDevice, VkFence, *const c_void);
pub type PfnWaitForFences =
    unsafe extern "system" fn(VkDevice, u32, *const VkFence, u32, u64) -> VkResult;
pub type PfnResetFences = unsafe extern "system" fn(VkDevice, u32, *const VkFence) -> VkResult;

#[repr(C)]
pub struct VkFenceCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
}

struct VkFns {
    get_instance_proc_addr: PfnGetInstanceProcAddr,
    destroy_instance: PfnDestroyInstance,
    get_physical_device_surface_capabilities_khr: PfnGetPhysicalDeviceSurfaceCapabilitiesKHR,
    get_physical_device_surface_formats_khr: PfnGetPhysicalDeviceSurfaceFormatsKHR,
    destroy_surface_khr: PfnDestroySurfaceKHR,
    destroy_device: PfnDestroyDevice,
    get_device_queue: PfnGetDeviceQueue,
    create_swapchain_khr: PfnCreateSwapchainKHR,
    destroy_swapchain_khr: PfnDestroySwapchainKHR,
    get_swapchain_images_khr: PfnGetSwapchainImagesKHR,
    acquire_next_image_khr: PfnAcquireNextImageKHR,
    queue_present_khr: PfnQueuePresentKHR,
    create_semaphore: PfnCreateSemaphore,
    destroy_semaphore: PfnDestroySemaphore,
    create_fence: PfnCreateFence,
    destroy_fence: PfnDestroyFence,
    wait_for_fences: PfnWaitForFences,
    reset_fences: PfnResetFences,
    device_wait_idle: PfnDeviceWaitIdle,
    create_command_pool: PfnCreateCommandPool,
    destroy_command_pool: PfnDestroyCommandPool,
    allocate_command_buffers: PfnAllocateCommandBuffers,
    reset_command_buffer: PfnResetCommandBuffer,
    begin_command_buffer: PfnBeginCommandBuffer,
    end_command_buffer: PfnEndCommandBuffer,
    cmd_pipeline_barrier: PfnCmdPipelineBarrier,
    queue_submit: PfnQueueSubmit,
}

pub fn get_instance_proc(
    get_instance_proc_addr: PfnGetInstanceProcAddr,
    instance: VkInstance,
    name: &str,
) -> Result<*mut c_void, String> {
    let cname = CString::new(name).unwrap();
    let addr = unsafe { get_instance_proc_addr(instance, cname.as_ptr()) };
    if addr.is_null() {
        Err(format!("vkGetInstanceProcAddr could not resolve {name}"))
    } else {
        Ok(addr)
    }
}

pub trait VulkanPlatform: Send {
    fn label(&self) -> &'static str;

    fn load_loader(&self) -> Result<PfnGetInstanceProcAddr, String>;

    fn instance_extensions(&self, available: &dyn Fn(&str) -> bool)
    -> Result<Vec<CString>, String>;

    fn instance_flags(&self, _available: &dyn Fn(&str) -> bool) -> u32 {
        0
    }

    fn instance_layers(&self) -> Vec<CString> {
        Vec::new()
    }

    unsafe fn create_surface(
        &self,
        instance: VkInstance,
        get_proc: PfnGetInstanceProcAddr,
    ) -> Result<VkSurfaceKHR, String>;

    fn device_extensions(&self, available: &[String]) -> Result<Vec<CString>, String>;

    fn prefer_hdr(&self) -> bool;
}

fn extension_names(list: &[VkExtensionProperties]) -> Vec<String> {
    list.iter()
        .filter_map(|e| {
            let end = e.extension_name.iter().position(|&c| c == 0)?;
            std::str::from_utf8(&e.extension_name[..end])
                .ok()
                .map(str::to_owned)
        })
        .collect()
}

pub struct VulkanContext {
    platform: Box<dyn VulkanPlatform>,
    fns: VkFns,
    instance: VkInstance,
    phys_device: VkPhysicalDevice,
    device: VkDevice,
    queue: VkQueue,
    queue_family_index: u32,
    surface: VkSurfaceKHR,
    swapchain: VkSwapchainKHR,
    images: Vec<VkImage>,
    image_format: i32,
    image_usage: u32,
    extent: VkExtent2D,
    acquire_semaphore: VkSemaphore,
    render_done_semaphore: VkSemaphore,
    transition_semaphore: VkSemaphore,
    in_flight_fence: VkFence,
    command_pool: VkCommandPool,
    command_buffer: VkCommandBuffer,
    hdr: AtomicBool,
    device_extension_names: Vec<CString>,
}

unsafe impl Send for VulkanContext {}

impl VulkanContext {
    pub fn new(platform: Box<dyn VulkanPlatform>, width: i32, height: i32) -> Result<Self, String> {
        let get_instance_proc_addr = platform.load_loader()?;
        let create_instance: PfnCreateInstance = unsafe {
            std::mem::transmute(get_instance_proc(
                get_instance_proc_addr,
                ptr::null_mut(),
                "vkCreateInstance",
            )?)
        };
        let enumerate_instance_extension_properties: PfnEnumerateInstanceExtensionProperties = unsafe {
            std::mem::transmute(get_instance_proc(
                get_instance_proc_addr,
                ptr::null_mut(),
                "vkEnumerateInstanceExtensionProperties",
            )?)
        };

        let loader_version = match get_instance_proc(
            get_instance_proc_addr,
            ptr::null_mut(),
            "vkEnumerateInstanceVersion",
        ) {
            Ok(addr) => {
                let f: PfnEnumerateInstanceVersion = unsafe { std::mem::transmute(addr) };
                let mut version: u32 = VK_API_VERSION_1_0;
                if unsafe { f(&mut version) } == VK_SUCCESS {
                    version
                } else {
                    VK_API_VERSION_1_0
                }
            }
            Err(_) => VK_API_VERSION_1_0,
        };
        if loader_version < VK_API_VERSION_1_2 {
            return Err(format!(
                "{} Vulkan loader reports {}.{}; libplacebo needs 1.2 or newer",
                platform.label(),
                (loader_version >> 22) & 0x7F,
                (loader_version >> 12) & 0x3FF
            ));
        }
        // Request the highest loader version we understand. The actual
        // physical-device feature/API support is still validated separately
        // below; 1.4 is not forced on a device that only exposes 1.2/1.3.
        let api_version = loader_version.min(VK_API_VERSION_1_4);
        log::info!(
            "{} Vulkan: loader reports {}.{}, requesting {}.{}",
            platform.label(),
            (loader_version >> 22) & 0x7F,
            (loader_version >> 12) & 0x3FF,
            (api_version >> 22) & 0x7F,
            (api_version >> 12) & 0x3FF
        );

        let mut ext_count: u32 = 0;
        unsafe {
            enumerate_instance_extension_properties(ptr::null(), &mut ext_count, ptr::null_mut())
        };
        let mut instance_exts = vec![
            VkExtensionProperties {
                extension_name: [0; 256],
                spec_version: 0
            };
            ext_count as usize
        ];
        unsafe {
            enumerate_instance_extension_properties(
                ptr::null(),
                &mut ext_count,
                instance_exts.as_mut_ptr(),
            )
        };
        let available_instance = extension_names(&instance_exts);
        let has_instance_ext = |name: &str| available_instance.iter().any(|e| e == name);

        let extensions = platform.instance_extensions(&has_instance_ext)?;
        let instance_flags = platform.instance_flags(&has_instance_ext);
        let mut layers = platform.instance_layers();
        if std::env::var_os("FLUXA_VULKAN_VALIDATION").is_some() {
            let validation = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
            if !layers.contains(&validation) {
                layers.push(validation);
            }
            log::info!(
                "{} Vulkan: FLUXA_VULKAN_VALIDATION set, enabling VK_LAYER_KHRONOS_validation",
                platform.label()
            );
        }

        let app_name = CString::new("fluxa-desktop").unwrap();
        let app_info = VkApplicationInfo {
            s_type: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: 0,
            p_engine_name: app_name.as_ptr(),
            engine_version: 0,
            api_version,
        };
        let extension_ptrs: Vec<*const i8> = extensions.iter().map(|e| e.as_ptr()).collect();
        let layer_ptrs: Vec<*const i8> = layers.iter().map(|l| l.as_ptr()).collect();
        let instance_create_info = VkInstanceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: instance_flags,
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
            ($name:expr) => {
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
        let get_physical_device_features2: PfnGetPhysicalDeviceFeatures2 =
            iproc!("vkGetPhysicalDeviceFeatures2");
        let enumerate_device_extension_properties: PfnEnumerateDeviceExtensionProperties =
            iproc!("vkEnumerateDeviceExtensionProperties");
        let get_queue_family_properties: PfnGetPhysicalDeviceQueueFamilyProperties =
            iproc!("vkGetPhysicalDeviceQueueFamilyProperties");
        let get_surface_support_khr: PfnGetPhysicalDeviceSurfaceSupportKHR =
            iproc!("vkGetPhysicalDeviceSurfaceSupportKHR");
        let get_surface_capabilities_khr: PfnGetPhysicalDeviceSurfaceCapabilitiesKHR =
            iproc!("vkGetPhysicalDeviceSurfaceCapabilitiesKHR");
        let get_surface_formats_khr: PfnGetPhysicalDeviceSurfaceFormatsKHR =
            iproc!("vkGetPhysicalDeviceSurfaceFormatsKHR");
        let destroy_surface_khr: PfnDestroySurfaceKHR = iproc!("vkDestroySurfaceKHR");
        let create_device: PfnCreateDevice = iproc!("vkCreateDevice");

        let surface = match unsafe { platform.create_surface(instance, get_instance_proc_addr) } {
            Ok(surface) => surface,
            Err(e) => {
                unsafe { destroy_instance(instance, ptr::null()) };
                return Err(e);
            }
        };

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
                "no Vulkan queue family supports both graphics and presenting to this surface"
                    .to_string(),
            );
        };

        {
            let mut props = [0u8; 1024];
            unsafe { get_physical_device_properties(phys_device, props.as_mut_ptr() as *mut c_void) };
            let device_api = u32::from_ne_bytes(props[0..4].try_into().unwrap());
            let end = props[20..276].iter().position(|&c| c == 0).unwrap_or(0);
            let name = String::from_utf8_lossy(&props[20..20 + end]).to_string();
            log::info!(
                "{} Vulkan device: {} api {}.{}.{}",
                platform.label(),
                name,
                (device_api >> 22) & 0x7F,
                (device_api >> 12) & 0x3FF,
                device_api & 0xFFF
            );
        }

        // libplacebo's gpu-next import path requires the device to have been
        // created with its required Vulkan features enabled. Query the
        // selected physical device before creating the logical device so a
        // MoltenVK feature mismatch becomes an explicit, actionable error
        // instead of MPV_ERROR_UNSUPPORTED later in pl_vulkan_import().
        let mut supported_sync2 = VkPhysicalDeviceSynchronization2Features {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            p_next: ptr::null_mut(),
            synchronization2: 0,
        };
        let mut supported_vulkan12 = VkPhysicalDeviceVulkan12Features {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_2_FEATURES,
            p_next: &mut supported_sync2 as *mut _ as *mut c_void,
            ..Default::default()
        };
        let mut supported_features = VkPhysicalDeviceFeatures2 {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2,
            p_next: &mut supported_vulkan12 as *mut _ as *mut c_void,
            features: [0; 55],
        };
        unsafe { get_physical_device_features2(phys_device, &mut supported_features) };

        let missing_features = [
            ("synchronization2", supported_sync2.synchronization2),
            ("hostQueryReset", supported_vulkan12.host_query_reset),
            ("timelineSemaphore", supported_vulkan12.timeline_semaphore),
            ("bufferDeviceAddress", supported_vulkan12.buffer_device_address),
        ]
        .into_iter()
        .filter_map(|(name, supported)| (supported == 0).then_some(name))
        .collect::<Vec<_>>();
        if !missing_features.is_empty() {
            unsafe {
                destroy_surface_khr(instance, surface, ptr::null());
                destroy_instance(instance, ptr::null());
            }
            return Err(format!(
                "selected Vulkan device is missing libplacebo gpu-next required feature(s): {}",
                missing_features.join(", ")
            ));
        }

        let mut dev_ext_count: u32 = 0;
        unsafe {
            enumerate_device_extension_properties(
                phys_device,
                ptr::null(),
                &mut dev_ext_count,
                ptr::null_mut(),
            )
        };
        let mut dev_exts = vec![
            VkExtensionProperties {
                extension_name: [0; 256],
                spec_version: 0
            };
            dev_ext_count as usize
        ];
        unsafe {
            enumerate_device_extension_properties(
                phys_device,
                ptr::null(),
                &mut dev_ext_count,
                dev_exts.as_mut_ptr(),
            )
        };
        let device_extensions = match platform.device_extensions(&extension_names(&dev_exts)) {
            Ok(list) => list,
            Err(e) => {
                unsafe {
                    destroy_surface_khr(instance, surface, ptr::null());
                    destroy_instance(instance, ptr::null());
                }
                return Err(e);
            }
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
        let device_extension_ptrs: Vec<*const i8> =
            device_extensions.iter().map(|e| e.as_ptr()).collect();
        let mut synchronization2_features = VkPhysicalDeviceSynchronization2Features {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            p_next: ptr::null_mut(),
            synchronization2: 1,
        };
        let mut vulkan12_features = VkPhysicalDeviceVulkan12Features {
            s_type: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_2_FEATURES,
            p_next: &mut synchronization2_features as *mut _ as *mut c_void,
            host_query_reset: 1,
            timeline_semaphore: 1,
            buffer_device_address: 1,
            ..Default::default()
        };
        let device_create_info = VkDeviceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            p_next: &mut vulkan12_features as *mut _ as *const c_void,
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
            ($name:expr) => {
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
            destroy_surface_khr,
            destroy_device,
            get_device_queue,
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
            platform,
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
            device_extension_names: device_extensions,
        };
        ctx.create_swapchain(width.max(2) as u32, height.max(2) as u32)?;
        ctx.create_semaphores()?;
        ctx.create_command_buffer()?;
        Ok(ctx)
    }

    fn create_semaphores(&mut self) -> Result<(), String> {
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

    fn create_command_buffer(&mut self) -> Result<(), String> {
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

    fn create_swapchain(&mut self, width: u32, height: u32) -> Result<(), String> {
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

        let hdr_format = self
            .platform
            .prefer_hdr()
            .then(|| {
                formats.iter().find(|f| {
                    f.format == VK_FORMAT_R16G16B16A16_SFLOAT
                        && f.color_space == VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT
                })
            })
            .flatten();
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
        log::info!(
            "{} Vulkan swapchain: format={} color_space={} hdr={} surface_extent={}x{}",
            self.platform.label(),
            chosen_format.format,
            chosen_format.color_space,
            hdr,
            caps.current_extent.width,
            caps.current_extent.height
        );

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
            present_mode: VK_PRESENT_MODE_FIFO_KHR,
            clipped: 1,
            old_swapchain,
        };

        let mut swapchain: VkSwapchainKHR = 0;
        let result = unsafe {
            (self.fns.create_swapchain_khr)(self.device, &create_info, ptr::null(), &mut swapchain)
        };
        if result != VK_SUCCESS {
            return Err(format!("vkCreateSwapchainKHR failed: VkResult {result}"));
        }
        if old_swapchain != 0 {
            unsafe { (self.fns.destroy_swapchain_khr)(self.device, old_swapchain, ptr::null()) };
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

    pub fn enabled_device_extension_ptrs(&self) -> Vec<*const i8> {
        self.device_extension_names
            .iter()
            .map(|name| name.as_ptr())
            .collect()
    }

    pub fn image_usage(&self) -> u32 {
        self.image_usage
    }

    pub fn is_hdr(&self) -> bool {
        self.hdr.load(Ordering::Acquire)
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

    pub fn vk_format(&self) -> i32 {
        self.image_format
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
                u64::MAX,
                self.acquire_semaphore,
                ptr::null_mut(),
                &mut image_index,
            )
        };
        if result == VK_ERROR_OUT_OF_DATE_KHR {
            return Err("vkAcquireNextImageKHR reported VK_ERROR_OUT_OF_DATE_KHR".to_string());
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
        if result == VK_ERROR_OUT_OF_DATE_KHR {
            return Err("vkQueuePresentKHR reported VK_ERROR_OUT_OF_DATE_KHR".to_string());
        }
        if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            return Err(format!("vkQueuePresentKHR failed: VkResult {result}"));
        }
        Ok(())
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
        }
    }
}
