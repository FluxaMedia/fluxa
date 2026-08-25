use crate::vulkan::{
    PfnGetInstanceProcAddr, VkInstance, VkResult, VkSurfaceKHR, VulkanPlatform, get_instance_proc,
};
use std::ffi::{CString, c_void};
use std::path::PathBuf;
use std::ptr;

pub use crate::vulkan::VulkanContext;

const VK_SUCCESS: VkResult = 0;
const VK_STRUCTURE_TYPE_METAL_SURFACE_CREATE_INFO_EXT: i32 = 1000217000;
const VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR: u32 = 0x1;

#[repr(C)]
struct VkMetalSurfaceCreateInfoEXT {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    p_layer: *const c_void,
}

type PfnCreateMetalSurfaceEXT = unsafe extern "system" fn(
    VkInstance,
    *const VkMetalSurfaceCreateInfoEXT,
    *const c_void,
    *mut VkSurfaceKHR,
) -> VkResult;

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

struct MacosPlatform {
    metal_layer: *const c_void,
}

unsafe impl Send for MacosPlatform {}

impl VulkanPlatform for MacosPlatform {
    fn label(&self) -> &'static str {
        "macOS"
    }

    fn load_loader(&self) -> Result<PfnGetInstanceProcAddr, String> {
        let path = find_moltenvk_path();
        log::info!("macOS Vulkan: loading MoltenVK from {}", path.display());
        let module = dlopen_path(&path)
            .ok_or("libMoltenVK.dylib not found (bundle it alongside libmpv.dylib)")?;
        Ok(unsafe { std::mem::transmute(dlsym_typed(module, "vkGetInstanceProcAddr")?) })
    }

    fn instance_extensions(
        &self,
        available: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<CString>, String> {
        if !available("VK_KHR_surface") || !available("VK_EXT_metal_surface") {
            return Err("MoltenVK does not expose the required surface extensions".to_string());
        }
        let mut extensions = vec![
            CString::new("VK_KHR_surface").unwrap(),
            CString::new("VK_EXT_metal_surface").unwrap(),
        ];
        if available("VK_EXT_swapchain_colorspace") {
            extensions.push(CString::new("VK_EXT_swapchain_colorspace").unwrap());
        }
        if available("VK_KHR_portability_enumeration") {
            extensions.push(CString::new("VK_KHR_portability_enumeration").unwrap());
        }
        Ok(extensions)
    }

    fn instance_flags(&self, available: &dyn Fn(&str) -> bool) -> u32 {
        if available("VK_KHR_portability_enumeration") {
            VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR
        } else {
            0
        }
    }

    unsafe fn create_surface(
        &self,
        instance: VkInstance,
        get_proc: PfnGetInstanceProcAddr,
    ) -> Result<VkSurfaceKHR, String> {
        let create: PfnCreateMetalSurfaceEXT = unsafe {
            std::mem::transmute(get_instance_proc(
                get_proc,
                instance,
                "vkCreateMetalSurfaceEXT",
            )?)
        };
        let create_info = VkMetalSurfaceCreateInfoEXT {
            s_type: VK_STRUCTURE_TYPE_METAL_SURFACE_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags: 0,
            p_layer: self.metal_layer,
        };
        let mut surface: VkSurfaceKHR = 0;
        let result = unsafe { create(instance, &create_info, ptr::null(), &mut surface) };
        if result != VK_SUCCESS {
            return Err(format!("vkCreateMetalSurfaceEXT failed: VkResult {result}"));
        }
        Ok(surface)
    }

    fn device_extensions(&self, available: &[String]) -> Result<Vec<CString>, String> {
        if !available.iter().any(|e| e == "VK_KHR_swapchain") {
            return Err("selected Vulkan device does not expose VK_KHR_swapchain".to_string());
        }
        let mut extensions = vec![CString::new("VK_KHR_swapchain").unwrap()];
        for name in [
            "VK_KHR_portability_subset",
            "VK_KHR_synchronization2",
            "VK_KHR_timeline_semaphore",
            "VK_EXT_host_query_reset",
            "VK_KHR_buffer_device_address",
        ] {
            if available.iter().any(|e| e == name) {
                extensions.push(CString::new(name).unwrap());
            }
        }
        Ok(extensions)
    }

    fn prefer_hdr(&self) -> bool {
        true
    }
}

pub fn create_context(
    metal_layer: *const c_void,
    width: i32,
    height: i32,
) -> Result<VulkanContext, String> {
    VulkanContext::new(Box::new(MacosPlatform { metal_layer }), width, height)
}
