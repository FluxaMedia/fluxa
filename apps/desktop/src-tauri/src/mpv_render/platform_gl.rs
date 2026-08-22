use super::*;

#[cfg(target_os = "linux")]
pub(super) fn x11_display_ptr() -> Option<*mut c_void> {
    use gdk::prelude::*;
    use glib::translate::ToGlibPtr;

    let display = gdk::Display::default()?;
    let x11_display: gdkx11::X11Display = display.downcast().ok()?;
    let xdisplay =
        unsafe { gdkx11::ffi::gdk_x11_display_get_xdisplay(x11_display.to_glib_none().0) };
    if xdisplay.is_null() {
        None
    } else {
        Some(xdisplay as *mut c_void)
    }
}

#[cfg(target_os = "linux")]
pub(super) fn wl_display_ptr() -> Option<*mut c_void> {
    use gdk::prelude::*;
    use glib::translate::ToGlibPtr;

    let display = gdk::Display::default()?;
    let wayland_display: gdkwayland::WaylandDisplay = display.downcast().ok()?;
    let wl_display = unsafe {
        gdkwayland::ffi::gdk_wayland_display_get_wl_display(wayland_display.to_glib_none().0)
    };
    if wl_display.is_null() {
        None
    } else {
        Some(wl_display as *mut c_void)
    }
}

#[cfg(target_os = "linux")]
pub(super) fn query_draw_fbo() -> c_int {
    let Ok(name) = CString::new("glGetIntegerv") else {
        return 0;
    };
    let ptr = unsafe { get_gl_proc_address(ptr::null_mut(), name.as_ptr()) };
    if ptr.is_null() {
        return 0;
    }
    let get_integerv: GlGetIntegerv = unsafe { std::mem::transmute(ptr) };
    let mut fbo: c_int = 0;
    unsafe { get_integerv(GL_DRAW_FRAMEBUFFER_BINDING, &mut fbo) };
    fbo
}

#[cfg(target_os = "linux")]
static GL_PROC_FN: OnceLock<Option<GlProcFn>> = OnceLock::new();

#[cfg(target_os = "linux")]
pub(super) unsafe extern "C" fn get_gl_proc_address(
    _ctx: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    let Some(f) = *GL_PROC_FN.get_or_init(load_linux_gl_proc_fn) else {
        return ptr::null_mut();
    };
    unsafe { f(name.cast::<c_uchar>()) }
}

/// Resolves the platform's GL proc-address function.
/// Tries EGL first (works on both X11/EGL and Wayland), then falls back to GLX.
#[cfg(target_os = "linux")]
pub(super) fn load_linux_gl_proc_fn() -> Option<GlProcFn> {
    // EGL covers both Wayland and X11-with-EGL (Mesa default on modern distros)
    for lib_name in &["libEGL.so.1", "libEGL.so"] {
        let Ok(name) = CString::new(*lib_name) else {
            continue;
        };
        let Ok(sym) = CString::new("eglGetProcAddress") else {
            continue;
        };
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) };
        if handle.is_null() {
            continue;
        }
        let ptr = unsafe { libc::dlsym(handle, sym.as_ptr()) };
        if !ptr.is_null() {
            return Some(unsafe { std::mem::transmute::<*mut c_void, GlProcFn>(ptr) });
        }
    }

    // GLX fallback for pure X11 setups
    for lib_name in &["libGL.so.1", "libGL.so"] {
        let Ok(name) = CString::new(*lib_name) else {
            continue;
        };
        let Ok(sym) = CString::new("glXGetProcAddressARB") else {
            continue;
        };
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) };
        if handle.is_null() {
            continue;
        }
        let ptr = unsafe { libc::dlsym(handle, sym.as_ptr()) };
        if !ptr.is_null() {
            return Some(unsafe { std::mem::transmute::<*mut c_void, GlProcFn>(ptr) });
        }
    }

    None
}

// Windows OpenGL proc address resolution — delegates to the ANGLE/EGL loader in
// windows_egl.rs, since mpv's D3D11VA hwdec needs an ANGLE-backed GLES context
// (plain WGL has no D3D11 device for hwdec to attach to).

#[cfg(target_os = "windows")]
pub(super) unsafe extern "C" fn get_gl_proc_address(
    ctx: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    unsafe { crate::windows_egl::get_gl_proc_address(ctx, name) }
}

// macOS OpenGL proc address resolution
// Store the handle as usize (Sync-safe) and cast back to pointer when needed.

#[cfg(target_os = "macos")]
static OPENGL_FW_HANDLE: OnceLock<usize> = OnceLock::new();

#[cfg(target_os = "macos")]
pub(super) unsafe extern "C" fn get_gl_proc_address(
    _ctx: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    let handle_addr = *OPENGL_FW_HANDLE.get_or_init(|| {
        let path =
            match std::ffi::CString::new("/System/Library/Frameworks/OpenGL.framework/OpenGL") {
                Ok(s) => s,
                Err(_) => return 0usize,
            };
        libc::dlopen(path.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) as usize
    });
    if handle_addr == 0 {
        return ptr::null_mut();
    }
    libc::dlsym(handle_addr as *mut c_void, name)
}
