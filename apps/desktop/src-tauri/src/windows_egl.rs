// Windows ANGLE/EGL context — backs mpv's OpenGL render API with a real D3D11
// device so mpv's D3D11VA hwdec (hwdec_d3d11egl.c) has something to attach to.
// Plain WGL exposes no D3D11 device at all, so hardware decode never activates
// there regardless of GPU vendor; ANGLE is Google's GLES-over-D3D11 layer and
// is the same mechanism mpv's own windowed `--gpu-context=angle` uses.
//
// libEGL.dll / libGLESv2.dll are bundled alongside libmpv (src-tauri/lib/),
// not present on a stock Windows install.

use std::ffi::{c_char, c_void, CString};
use std::ptr;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExA};

const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x0000_1000;
const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x0000_0100;

type EglDisplay = *mut c_void;
type EglSurfaceHandle = *mut c_void;
type EglContextHandle = *mut c_void;
type EglConfig = *mut c_void;

const EGL_NONE: i32 = 0x3038;
const EGL_OPENGL_ES2_BIT: i32 = 0x0004;
const EGL_OPENGL_ES3_BIT_KHR: i32 = 0x0040;
const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_WINDOW_BIT: i32 = 0x0004;
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_GREEN_SIZE: i32 = 0x3023;
const EGL_BLUE_SIZE: i32 = 0x3022;
const EGL_ALPHA_SIZE: i32 = 0x3021;
const EGL_CONTEXT_CLIENT_VERSION: i32 = 0x3098;

// ANGLE platform extension (EGL_ANGLE_platform_angle / EGL_EXT_platform_base).
const EGL_PLATFORM_ANGLE_ANGLE: i32 = 0x3202;
const EGL_PLATFORM_ANGLE_TYPE_ANGLE: i32 = 0x3203;
const EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE: i32 = 0x3208;

type EglGetPlatformDisplayExt =
    unsafe extern "system" fn(i32, *mut c_void, *const i32) -> EglDisplay;
type EglInitialize = unsafe extern "system" fn(EglDisplay, *mut i32, *mut i32) -> i32;
type EglChooseConfig =
    unsafe extern "system" fn(EglDisplay, *const i32, *mut EglConfig, i32, *mut i32) -> i32;
type EglCreateContext = unsafe extern "system" fn(
    EglDisplay,
    EglConfig,
    EglContextHandle,
    *const i32,
) -> EglContextHandle;
type EglCreateWindowSurface =
    unsafe extern "system" fn(EglDisplay, EglConfig, isize, *const i32) -> EglSurfaceHandle;
type EglMakeCurrent = unsafe extern "system" fn(
    EglDisplay,
    EglSurfaceHandle,
    EglSurfaceHandle,
    EglContextHandle,
) -> i32;
type EglSwapBuffers = unsafe extern "system" fn(EglDisplay, EglSurfaceHandle) -> i32;
type EglSwapInterval = unsafe extern "system" fn(EglDisplay, i32) -> i32;
type EglDestroySurface = unsafe extern "system" fn(EglDisplay, EglSurfaceHandle) -> i32;
type EglDestroyContext = unsafe extern "system" fn(EglDisplay, EglContextHandle) -> i32;
type EglTerminate = unsafe extern "system" fn(EglDisplay) -> i32;
type EglGetProcAddress = unsafe extern "system" fn(*const c_char) -> *mut c_void;
type EglWaitClient = unsafe extern "system" fn() -> i32;

pub struct EglContext {
    display: EglDisplay,
    surface: EglSurfaceHandle,
    context: EglContextHandle,
    make_current: EglMakeCurrent,
    swap_buffers_fn: EglSwapBuffers,
    swap_interval_fn: EglSwapInterval,
    destroy_surface: EglDestroySurface,
    destroy_context: EglDestroyContext,
    terminate: EglTerminate,
    wait_client: Option<EglWaitClient>,
}

unsafe impl Send for EglContext {}

impl EglContext {
    pub fn swap_buffers(&self) {
        unsafe { (self.swap_buffers_fn)(self.display, self.surface) };
    }

    pub fn set_swap_interval(&self, interval: i32) -> bool {
        unsafe { (self.swap_interval_fn)(self.display, interval) != 0 }
    }

    /// Forces ANGLE to pick up an HWND client-area resize immediately rather
    /// than waiting for the next present to notice.
    pub fn poll_resize(&self) {
        if let Some(wait_client) = self.wait_client {
            unsafe { wait_client() };
        }
    }
}

impl Drop for EglContext {
    fn drop(&mut self) {
        unsafe {
            (self.make_current)(
                self.display,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            (self.destroy_context)(self.display, self.context);
            (self.destroy_surface)(self.display, self.surface);
            (self.terminate)(self.display);
        }
    }
}

fn find_and_load_dll(name: &str) -> Result<isize, String> {
    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            search_dirs.push(exe_dir.to_path_buf());
            search_dirs.push(exe_dir.join("lib"));
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        search_dirs.push(std::path::PathBuf::from(&manifest_dir).join("lib"));
    }

    for dir in &search_dirs {
        let path = dir.join(name);
        if !path.exists() {
            continue;
        }
        if let Some(path_str) = path.to_str() {
            if let Ok(cpath) = CString::new(path_str) {
                let handle = unsafe {
                    LoadLibraryExA(
                        cpath.as_ptr() as *const u8,
                        0,
                        LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
                    )
                };
                if handle != 0 {
                    return Ok(handle);
                }
            }
        }
    }

    let cname = CString::new(name).map_err(|e| e.to_string())?;
    let handle = unsafe { LoadLibraryExA(cname.as_ptr() as *const u8, 0, 0) };
    if handle == 0 {
        Err(format!(
            "failed to load {name} (not bundled beside the executable, and not on PATH)"
        ))
    } else {
        Ok(handle)
    }
}

fn load_proc<T: Copy>(module: isize, name: &[u8]) -> Option<T> {
    let addr = unsafe { GetProcAddress(module as _, name.as_ptr()) }?;
    Some(unsafe { std::mem::transmute_copy::<_, T>(&addr) })
}

pub fn create_window_context(hwnd: HWND) -> Result<EglContext, String> {
    let module = find_and_load_dll("libEGL.dll")?;

    let get_proc_address: EglGetProcAddress = load_proc(module, b"eglGetProcAddress\0")
        .ok_or("libEGL.dll is missing eglGetProcAddress")?;
    let initialize: EglInitialize =
        load_proc(module, b"eglInitialize\0").ok_or("libEGL.dll is missing eglInitialize")?;
    let choose_config: EglChooseConfig =
        load_proc(module, b"eglChooseConfig\0").ok_or("libEGL.dll is missing eglChooseConfig")?;
    let create_context: EglCreateContext =
        load_proc(module, b"eglCreateContext\0").ok_or("libEGL.dll is missing eglCreateContext")?;
    let create_window_surface: EglCreateWindowSurface =
        load_proc(module, b"eglCreateWindowSurface\0")
            .ok_or("libEGL.dll is missing eglCreateWindowSurface")?;
    let make_current: EglMakeCurrent =
        load_proc(module, b"eglMakeCurrent\0").ok_or("libEGL.dll is missing eglMakeCurrent")?;
    let swap_buffers_fn: EglSwapBuffers =
        load_proc(module, b"eglSwapBuffers\0").ok_or("libEGL.dll is missing eglSwapBuffers")?;
    let swap_interval_fn: EglSwapInterval =
        load_proc(module, b"eglSwapInterval\0").ok_or("libEGL.dll is missing eglSwapInterval")?;
    let destroy_surface: EglDestroySurface = load_proc(module, b"eglDestroySurface\0")
        .ok_or("libEGL.dll is missing eglDestroySurface")?;
    let destroy_context: EglDestroyContext = load_proc(module, b"eglDestroyContext\0")
        .ok_or("libEGL.dll is missing eglDestroyContext")?;
    let terminate: EglTerminate =
        load_proc(module, b"eglTerminate\0").ok_or("libEGL.dll is missing eglTerminate")?;
    let wait_client: Option<EglWaitClient> = load_proc(module, b"eglWaitClient\0");

    let get_platform_display_ext: EglGetPlatformDisplayExt = {
        let name = CString::new("eglGetPlatformDisplayEXT").unwrap();
        let raw = unsafe { get_proc_address(name.as_ptr()) };
        if raw.is_null() {
            return Err(
                "ANGLE is missing EGL_EXT_platform_base (eglGetPlatformDisplayEXT)".to_string(),
            );
        }
        unsafe { std::mem::transmute(raw) }
    };

    let display_attribs = [
        EGL_PLATFORM_ANGLE_TYPE_ANGLE,
        EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE,
        EGL_NONE,
    ];
    let display = unsafe {
        get_platform_display_ext(
            EGL_PLATFORM_ANGLE_ANGLE,
            ptr::null_mut(),
            display_attribs.as_ptr(),
        )
    };
    if display.is_null() {
        return Err("eglGetPlatformDisplayEXT(ANGLE/D3D11) failed".to_string());
    }

    let mut major = 0i32;
    let mut minor = 0i32;
    if unsafe { initialize(display, &mut major, &mut minor) } == 0 {
        return Err("eglInitialize failed".to_string());
    }

    let config_attribs = [
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT | EGL_OPENGL_ES3_BIT_KHR,
        EGL_SURFACE_TYPE,
        EGL_WINDOW_BIT,
        EGL_RED_SIZE,
        8,
        EGL_GREEN_SIZE,
        8,
        EGL_BLUE_SIZE,
        8,
        EGL_ALPHA_SIZE,
        8,
        EGL_NONE,
    ];
    let mut config: EglConfig = ptr::null_mut();
    let mut num_config = 0i32;
    if unsafe {
        choose_config(
            display,
            config_attribs.as_ptr(),
            &mut config,
            1,
            &mut num_config,
        )
    } == 0
        || num_config == 0
    {
        unsafe { terminate(display) };
        return Err("eglChooseConfig found no usable ANGLE/D3D11 config".to_string());
    }

    let surface = unsafe { create_window_surface(display, config, hwnd, ptr::null()) };
    if surface.is_null() {
        unsafe { terminate(display) };
        return Err("eglCreateWindowSurface failed".to_string());
    }

    let mut context: EglContextHandle = ptr::null_mut();
    for client_version in [3, 2] {
        let context_attribs = [EGL_CONTEXT_CLIENT_VERSION, client_version, EGL_NONE];
        context =
            unsafe { create_context(display, config, ptr::null_mut(), context_attribs.as_ptr()) };
        if !context.is_null() {
            break;
        }
    }
    if context.is_null() {
        unsafe {
            destroy_surface(display, surface);
            terminate(display);
        }
        return Err("eglCreateContext failed for both GLES3 and GLES2".to_string());
    }

    if unsafe { make_current(display, surface, surface, context) } == 0 {
        unsafe {
            destroy_context(display, context);
            destroy_surface(display, surface);
            terminate(display);
        }
        return Err("eglMakeCurrent failed".to_string());
    }

    Ok(EglContext {
        display,
        surface,
        context,
        make_current,
        swap_buffers_fn,
        swap_interval_fn,
        destroy_surface,
        destroy_context,
        terminate,
        wait_client,
    })
}

// GL proc-address resolution for mpv's MPV_RENDER_PARAM_OPENGL_INIT_PARAMS callback.
// Self-contained: loads libEGL.dll/libGLESv2.dll independently of any EglContext,
// mirroring how the Linux build resolves GL/GLES symbols via dlopen(libEGL.so).

static EGL_GET_PROC_ADDRESS: OnceLock<Option<EglGetProcAddress>> = OnceLock::new();
static GLESV2_MODULE: OnceLock<isize> = OnceLock::new();

pub unsafe extern "C" fn get_gl_proc_address(
    _ctx: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    let resolver = *EGL_GET_PROC_ADDRESS.get_or_init(|| {
        let module = find_and_load_dll("libEGL.dll").ok()?;
        load_proc(module, b"eglGetProcAddress\0")
    });
    if let Some(get_proc) = resolver {
        let addr = unsafe { get_proc(name) };
        if !addr.is_null() {
            return addr;
        }
    }
    let module = *GLESV2_MODULE.get_or_init(|| find_and_load_dll("libGLESv2.dll").unwrap_or(0));
    if module == 0 {
        return ptr::null_mut();
    }
    match unsafe { GetProcAddress(module as _, name as *const u8) } {
        Some(f) => f as *mut c_void,
        None => ptr::null_mut(),
    }
}
