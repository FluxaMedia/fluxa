use std::ffi::{CString, c_void};
use std::path::PathBuf;
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
const VK_ERROR_OUT_OF_DATE_KHR: VkResult = -1000001004;
const VK_SUBOPTIMAL_KHR: VkResult = 1000001003;
const VK_STRUCTURE_TYPE_APPLICATION_INFO: i32 = 0;
const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: i32 = 1;
const VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO: i32 = 2;
const VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO: i32 = 3;
const VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO: i32 = 9;
const VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR: i32 = 1000001000;
const VK_STRUCTURE_TYPE_PRESENT_INFO_KHR: i32 = 1000001001;
const VK_STRUCTURE_TYPE_METAL_SURFACE_CREATE_INFO_EXT: i32 = 1000217000;
const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES: i32 = 1000207000;
const VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES: i32 = 1000314007;
const VK_STRUCTURE_TYPE_SUBMIT_INFO: i32 = 4;
const VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO: i32 = 39;
const VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO: i32 = 40;
const VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO: i32 = 42;
const VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER: i32 = 45;

const VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR: u32 = 0x1;
const VK_QUEUE_GRAPHICS_BIT: u32 = 0x1;
const VK_IMAGE_USAGE_TRANSFER_DST_BIT: u32 = 0x2;
const VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT: u32 = 0x10;
const VK_FORMAT_B8G8R8A8_UNORM: i32 = 44;
const VK_FORMAT_B8G8R8A8_SRGB: i32 = 50;
const VK_FORMAT_R16G16B16A16_SFLOAT: i32 = 97;
const VK_COLOR_SPACE_SRGB_NONLINEAR_KHR: i32 = 0;
const VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT: i32 = 1000104002;
const VK_SHARING_MODE_EXCLUSIVE: i32 = 0;
const VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR: u32 = 0x1;
const VK_PRESENT_MODE_FIFO_KHR: i32 = 2;
const VK_API_VERSION_1_0: u32 = 1 << 22;
const VK_IMAGE_LAYOUT_PRESENT_SRC_KHR: i32 = 1000001002;
const VK_IMAGE_ASPECT_COLOR_BIT: u32 = 0x1;
const VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT: u32 = 0x00002000;
const VK_PIPELINE_STAGE_ALL_COMMANDS_BIT: u32 = 0x00010000;
const VK_ACCESS_MEMORY_READ_BIT: u32 = 0x00008000;
const VK_ACCESS_MEMORY_WRITE_BIT: u32 = 0x00010000;
const VK_COMMAND_BUFFER_LEVEL_PRIMARY: i32 = 0;
const VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT: u32 = 0x1;
const VK_QUEUE_FAMILY_IGNORED: u32 = 0xFFFFFFFF;

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
#[derive(Clone)]
struct VkExtensionProperties {
    extension_name: [u8; 256],
    spec_version: u32,
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
struct VkMetalSurfaceCreateInfoEXT {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    p_layer: *const c_void,
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
type PfnEnumerateInstanceExtensionProperties =
    unsafe extern "system" fn(*const i8, *mut u32, *mut VkExtensionProperties) -> VkResult;
type PfnCreateInstance = unsafe extern "system" fn(
    *const VkInstanceCreateInfo,
    *const c_void,
    *mut VkInstance,
) -> VkResult;
type PfnDestroyInstance = unsafe extern "system" fn(VkInstance, *const c_void);
type PfnEnumeratePhysicalDevices =
    unsafe extern "system" fn(VkInstance, *mut u32, *mut VkPhysicalDevice) -> VkResult;
type PfnGetPhysicalDeviceProperties = unsafe extern "system" fn(VkPhysicalDevice, *mut c_void);
type PfnEnumerateDeviceExtensionProperties = unsafe extern "system" fn(
    VkPhysicalDevice,
    *const i8,
    *mut u32,
    *mut VkExtensionProperties,
) -> VkResult;
type PfnGetPhysicalDeviceQueueFamilyProperties =
    unsafe extern "system" fn(VkPhysicalDevice, *mut u32, *mut VkQueueFamilyProperties);
type PfnGetPhysicalDeviceSurfaceSupportKHR =
    unsafe extern "system" fn(VkPhysicalDevice, u32, VkSurfaceKHR, *mut u32) -> VkResult;
type PfnGetPhysicalDeviceSurfaceCapabilitiesKHR = unsafe extern "system" fn(
    VkPhysicalDevice,
    VkSurfaceKHR,
    *mut VkSurfaceCapabilitiesKHR,
) -> VkResult;
type PfnGetPhysicalDeviceSurfaceFormatsKHR = unsafe extern "system" fn(
    VkPhysicalDevice,
    VkSurfaceKHR,
    *mut u32,
    *mut VkSurfaceFormatKHR,
) -> VkResult;
type PfnCreateMetalSurfaceEXT = unsafe extern "system" fn(
    VkInstance,
    *const VkMetalSurfaceCreateInfoEXT,
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

fn find_moltenvk_path() -> PathBuf {
    let names = ["libMoltenVK.dylib"];
    let mut search_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            search_dirs.push(exe_dir.to_path_buf());
            search_dirs.push(exe_dir.join("lib"));
            if let Some(contents_dir) = exe_dir.parent() {
                search_dirs.push(contents_dir.join("Resources").join("lib"));
                search_dirs.push(contents_dir.join("Frameworks"));
            }
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        search_dirs.push(PathBuf::from(&manifest_dir).join("lib"));
    }
    search_dirs.push(PathBuf::from("/usr/local/lib"));
    search_dirs.push(PathBuf::from("/opt/homebrew/lib"));

    for dir in &search_dirs {
        for name in &names {
            let path = dir.join(name);
            if path.exists() {
                return path;
            }
        }
    }
    PathBuf::from("libMoltenVK.dylib")
}

fn dlopen_path(path: &std::path::Path) -> Option<isize> {
    let cname = CString::new(path.to_string_lossy().as_bytes()).ok()?;
    let handle = unsafe { libc::dlopen(cname.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() {
        None
    } else {
        Some(handle as isize)
    }
}

fn dlsym_typed(module: isize, name: &str) -> Result<*mut c_void, String> {
    let cname = CString::new(name).map_err(|e| e.to_string())?;
    let addr = unsafe { libc::dlsym(module as *mut c_void, cname.as_ptr()) };
    if addr.is_null() {
        Err(format!("libMoltenVK.dylib is missing {name}"))
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
    command_pool: VkCommandPool,
    command_buffer: VkCommandBuffer,
    hdr: AtomicBool,
    device_extension_names: Vec<CString>,
}

unsafe impl Send for VulkanContext {}

impl VulkanContext {
    pub fn new(metal_layer: *const c_void, width: i32, height: i32) -> Result<Self, String> {
        let moltenvk_path = find_moltenvk_path();
        log::info!(
            "macOS Vulkan: loading MoltenVK from {}",
            moltenvk_path.display()
        );
        let module = dlopen_path(&moltenvk_path)
            .ok_or("libMoltenVK.dylib not found (bundle it alongside libmpv.dylib)")?;

        let get_instance_proc_addr: PfnGetInstanceProcAddr =
            unsafe { std::mem::transmute(dlsym_typed(module, "vkGetInstanceProcAddr")?) };
        let create_instance: PfnCreateInstance = unsafe {
            std::mem::transmute(get_instance_proc(
                get_instance_proc_addr,
                ptr::null_mut(),
                "vkCreateInstance",
            )?)
        };
        let enumerate_instance_extension_properties: PfnEnumerateInstanceExtensionProperties = unsafe {
            std::mem::transmute(dlsym_typed(
                module,
                "vkEnumerateInstanceExtensionProperties",
            )?)
        };

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
        let has_instance_ext = |name: &str| {
            instance_exts.iter().any(|e| {
                let end = e.extension_name.iter().position(|&c| c == 0).unwrap_or(0);
                std::str::from_utf8(&e.extension_name[..end]) == Ok(name)
            })
        };
        let portability_enumeration = has_instance_ext("VK_KHR_portability_enumeration");
        if !has_instance_ext("VK_KHR_surface") || !has_instance_ext("VK_EXT_metal_surface") {
            return Err("MoltenVK does not expose the required surface extensions".to_string());
        }

        let app_name = CString::new("fluxa-desktop").unwrap();
        let app_info = VkApplicationInfo {
            s_type: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: 0,
            p_engine_name: app_name.as_ptr(),
            engine_version: 0,
            api_version: VK_API_VERSION_1_0,
        };
        let mut extensions = vec![
            CString::new("VK_KHR_surface").unwrap(),
            CString::new("VK_EXT_metal_surface").unwrap(),
        ];
        if has_instance_ext("VK_EXT_swapchain_colorspace") {
            extensions.push(CString::new("VK_EXT_swapchain_colorspace").unwrap());
        }
        if portability_enumeration {
            extensions.push(CString::new("VK_KHR_portability_enumeration").unwrap());
        }
        let extension_ptrs: Vec<*const i8> = extensions.iter().map(|e| e.as_ptr()).collect();
        let instance_create_info = VkInstanceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: if portability_enumeration {
                VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR
            } else {
                0
            },
            p_application_info: &app_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
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
        let create_metal_surface_ext: PfnCreateMetalSurfaceEXT = iproc!("vkCreateMetalSurfaceEXT");
        let destroy_surface_khr: PfnDestroySurfaceKHR = iproc!("vkDestroySurfaceKHR");
        let create_device: PfnCreateDevice = iproc!("vkCreateDevice");

        let surface_create_info = VkMetalSurfaceCreateInfoEXT {
            s_type: VK_STRUCTURE_TYPE_METAL_SURFACE_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags: 0,
            p_layer: metal_layer,
        };
        let mut surface: VkSurfaceKHR = 0;
        let result = unsafe {
            create_metal_surface_ext(instance, &surface_create_info, ptr::null(), &mut surface)
        };
        if result != VK_SUCCESS {
            unsafe { destroy_instance(instance, ptr::null()) };
            return Err(format!("vkCreateMetalSurfaceEXT failed: VkResult {result}"));
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
                "no Vulkan queue family supports both graphics and presenting to this layer"
                    .to_string(),
            );
        };

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
        let has_device_ext = |name: &str| {
            dev_exts.iter().any(|e| {
                let end = e.extension_name.iter().position(|&c| c == 0).unwrap_or(0);
                std::str::from_utf8(&e.extension_name[..end]) == Ok(name)
            })
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
        if !has_device_ext("VK_KHR_swapchain") {
            unsafe {
                destroy_surface_khr(instance, surface, ptr::null());
                destroy_instance(instance, ptr::null());
            }
            return Err("selected Vulkan device does not expose VK_KHR_swapchain".to_string());
        }
        let mut device_extensions = vec![CString::new("VK_KHR_swapchain").unwrap()];
        if has_device_ext("VK_KHR_portability_subset") {
            device_extensions.push(CString::new("VK_KHR_portability_subset").unwrap());
        }
        let device_extension_ptrs: Vec<*const i8> =
            device_extensions.iter().map(|e| e.as_ptr()).collect();
        let device_create_info = VkDeviceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            p_next: ptr::null(),
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
        Ok(())
    }

    fn create_command_buffer(&mut self) -> Result<(), String> {
        let pool_info = VkCommandPoolCreateInfo {
            s_type: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: 0,
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

        let hdr_format = formats.iter().find(|f| {
            f.format == VK_FORMAT_R16G16B16A16_SFLOAT
                && f.color_space == VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT
        });
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
            "macOS Vulkan swapchain: format={} color_space={} hdr={} surface_extent={}x{}",
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

    pub fn device_handles(&self) -> (*mut c_void, *mut c_void, *mut c_void, u32, u32) {
        (
            self.instance,
            self.phys_device,
            self.device,
            self.queue_family_index,
            1,
        )
    }

    pub fn enabled_device_extensions(&self) -> Vec<*const i8> {
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
        }
        self.acquire_semaphore = 0;
        self.render_done_semaphore = 0;
        self.transition_semaphore = 0;
        let extent = self.extent;
        let _ = self.create_semaphores();
        let _ = self.create_swapchain(extent.width, extent.height);
    }

    fn render_and_present_inner<F>(&mut self, mut render: F) -> Result<(), String>
    where
        F: FnMut(u64, i32, u32, u32, u64, u64) -> Result<i32, String>,
    {
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
            let result = (self.fns.queue_submit)(self.queue, 1, &submit_info, 0);
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
