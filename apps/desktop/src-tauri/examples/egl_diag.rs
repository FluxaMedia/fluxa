#[cfg(target_os = "windows")]
mod windows_diag {

use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryExA};
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExA, WS_POPUP};

const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x0000_1000;
const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x0000_0100;

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
const EGL_EXTENSIONS: i32 = 0x3055;
const EGL_VENDOR: i32 = 0x3053;
const EGL_VERSION: i32 = 0x3054;
const EGL_PLATFORM_ANGLE_ANGLE: i32 = 0x3202;
const EGL_PLATFORM_ANGLE_TYPE_ANGLE: i32 = 0x3203;
const EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE: i32 = 0x3208;
const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;

type FnGetProcAddress = unsafe extern "system" fn(*const c_char) -> *mut c_void;
type FnGetPlatformDisplayExt =
    unsafe extern "system" fn(i32, *mut c_void, *const i32) -> *mut c_void;
type FnInitialize = unsafe extern "system" fn(*mut c_void, *mut i32, *mut i32) -> i32;
type FnChooseConfig =
    unsafe extern "system" fn(*mut c_void, *const i32, *mut *mut c_void, i32, *mut i32) -> i32;
type FnCreateContext =
    unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void, *const i32) -> *mut c_void;
type FnCreateWindowSurface =
    unsafe extern "system" fn(*mut c_void, *mut c_void, isize, *const i32) -> *mut c_void;
type FnMakeCurrent =
    unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> i32;
type FnQueryString = unsafe extern "system" fn(*mut c_void, i32) -> *const c_char;
type FnGlGetString = unsafe extern "system" fn(u32) -> *const u8;

fn load_dll(name: &str) -> isize {
    let manifest_lib = format!("{}\\lib\\{}", env!("CARGO_MANIFEST_DIR"), name);
    for path in [manifest_lib.as_str(), name] {
        if let Ok(cpath) = CString::new(path) {
            let flags = if path.contains('\\') {
                LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR
            } else {
                0
            };
            let handle = unsafe { LoadLibraryExA(cpath.as_ptr() as *const u8, 0, flags) };
            if handle != 0 {
                println!("loaded {path}");
                return handle;
            }
        }
    }
    panic!("failed to load {name}");
}

fn proc<T: Copy>(module: isize, name: &str) -> T {
    let cname = CString::new(name).unwrap();
    let addr = unsafe { GetProcAddress(module as _, cname.as_ptr() as *const u8) }
        .unwrap_or_else(|| panic!("missing export {name}"));
    unsafe { std::mem::transmute_copy::<_, T>(&addr) }
}

fn has(haystack: &str, ext: &str) -> bool {
    haystack.split_ascii_whitespace().any(|e| e == ext)
}

fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        "<null>".to_string()
    } else {
        unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
    }
}

pub(super) fn main() {
    let egl = load_dll("libEGL.dll");
    let get_proc: FnGetProcAddress = proc(egl, "eglGetProcAddress");
    let initialize: FnInitialize = proc(egl, "eglInitialize");
    let choose_config: FnChooseConfig = proc(egl, "eglChooseConfig");
    let create_context: FnCreateContext = proc(egl, "eglCreateContext");
    let create_surface: FnCreateWindowSurface = proc(egl, "eglCreateWindowSurface");
    let make_current: FnMakeCurrent = proc(egl, "eglMakeCurrent");
    let query_string: FnQueryString = proc(egl, "eglQueryString");

    let client_exts = cstr(unsafe { query_string(ptr::null_mut(), EGL_EXTENSIONS) });
    println!("\n== EGL client extensions ==\n{client_exts}");

    let get_platform_display: FnGetPlatformDisplayExt = {
        let name = CString::new("eglGetPlatformDisplayEXT").unwrap();
        let raw = unsafe { get_proc(name.as_ptr()) };
        assert!(!raw.is_null(), "no eglGetPlatformDisplayEXT");
        unsafe { std::mem::transmute(raw) }
    };

    let display_attribs = [
        EGL_PLATFORM_ANGLE_TYPE_ANGLE,
        EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE,
        EGL_NONE,
    ];
    let display = unsafe {
        get_platform_display(
            EGL_PLATFORM_ANGLE_ANGLE,
            ptr::null_mut(),
            display_attribs.as_ptr(),
        )
    };
    assert!(!display.is_null(), "eglGetPlatformDisplayEXT failed");

    let (mut major, mut minor) = (0i32, 0i32);
    assert!(
        unsafe { initialize(display, &mut major, &mut minor) } != 0,
        "eglInitialize failed"
    );
    println!(
        "\nEGL {major}.{minor}  vendor: {}  version: {}",
        cstr(unsafe { query_string(display, EGL_VENDOR) }),
        cstr(unsafe { query_string(display, EGL_VERSION) })
    );

    let display_exts = cstr(unsafe { query_string(display, EGL_EXTENSIONS) });
    println!("\n== EGL display extensions (ANGLE D3D11) ==\n{display_exts}");

    let hwnd = unsafe {
        let class = CString::new("STATIC").unwrap();
        let title = CString::new("egl-diag").unwrap();
        CreateWindowExA(
            0,
            class.as_ptr() as *const u8,
            title.as_ptr() as *const u8,
            WS_POPUP,
            0,
            0,
            32,
            32,
            0 as _,
            0 as _,
            GetModuleHandleA(ptr::null()) as _,
            ptr::null(),
        )
    };
    assert!(hwnd as isize != 0, "CreateWindowExA failed");

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
    let mut config: *mut c_void = ptr::null_mut();
    let mut num = 0i32;
    assert!(
        unsafe { choose_config(display, config_attribs.as_ptr(), &mut config, 1, &mut num) } != 0
            && num > 0,
        "eglChooseConfig failed"
    );

    let surface = unsafe { create_surface(display, config, hwnd as isize, ptr::null()) };
    assert!(!surface.is_null(), "eglCreateWindowSurface failed");

    let gles = load_dll("libGLESv2.dll");
    let gl_get_string: FnGlGetString = proc(gles, "glGetString");

    for client_version in [3, 2] {
        let attribs = [EGL_CONTEXT_CLIENT_VERSION, client_version, EGL_NONE];
        let context = unsafe { create_context(display, config, ptr::null_mut(), attribs.as_ptr()) };
        if context.is_null() {
            println!("\n== GLES{client_version}: context creation FAILED ==");
            continue;
        }
        assert!(
            unsafe { make_current(display, surface, surface, context) } != 0,
            "eglMakeCurrent failed"
        );

        let version = cstr(unsafe { gl_get_string(GL_VERSION) } as *const c_char);
        let renderer = cstr(unsafe { gl_get_string(GL_RENDERER) } as *const c_char);
        let vendor = cstr(unsafe { gl_get_string(GL_VENDOR) } as *const c_char);
        let gl_exts = cstr(unsafe { gl_get_string(GL_EXTENSIONS) } as *const c_char);

        println!("\n== GLES{client_version} context ==");
        println!("GL_VERSION:  {version}");
        println!("GL_RENDERER: {renderer}");
        println!("GL_VENDOR:   {vendor}");

        println!("\n-- mpv hwdec_d3d11egl probe checklist (GLES{client_version}) --");
        for ext in [
            "EGL_ANGLE_d3d_share_handle_client_buffer",
            "EGL_ANGLE_stream_producer_d3d_texture",
            "EGL_EXT_device_query",
            "EGL_KHR_stream",
            "EGL_KHR_stream_consumer_gltexture",
            "EGL_ANGLE_device_d3d",
            "EGL_ANGLE_device_d3d11",
        ] {
            println!(
                "display {ext}: {}",
                if has(&display_exts, ext) {
                    "YES"
                } else {
                    "MISSING"
                }
            );
        }
        for ext in [
            "GL_OES_EGL_image_external",
            "GL_OES_EGL_image_external_essl3",
            "GL_EXT_texture_rg",
        ] {
            println!(
                "gl      {ext}: {}",
                if has(&gl_exts, ext) { "YES" } else { "MISSING" }
            );
        }

        for name in [
            "eglCreateStreamKHR",
            "eglDestroyStreamKHR",
            "eglStreamConsumerAcquireKHR",
            "eglStreamConsumerReleaseKHR",
            "eglStreamConsumerGLTextureExternalAttribsNV",
            "eglCreateStreamProducerD3DTextureANGLE",
            "eglStreamPostD3DTextureANGLE",
            "eglQueryDisplayAttribEXT",
            "eglQueryDeviceAttribEXT",
        ] {
            let cname = CString::new(name).unwrap();
            let addr = unsafe { get_proc(cname.as_ptr()) };
            println!(
                "fn      {name}: {}",
                if addr.is_null() { "MISSING" } else { "YES" }
            );
        }

        unsafe { make_current(display, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
    }
}

}

#[cfg(target_os = "windows")]
fn main() {
    windows_diag::main();
}

// This diagnostic is implemented only for Windows. Cargo still discovers
// examples on other hosts during a workspace test, so provide a harmless
// entry point instead of making unrelated Linux/macOS test runs fail before
// they reach the library tests.
#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("egl_diag is only available on Windows");
}
