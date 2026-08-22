use crate::vulkan::{
    PfnGetInstanceProcAddr, VkInstance, VkResult, VkSurfaceKHR, VulkanPlatform, get_instance_proc,
};
use std::ffi::{CString, c_void};
use std::ptr;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryA};

pub use crate::vulkan::VulkanContext;

const VK_SUCCESS: VkResult = 0;
const VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR: i32 = 1000009000;

#[repr(C)]
struct VkWin32SurfaceCreateInfoKHR {
    s_type: i32,
    p_next: *const c_void,
    flags: u32,
    hinstance: *mut c_void,
    hwnd: HWND,
}

type PfnCreateWin32SurfaceKHR = unsafe extern "system" fn(
    VkInstance,
    *const VkWin32SurfaceCreateInfoKHR,
    *const c_void,
    *mut VkSurfaceKHR,
) -> VkResult;

fn load_proc(module: isize, name: &str) -> Result<*mut c_void, String> {
    let cname = CString::new(name).map_err(|_| format!("bad symbol name {name}"))?;
    let addr = unsafe { GetProcAddress(module, cname.as_ptr() as *const u8) };
    match addr {
        Some(addr) => Ok(addr as *mut c_void),
        None => Err(format!("vulkan-1.dll is missing {name}")),
    }
}

struct WindowsPlatform {
    hwnd: isize,
}

impl VulkanPlatform for WindowsPlatform {
    fn label(&self) -> &'static str {
        "windows"
    }

    fn load_loader(&self) -> Result<PfnGetInstanceProcAddr, String> {
        let module = unsafe { LoadLibraryA(b"vulkan-1.dll\0".as_ptr()) };
        if module == 0 {
            return Err("vulkan-1.dll not found (no Vulkan-capable driver installed?)".into());
        }
        Ok(unsafe { std::mem::transmute(load_proc(module, "vkGetInstanceProcAddr")?) })
    }

    fn instance_extensions(
        &self,
        _available: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<CString>, String> {
        Ok(vec![
            CString::new("VK_KHR_surface").unwrap(),
            CString::new("VK_KHR_win32_surface").unwrap(),
            CString::new("VK_EXT_swapchain_colorspace").unwrap(),
        ])
    }

    unsafe fn create_surface(
        &self,
        instance: VkInstance,
        get_proc: PfnGetInstanceProcAddr,
    ) -> Result<VkSurfaceKHR, String> {
        let create: PfnCreateWin32SurfaceKHR = unsafe {
            std::mem::transmute(get_instance_proc(
                get_proc,
                instance,
                "vkCreateWin32SurfaceKHR",
            )?)
        };
        let hinstance = unsafe { GetModuleHandleW(ptr::null()) };
        let create_info = VkWin32SurfaceCreateInfoKHR {
            s_type: VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: 0,
            hinstance: hinstance as *mut c_void,
            hwnd: self.hwnd as HWND,
        };
        let mut surface: VkSurfaceKHR = 0;
        let result = unsafe { create(instance, &create_info, ptr::null(), &mut surface) };
        if result != VK_SUCCESS {
            return Err(format!("vkCreateWin32SurfaceKHR failed: VkResult {result}"));
        }
        Ok(surface)
    }

    fn device_extensions(&self, available: &[String]) -> Result<Vec<CString>, String> {
        if !available.iter().any(|e| e == "VK_KHR_swapchain") {
            return Err("selected Vulkan device does not expose VK_KHR_swapchain".to_string());
        }
        Ok(vec![CString::new("VK_KHR_swapchain").unwrap()])
    }

    fn prefer_hdr(&self) -> bool {
        true
    }
}

pub fn create_context(hwnd: isize, width: i32, height: i32) -> Result<VulkanContext, String> {
    VulkanContext::new(Box::new(WindowsPlatform { hwnd }), width, height)
}
