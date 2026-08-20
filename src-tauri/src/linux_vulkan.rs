use crate::vulkan::{
    PfnGetInstanceProcAddr, VkInstance, VkResult, VkSurfaceKHR, VulkanPlatform, get_instance_proc,
};
use std::ffi::{CString, c_void};
use std::ptr;
use std::sync::Mutex;

pub use crate::vulkan::VulkanContext;

const VK_SUCCESS: VkResult = 0;
const VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR: i32 = 1000004000;
const VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR: i32 = 1000006000;

#[derive(Clone, Copy)]
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

unsafe impl Send for NativeSurface {}

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
    let cname = CString::new(name).map_err(|_| format!("bad symbol name {name}"))?;
    let addr = unsafe { libc::dlsym(module as *mut c_void, cname.as_ptr()) };
    if addr.is_null() {
        Err(format!("libvulkan.so.1 is missing {name}"))
    } else {
        Ok(addr)
    }
}

struct LinuxPlatform {
    native_surface: NativeSurface,
    owned_xlib_display: Mutex<*mut c_void>,
}

unsafe impl Send for LinuxPlatform {}

impl VulkanPlatform for LinuxPlatform {
    fn label(&self) -> &'static str {
        "linux"
    }

    fn load_loader(&self) -> Result<PfnGetInstanceProcAddr, String> {
        let module = dlopen_first(&["libvulkan.so.1", "libvulkan.so"])
            .ok_or("libvulkan.so.1 not found (no Vulkan-capable driver installed?)")?;
        Ok(unsafe { std::mem::transmute(dlsym_typed(module, "vkGetInstanceProcAddr")?) })
    }

    fn instance_extensions(
        &self,
        _available: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<CString>, String> {
        let surface_ext = match self.native_surface {
            NativeSurface::Xlib { .. } => "VK_KHR_xlib_surface",
            NativeSurface::Wayland { .. } => "VK_KHR_wayland_surface",
        };
        Ok(vec![
            CString::new("VK_KHR_surface").unwrap(),
            CString::new(surface_ext).unwrap(),
            CString::new("VK_EXT_swapchain_colorspace").unwrap(),
        ])
    }

    unsafe fn create_surface(
        &self,
        instance: VkInstance,
        get_proc: PfnGetInstanceProcAddr,
    ) -> Result<VkSurfaceKHR, String> {
        let mut surface: VkSurfaceKHR = 0;
        let result = match self.native_surface {
            NativeSurface::Xlib { display, window } => {
                let create: PfnCreateXlibSurfaceKHR = unsafe {
                    std::mem::transmute(get_instance_proc(
                        get_proc,
                        instance,
                        "vkCreateXlibSurfaceKHR",
                    )?)
                };
                let owned = unsafe { x11::xlib::XOpenDisplay(ptr::null()) as *mut c_void };
                *self.owned_xlib_display.lock().unwrap() = owned;
                let dpy = if owned.is_null() { display } else { owned };
                let create_info = VkXlibSurfaceCreateInfoKHR {
                    s_type: VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
                    p_next: ptr::null(),
                    flags: 0,
                    dpy,
                    window,
                };
                unsafe { create(instance, &create_info, ptr::null(), &mut surface) }
            }
            NativeSurface::Wayland {
                display,
                surface: wl_surface,
            } => {
                let create: PfnCreateWaylandSurfaceKHR = unsafe {
                    std::mem::transmute(get_instance_proc(
                        get_proc,
                        instance,
                        "vkCreateWaylandSurfaceKHR",
                    )?)
                };
                let create_info = VkWaylandSurfaceCreateInfoKHR {
                    s_type: VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
                    p_next: ptr::null(),
                    flags: 0,
                    display,
                    surface: wl_surface,
                };
                unsafe { create(instance, &create_info, ptr::null(), &mut surface) }
            }
        };
        if result != VK_SUCCESS {
            return Err(format!("vkCreate*SurfaceKHR failed: VkResult {result}"));
        }
        Ok(surface)
    }

    fn device_extensions(&self, available: &[String]) -> Result<Vec<CString>, String> {
        if !available.iter().any(|e| e == "VK_KHR_swapchain") {
            return Err("selected Vulkan device does not expose VK_KHR_swapchain".to_string());
        }
        Ok(available
            .iter()
            .filter_map(|name| CString::new(name.as_str()).ok())
            .collect())
    }

    fn prefer_hdr(&self) -> bool {
        false
    }
}

impl Drop for LinuxPlatform {
    fn drop(&mut self) {
        let display = *self.owned_xlib_display.lock().unwrap();
        if !display.is_null() {
            unsafe { x11::xlib::XCloseDisplay(display as *mut x11::xlib::Display) };
        }
    }
}

pub fn create_context(
    native_surface: NativeSurface,
    width: i32,
    height: i32,
) -> Result<VulkanContext, String> {
    VulkanContext::new(
        Box::new(LinuxPlatform {
            native_surface,
            owned_xlib_display: Mutex::new(ptr::null_mut()),
        }),
        width,
        height,
    )
}
