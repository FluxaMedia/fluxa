use std::ffi::{c_void, CString};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

type VkInstance = *mut c_void;
type VkPhysicalDevice = *mut c_void;
type VkDevice = *mut c_void;
type VkQueue = *mut c_void;
type VkSurfaceKHR = u64;
type VkSwapchainKHR = u64;
type VkSemaphore = u64;
type VkImage = u64;
type VkFence = u64;
type VkCommandPool = *mut c_void;
type VkCommandBuffer = *mut c_void;
type VkResult = i32;

const VK_SUCCESS: VkResult = 0;
const VK_TIMEOUT: VkResult = 2;
const VK_SUBOPTIMAL_KHR: VkResult = 1000001003;
const VK_STRUCTURE_TYPE_APPLICATION_INFO: i32 = 0;
const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: i32 = 1;
const VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO: i32 = 2;
const VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO: i32 = 3;
const VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO: i32 = 9;
const VK_STRUCTURE_TYPE_FENCE_CREATE_INFO: i32 = 8;
const VK_FENCE_CREATE_SIGNALED_BIT: u32 = 1;
const VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR: i32 = 1000001000;
const VK_STRUCTURE_TYPE_PRESENT_INFO_KHR: i32 = 1000001001;
const VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR: i32 = 1000004000;
const VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR: i32 = 1000006000;
const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES: i32 = 1000207000;
const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES: i32 = 1000314007;
const VK_STRUCTURE_TYPE_SUBMIT_INFO: i32 = 4;
const VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO: i32 = 39;
const VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT: u32 = 2;
const VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO: i32 = 40;
const VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO: i32 = 42;
const VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER: i32 = 45;

const VK_QUEUE_GRAPHICS_BIT: u32 = 0x1;
const VK_IMAGE_USAGE_TRANSFER_DST_BIT: u32 = 0x2;
const VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT: u32 = 0x10;
const VK_FORMAT_B8G8R8A8_UNORM: i32 = 44;
const VK_COLOR_SPACE_SRGB_NONLINEAR_KHR: i32 = 0;
const VK_SHARING_MODE_EXCLUSIVE: i32 = 0;
const VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR: u32 = 0x1;
const VK_PRESENT_MODE_MAILBOX_KHR: i32 = 1;
const VK_PRESENT_MODE_FIFO_KHR: i32 = 2;
const VK_API_VERSION_1_3: u32 = (1 << 22) | (3 << 12);
const VK_IMAGE_LAYOUT_PRESENT_SRC_KHR: i32 = 1000001002;
const VK_IMAGE_ASPECT_COLOR_BIT: u32 = 0x1;
const VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT: u32 = 0x00002000;
const VK_PIPELINE_STAGE_ALL_COMMANDS_BIT: u32 = 0x00010000;
const VK_ACCESS_MEMORY_READ_BIT: u32 = 0x00008000;
const VK_ACCESS_MEMORY_WRITE_BIT: u32 = 0x00010000;
const VK_COMMAND_BUFFER_LEVEL_PRIMARY: i32 = 0;
const VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT: u32 = 0x1;
const VK_QUEUE_FAMILY_IGNORED: u32 = 0xFFFFFFFF;

#[derive(Debug)]
pub enum NativeSurface {
    Xlib {
        display: *mut c_void,
        window: u64,
    },
    Wayland {
        display: *mut c_void,
        surface: *mut c_void,
    },
}

#[repr(C)]
struct VkApplicationInfo {
    s_type: i32,
    p_next: *const c_void,
    p_application_name: *const i8,
    application_version: u32,
    p_engine_name: *const i8,
    engine_version: u32,
    api_version: u32,
}

#[repr(C)]
struct VkInstanceCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    p_application_info: *const VkApplicationInfo,
    enabled_layer_count: u32,
    pp_enabled_layer_names: *const *const i8,
    enabled_extension_count: u32,
    pp_enabled_extension_names: *const *const i8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct VkExtent2D {
    width: u32,
    height: u32,
}

#[repr(C)]
struct VkSurfaceCapabilitiesKHR {
    min_image_count: u32,
    max_image_count: u32,
    current_extent: VkExtent2D,
    min_image_extent: VkExtent2D,
    max_image_extent: VkExtent2D,
    max_image_array_layers: u32,
    supported_transforms: u32,
    current_transform: u32,
    supported_composite_alpha: u32,
    supported_usage_flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VkSurfaceFormatKHR {
    format: i32,
    color_space: i32,
}

#[repr(C)]
struct VkQueueFamilyProperties {
    queue_flags: u32,
    queue_count: u32,
    timestamp_valid_bits: u32,
    min_image_transfer_granularity: [u32; 3],
}

#[repr(C)]
struct VkDeviceQueueCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    queue_family_index: u32,
    queue_count: u32,
    p_queue_priorities: *const f32,
}

#[repr(C)]
struct VkDeviceCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    queue_create_info_count: u32,
    p_queue_create_infos: *const VkDeviceQueueCreateInfo,
    enabled_layer_count: u32,
    pp_enabled_layer_names: *const *const i8,
    enabled_extension_count: u32,
    pp_enabled_extension_names: *const *const i8,
    p_enabled_features: *const c_void,
}

#[repr(C)]
struct VkPhysicalDeviceTimelineSemaphoreFeatures {
    s_type: i32,
    p_next: *mut c_void,
    timeline_semaphore: u32,
}

#[repr(C)]
struct VkPhysicalDeviceSynchronization2Features {
    s_type: i32,
    p_next: *mut c_void,
    synchronization2: u32,
}

#[repr(C)]
struct VkXlibSurfaceCreateInfoKHR {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    dpy: *mut c_void,
    window: u64,
}

#[repr(C)]
struct VkWaylandSurfaceCreateInfoKHR {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    display: *mut c_void,
    surface: *mut c_void,
}

#[repr(C)]
struct VkSwapchainCreateInfoKHR {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    surface: VkSurfaceKHR,
    min_image_count: u32,
    image_format: i32,
    image_color_space: i32,
    image_extent: VkExtent2D,
    image_array_layers: u32,
    image_usage: u32,
    image_sharing_mode: i32,
    queue_family_index_count: u32,
    p_queue_family_indices: *const u32,
    pre_transform: u32,
    composite_alpha: u32,
    present_mode: i32,
    clipped: u32,
    old_swapchain: VkSwapchainKHR,
}

#[repr(C)]
struct VkSemaphoreCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
}

#[repr(C)]
struct VkFenceCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
}

#[repr(C)]
struct VkPresentInfoKHR {
    s_type: i32,
    p_next: *const c_void,
    wait_semaphore_count: u32,
    p_wait_semaphores: *const VkSemaphore,
    swapchain_count: u32,
    p_swapchains: *const VkSwapchainKHR,
    p_image_indices: *const u32,
    p_results: *mut VkResult,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VkImageSubresourceRange {
    aspect_mask: u32,
    base_mip_level: u32,
    level_count: u32,
    base_array_layer: u32,
    layer_count: u32,
}

#[repr(C)]
struct VkImageMemoryBarrier {
    s_type: i32,
    p_next: *const c_void,
    src_access_mask: u32,
    dst_access_mask: u32,
    old_layout: i32,
    new_layout: i32,
    src_queue_family_index: u32,
    dst_queue_family_index: u32,
    image: VkImage,
    subresource_range: VkImageSubresourceRange,
}

#[repr(C)]
struct VkCommandPoolCreateInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    queue_family_index: u32,
}

#[repr(C)]
struct VkCommandBufferAllocateInfo {
    s_type: i32,
    p_next: *const c_void,
    command_pool: VkCommandPool,
    level: i32,
    command_buffer_count: u32,
}

#[repr(C)]
struct VkCommandBufferBeginInfo {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    p_inheritance_info: *const c_void,
}

#[repr(C)]
struct VkSubmitInfo {
    s_type: i32,
    p_next: *const c_void,
    wait_semaphore_count: u32,
    p_wait_semaphores: *const VkSemaphore,
    p_wait_dst_stage_mask: *const u32,
    command_buffer_count: u32,
    p_command_buffers: *const VkCommandBuffer,
    signal_semaphore_count: u32,
    p_signal_semaphores: *const VkSemaphore,
}

type PfnGetInstanceProcAddr =
    unsafe extern "system" fn(instance: VkInstance, name: *const i8) -> *mut c_void;
type PfnCreateInstance = unsafe extern "system" fn(
    *const VkInstanceCreateInfo,
    *const c_void,
    *mut VkInstance,
) -> VkResult;
type PfnDestroyInstance = unsafe extern "system" fn(VkInstance, *const c_void);
type PfnEnumeratePhysicalDevices =
    unsafe extern "system" fn(VkInstance, *mut u32, *mut VkPhysicalDevice) -> VkResult;
type PfnEnumerateDeviceExtensionProperties = unsafe extern "system" fn(
    VkPhysicalDevice,
    *const i8,
    *mut u32,
    *mut VkExtensionProperties,
) -> VkResult;

#[repr(C)]
#[derive(Clone, Copy)]
struct VkExtensionProperties {
    extension_name: [u8; 256],
    spec_version: u32,
}
type PfnGetPhysicalDeviceProperties = unsafe extern "system" fn(VkPhysicalDevice, *mut c_void);
type PfnGetPhysicalDeviceQueueFamilyProperties =
    unsafe extern "system" fn(VkPhysicalDevice, *mut u32, *mut VkQueueFamilyProperties);
type PfnGetPhysicalDeviceSurfaceSupportKHR =
    unsafe extern "system" fn(VkPhysicalDevice, u32, VkSurfaceKHR, *mut u32) -> VkResult;
type PfnGetPhysicalDeviceSurfaceCapabilitiesKHR = unsafe extern "system" fn(
    VkPhysicalDevice,
    VkSurfaceKHR,
    *mut VkSurfaceCapabilitiesKHR,
) -> VkResult;
type PfnGetPhysicalDeviceSurfacePresentModesKHR =
    unsafe extern "system" fn(VkPhysicalDevice, VkSurfaceKHR, *mut u32, *mut i32) -> VkResult;
type PfnGetPhysicalDeviceSurfaceFormatsKHR = unsafe extern "system" fn(
    VkPhysicalDevice,
    VkSurfaceKHR,
    *mut u32,
    *mut VkSurfaceFormatKHR,
) -> VkResult;
type PfnCreateXlibSurfaceKHR = unsafe extern "system" fn(
    VkInstance,
    *const VkXlibSurfaceCreateInfoKHR,
    *const c_void,
    *mut VkSurfaceKHR,
) -> VkResult;
type PfnCreateWaylandSurfaceKHR = unsafe extern "system" fn(
    VkInstance,
    *const VkWaylandSurfaceCreateInfoKHR,
    *const c_void,
    *mut VkSurfaceKHR,
) -> VkResult;
type PfnDestroySurfaceKHR = unsafe extern "system" fn(VkInstance, VkSurfaceKHR, *const c_void);
type PfnCreateDevice = unsafe extern "system" fn(
    VkPhysicalDevice,
    *const VkDeviceCreateInfo,
    *const c_void,
    *mut VkDevice,
) -> VkResult;
type PfnDestroyDevice = unsafe extern "system" fn(VkDevice, *const c_void);
type PfnGetDeviceQueue = unsafe extern "system" fn(VkDevice, u32, u32, *mut VkQueue);
type PfnCreateSwapchainKHR = unsafe extern "system" fn(
    VkDevice,
    *const VkSwapchainCreateInfoKHR,
    *const c_void,
    *mut VkSwapchainKHR,
) -> VkResult;
type PfnDestroySwapchainKHR = unsafe extern "system" fn(VkDevice, VkSwapchainKHR, *const c_void);
type PfnGetSwapchainImagesKHR =
    unsafe extern "system" fn(VkDevice, VkSwapchainKHR, *mut u32, *mut VkImage) -> VkResult;
type PfnAcquireNextImageKHR = unsafe extern "system" fn(
    VkDevice,
    VkSwapchainKHR,
    u64,
    VkSemaphore,
    *mut c_void,
    *mut u32,
) -> VkResult;
type PfnQueuePresentKHR = unsafe extern "system" fn(VkQueue, *const VkPresentInfoKHR) -> VkResult;
type PfnCreateSemaphore = unsafe extern "system" fn(
    VkDevice,
    *const VkSemaphoreCreateInfo,
    *const c_void,
    *mut VkSemaphore,
) -> VkResult;
type PfnDestroySemaphore = unsafe extern "system" fn(VkDevice, VkSemaphore, *const c_void);
type PfnCreateFence = unsafe extern "system" fn(
    VkDevice,
    *const VkFenceCreateInfo,
    *const c_void,
    *mut VkFence,
) -> VkResult;
type PfnDestroyFence = unsafe extern "system" fn(VkDevice, VkFence, *const c_void);
type PfnWaitForFences =
    unsafe extern "system" fn(VkDevice, u32, *const VkFence, u32, u64) -> VkResult;
type PfnResetFences = unsafe extern "system" fn(VkDevice, u32, *const VkFence) -> VkResult;
type PfnDeviceWaitIdle = unsafe extern "system" fn(VkDevice) -> VkResult;
type PfnCreateCommandPool = unsafe extern "system" fn(
    VkDevice,
    *const VkCommandPoolCreateInfo,
    *const c_void,
    *mut VkCommandPool,
) -> VkResult;
type PfnDestroyCommandPool = unsafe extern "system" fn(VkDevice, VkCommandPool, *const c_void);
type PfnAllocateCommandBuffers = unsafe extern "system" fn(
    VkDevice,
    *const VkCommandBufferAllocateInfo,
    *mut VkCommandBuffer,
) -> VkResult;
type PfnResetCommandBuffer = unsafe extern "system" fn(VkCommandBuffer, u32) -> VkResult;
type PfnBeginCommandBuffer =
    unsafe extern "system" fn(VkCommandBuffer, *const VkCommandBufferBeginInfo) -> VkResult;
type PfnEndCommandBuffer = unsafe extern "system" fn(VkCommandBuffer) -> VkResult;
type PfnCmdPipelineBarrier = unsafe extern "system" fn(
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
type PfnQueueSubmit =
    unsafe extern "system" fn(VkQueue, u32, *const VkSubmitInfo, VkFence) -> VkResult;

struct VkFns {
    get_instance_proc_addr: PfnGetInstanceProcAddr,
    destroy_instance: PfnDestroyInstance,
    get_physical_device_surface_capabilities_khr: PfnGetPhysicalDeviceSurfaceCapabilitiesKHR,
    get_physical_device_surface_formats_khr: PfnGetPhysicalDeviceSurfaceFormatsKHR,
    get_physical_device_surface_present_modes_khr: PfnGetPhysicalDeviceSurfacePresentModesKHR,
    destroy_surface_khr: PfnDestroySurfaceKHR,
    destroy_device: PfnDestroyDevice,
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

fn dlopen_first(names: &[&str]) -> Option<isize> {
    for name in names {
        let cname = CString::new(*name).ok()?;
        let handle = unsafe { libc::dlopen(cname.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        if !handle.is_null() {
            return Some(handle as isize);
        }
    }
    None
}

fn dlsym_typed(module: isize, name: &str) -> Result<*mut c_void, String> {
    let cname = CString::new(name).map_err(|e| e.to_string())?;
    let addr = unsafe { libc::dlsym(module as *mut c_void, cname.as_ptr()) };
    if addr.is_null() {
        Err(format!("libvulkan.so is missing {name}"))
    } else {
        Ok(addr)
    }
}

fn get_instance_proc(
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

pub struct VulkanContext {
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
    enabled_device_extensions: Vec<CString>,
    owned_xlib_display: *mut c_void,
}

unsafe impl Send for VulkanContext {}


mod init;
mod render;
mod swapchain;
