//! Compositor-native fallback controls for the Vulkan player.
//!
//! This is a `wl_subsurface` sibling of the video, not another toplevel. It
//! therefore survives an xdg-toplevel fullscreen transition in the same
//! surface tree as the video. The first fallback is visual-only: its input
//! region is empty so React remains the input owner.

use std::ffi::{c_void, CStr, CString};
use std::ptr;

const WL_DISPLAY_GET_REGISTRY: u32 = 1;
const WL_REGISTRY_BIND: u32 = 0;
const WL_COMPOSITOR_CREATE_SURFACE: u32 = 0;
const WL_COMPOSITOR_CREATE_REGION: u32 = 1;
const WL_SUBCOMPOSITOR_GET_SUBSURFACE: u32 = 1;
const WL_SUBSURFACE_PLACE_ABOVE: u32 = 2;
const WL_SUBSURFACE_SET_DESYNC: u32 = 5;
const WL_SURFACE_ATTACH: u32 = 1;
const WL_SURFACE_DAMAGE: u32 = 2;
const WL_SURFACE_SET_INPUT_REGION: u32 = 5;
const WL_SURFACE_COMMIT: u32 = 6;
const WL_SHM_CREATE_POOL: u32 = 0;
const WL_SHM_POOL_CREATE_BUFFER: u32 = 0;
const WL_SHM_POOL_DESTROY: u32 = 1;
const WL_BUFFER_DESTROY: u32 = 0;
const WL_SUBSURFACE_DESTROY: u32 = 0;
const WL_SURFACE_DESTROY: u32 = 0;
const WL_REGION_DESTROY: u32 = 0;
const WL_SHM_FORMAT_ARGB8888: i32 = 0;

type MarshalNewNoArgs = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *const c_void) -> *mut c_void;
type MarshalNewTwoObjects = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *const c_void, *mut c_void, *mut c_void) -> *mut c_void;
type MarshalBind = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, u32, *const i8, u32, *const c_void) -> *mut c_void;
type MarshalNoArgs = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32) -> *mut c_void;
type MarshalOneObject = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *mut c_void) -> *mut c_void;
type MarshalFdInt = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *const c_void, i32, i32) -> *mut c_void;
type MarshalInts4 = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, i32, i32, i32, i32) -> *mut c_void;
type MarshalInts5 = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *const c_void, i32, i32, i32, i32, i32) -> *mut c_void;
type MarshalAttach = unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *mut c_void, i32, i32) -> *mut c_void;
type ProxyAddListener = unsafe extern "C" fn(*mut c_void, *const c_void, *mut c_void) -> i32;
type ProxyDestroy = unsafe extern "C" fn(*mut c_void);
type ProxyVersion = unsafe extern "C" fn(*mut c_void) -> u32;
type ProxySetQueue = unsafe extern "C" fn(*mut c_void, *mut c_void);
type DisplayCreateQueue = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type DisplayRoundtripQueue = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32;
type DisplayFlush = unsafe extern "C" fn(*mut c_void) -> i32;
type EventQueueDestroy = unsafe extern "C" fn(*mut c_void);

#[repr(C)]
struct RegistryListener {
    global: unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *const i8, u32),
    global_remove: unsafe extern "C" fn(*mut c_void, *mut c_void, u32),
}

#[derive(Clone, Copy)]
struct Fns {
    new_no_args: MarshalNewNoArgs,
    new_two_objects: MarshalNewTwoObjects,
    bind: MarshalBind,
    no_args: MarshalNoArgs,
    one_object: MarshalOneObject,
    fd_int: MarshalFdInt,
    ints4: MarshalInts4,
    ints5: MarshalInts5,
    attach: MarshalAttach,
    add_listener: ProxyAddListener,
    destroy: ProxyDestroy,
    version: ProxyVersion,
    set_queue: ProxySetQueue,
    create_queue: DisplayCreateQueue,
    roundtrip_queue: DisplayRoundtripQueue,
    flush: DisplayFlush,
    queue_destroy: EventQueueDestroy,
}

fn load_symbol<T: Copy>(name: &str) -> Result<T, String> {
    let library = CString::new("libwayland-client.so.0").unwrap();
    let handle = unsafe { libc::dlopen(library.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() { return Err("libwayland-client.so.0 is unavailable".into()); }
    let symbol = CString::new(name).map_err(|error| error.to_string())?;
    let address = unsafe { libc::dlsym(handle, symbol.as_ptr()) };
    if address.is_null() { return Err(format!("Wayland symbol is unavailable: {name}")); }
    Ok(unsafe { std::mem::transmute_copy(&address) })
}

fn load_interface(name: &str) -> Result<*const c_void, String> {
    let library = CString::new("libwayland-client.so.0").unwrap();
    let handle = unsafe { libc::dlopen(library.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() { return Err("libwayland-client.so.0 is unavailable".into()); }
    let symbol = CString::new(name).map_err(|error| error.to_string())?;
    let address = unsafe { libc::dlsym(handle, symbol.as_ptr()) };
    if address.is_null() { return Err(format!("Wayland interface is unavailable: {name}")); }
    Ok(address)
}

fn load_fns() -> Result<Fns, String> {
    Ok(Fns {
        new_no_args: load_symbol("wl_proxy_marshal_flags")?,
        new_two_objects: load_symbol("wl_proxy_marshal_flags")?,
        bind: load_symbol("wl_proxy_marshal_flags")?,
        no_args: load_symbol("wl_proxy_marshal_flags")?,
        one_object: load_symbol("wl_proxy_marshal_flags")?,
        fd_int: load_symbol("wl_proxy_marshal_flags")?,
        ints4: load_symbol("wl_proxy_marshal_flags")?,
        ints5: load_symbol("wl_proxy_marshal_flags")?,
        attach: load_symbol("wl_proxy_marshal_flags")?,
        add_listener: load_symbol("wl_proxy_add_listener")?,
        destroy: load_symbol("wl_proxy_destroy")?,
        version: load_symbol("wl_proxy_get_version")?,
        set_queue: load_symbol("wl_proxy_set_queue")?,
        create_queue: load_symbol("wl_display_create_queue")?,
        roundtrip_queue: load_symbol("wl_display_roundtrip_queue")?,
        flush: load_symbol("wl_display_flush")?,
        queue_destroy: load_symbol("wl_event_queue_destroy")?,
    })
}

#[derive(Default)]
struct Globals { subcompositor: Option<(u32, u32)>, shm: Option<(u32, u32)> }

unsafe extern "C" fn on_global(data: *mut c_void, _registry: *mut c_void, name: u32, interface: *const i8, version: u32) {
    if interface.is_null() { return; }
    let interface = unsafe { CStr::from_ptr(interface) }.to_string_lossy();
    let globals = unsafe { &mut *(data as *mut Globals) };
    match interface.as_ref() {
        "wl_subcompositor" => globals.subcompositor = Some((name, version.min(1))),
        "wl_shm" => globals.shm = Some((name, version.min(1))),
        _ => {}
    }
}

unsafe extern "C" fn on_global_remove(_data: *mut c_void, _registry: *mut c_void, _name: u32) {}

fn bind_globals(fns: Fns, display: *mut c_void, queue: *mut c_void) -> Result<(*mut c_void, *mut c_void), String> {
    let registry_interface = load_interface("wl_registry_interface")?;
    let subcompositor_interface = load_interface("wl_subcompositor_interface")?;
    let shm_interface = load_interface("wl_shm_interface")?;
    let registry = unsafe { (fns.new_no_args)(display, WL_DISPLAY_GET_REGISTRY, registry_interface, (fns.version)(display), 0, ptr::null()) };
    if registry.is_null() { return Err("wl_display_get_registry failed for native overlay".into()); }
    unsafe { (fns.set_queue)(registry, queue) };
    let mut globals = Globals::default();
    let listener = RegistryListener { global: on_global, global_remove: on_global_remove };
    let result = unsafe { (fns.add_listener)(registry, &listener as *const _ as *const c_void, &mut globals as *mut _ as *mut c_void) };
    if result != 0 || unsafe { (fns.roundtrip_queue)(display, queue) } < 0 {
        unsafe { (fns.destroy)(registry) };
        return Err("Wayland registry roundtrip failed for native overlay".into());
    }
    let (sub_name, sub_version) = globals.subcompositor.ok_or("wl_subcompositor is unavailable for native overlay")?;
    let (shm_name, shm_version) = globals.shm.ok_or("wl_shm is unavailable for native overlay")?;
    let subcompositor = unsafe { (fns.bind)(registry, WL_REGISTRY_BIND, subcompositor_interface, sub_version, 0, sub_name, b"wl_subcompositor\0".as_ptr() as *const i8, sub_version, ptr::null()) };
    let shm = unsafe { (fns.bind)(registry, WL_REGISTRY_BIND, shm_interface, shm_version, 0, shm_name, b"wl_shm\0".as_ptr() as *const i8, shm_version, ptr::null()) };
    unsafe { (fns.destroy)(registry) };
    if subcompositor.is_null() || shm.is_null() { return Err("wl_registry_bind failed for native overlay".into()); }
    Ok((subcompositor, shm))
}

pub struct NativeOverlaySurface {
    fns: Fns,
    display: *mut c_void,
    queue: *mut c_void,
    surface: *mut c_void,
    subsurface: *mut c_void,
    subcompositor: *mut c_void,
    shm: *mut c_void,
    buffer: *mut c_void,
    mapping: *mut u8,
    mapping_len: usize,
    width: i32,
    height: i32,
}

impl NativeOverlaySurface {
    pub fn new(display: *mut c_void, compositor: *mut c_void, parent_surface: *mut c_void, video_surface: *mut c_void, width: i32, height: i32) -> Result<Self, String> {
        let fns = load_fns()?;
        let queue = unsafe { (fns.create_queue)(display) };
        if queue.is_null() { return Err("wl_display_create_queue failed for native overlay".into()); }
        let (subcompositor, shm) = match bind_globals(fns, display, queue) {
            Ok(value) => value,
            Err(error) => { unsafe { (fns.queue_destroy)(queue) }; return Err(error); }
        };
        let surface_interface = load_interface("wl_surface_interface")?;
        let subsurface_interface = load_interface("wl_subsurface_interface")?;
        let region_interface = load_interface("wl_region_interface")?;
        let pool_interface = load_interface("wl_shm_pool_interface")?;
        let buffer_interface = load_interface("wl_buffer_interface")?;
        let surface = unsafe { (fns.new_no_args)(compositor, WL_COMPOSITOR_CREATE_SURFACE, surface_interface, (fns.version)(compositor), 0, ptr::null()) };
        let subsurface = unsafe { (fns.new_two_objects)(subcompositor, WL_SUBCOMPOSITOR_GET_SUBSURFACE, subsurface_interface, (fns.version)(subcompositor), 0, ptr::null(), surface, parent_surface) };
        if surface.is_null() || subsurface.is_null() {
            if !subsurface.is_null() { unsafe { (fns.destroy)(subsurface) }; }
            if !surface.is_null() { unsafe { (fns.destroy)(surface) }; }
            unsafe { (fns.destroy)(subcompositor); (fns.destroy)(shm); (fns.queue_destroy)(queue); }
            return Err("failed to create native overlay wl_subsurface".into());
        }
        let width = width.max(2);
        let height = height.max(2);
        let size = (width as usize).checked_mul(height as usize).and_then(|value| value.checked_mul(4)).ok_or("native overlay buffer is too large")?;
        let fd = unsafe { libc::memfd_create(b"fluxa-native-overlay\0".as_ptr() as *const i8, libc::MFD_CLOEXEC) };
        if fd < 0 || unsafe { libc::ftruncate(fd, size as libc::off_t) } != 0 {
            if fd >= 0 { unsafe { libc::close(fd) }; }
            return Err("memfd_create/ftruncate failed for native overlay".into());
        }
        let mapping = unsafe { libc::mmap(ptr::null_mut(), size, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0) } as *mut u8;
        if mapping as *mut c_void == libc::MAP_FAILED { unsafe { libc::close(fd) }; return Err("mmap failed for native overlay".into()); }
        let pool = unsafe { (fns.fd_int)(shm, WL_SHM_CREATE_POOL, pool_interface, (fns.version)(shm), 0, ptr::null(), fd, size as i32) };
        unsafe { libc::close(fd) };
        if pool.is_null() { unsafe { libc::munmap(mapping as *mut c_void, size) }; return Err("wl_shm.create_pool failed for native overlay".into()); }
        let buffer = unsafe { (fns.ints5)(pool, WL_SHM_POOL_CREATE_BUFFER, buffer_interface, (fns.version)(pool), 0, ptr::null(), 0, width, height, width * 4, WL_SHM_FORMAT_ARGB8888) };
        unsafe { (fns.no_args)(pool, WL_SHM_POOL_DESTROY, ptr::null(), (fns.version)(pool), 0) };
        if buffer.is_null() { unsafe { libc::munmap(mapping as *mut c_void, size) }; return Err("wl_shm_pool.create_buffer failed for native overlay".into()); }
        unsafe {
            // An explicitly empty region leaves pointer/keyboard handling
            // with the parent WebView. A null region would mean the default
            // full-surface input region and would swallow React clicks.
            let empty_region = (fns.new_no_args)(compositor, WL_COMPOSITOR_CREATE_REGION, region_interface, (fns.version)(compositor), 0, ptr::null());
            if !empty_region.is_null() {
                (fns.one_object)(surface, WL_SURFACE_SET_INPUT_REGION, ptr::null(), (fns.version)(surface), 0, empty_region);
                (fns.no_args)(empty_region, WL_REGION_DESTROY, ptr::null(), (fns.version)(empty_region), 0);
            }
            (fns.no_args)(subsurface, WL_SUBSURFACE_SET_DESYNC, ptr::null(), (fns.version)(subsurface), 0);
            (fns.one_object)(subsurface, WL_SUBSURFACE_PLACE_ABOVE, ptr::null(), (fns.version)(subsurface), 0, video_surface);
        }
        let result = Self { fns, display, queue, surface, subsurface, subcompositor, shm, buffer, mapping, mapping_len: size, width, height };
        // Keep the surface hidden until the player receives its first Load.
        // Otherwise installing the native player at app startup would leave
        // the fallback controls over the ordinary Fluxa screens.
        result.hide();
        log::warn!("experimental-native: native Wayland sibling overlay created surface={:?} subsurface={:?} size={}x{}", surface, subsurface, width, height);
        Ok(result)
    }

    fn fill_rect(mapping: *mut u8, width: i32, height: i32, x: i32, y: i32, rect_width: i32, rect_height: i32, color: (u8, u8, u8, u8)) {
        let x_end = (x + rect_width).min(width);
        let y_end = (y + rect_height).min(height);
        for yy in y.max(0)..y_end {
            for xx in x.max(0)..x_end {
                let index = ((yy * width + xx) * 4) as usize;
                unsafe { *mapping.add(index) = color.2; *mapping.add(index + 1) = color.1; *mapping.add(index + 2) = color.0; *mapping.add(index + 3) = color.3; }
            }
        }
    }

    fn draw(&self) {
        unsafe { ptr::write_bytes(self.mapping, 0, self.mapping_len) };
        // Top-left controls make the layer visible in windowed mode too.
        Self::fill_rect(self.mapping, self.width, self.height, 20, 20, 360, 44, (0, 0, 0, 210));
        Self::fill_rect(self.mapping, self.width, self.height, 28, 28, 46, 28, (255, 255, 255, 235));
        Self::fill_rect(self.mapping, self.width, self.height, 88, 28, 86, 28, (45, 45, 45, 235));
        Self::fill_rect(self.mapping, self.width, self.height, 184, 28, 86, 28, (45, 45, 45, 235));
        let y = self.height - 92;
        Self::fill_rect(self.mapping, self.width, self.height, 20, y, self.width - 40, 64, (0, 0, 0, 210));
        unsafe {
            (self.fns.attach)(self.surface, WL_SURFACE_ATTACH, ptr::null(), (self.fns.version)(self.surface), 0, self.buffer, 0, 0);
            (self.fns.ints4)(self.surface, WL_SURFACE_DAMAGE, ptr::null(), (self.fns.version)(self.surface), 0, 0, 0, self.width, self.height);
            (self.fns.no_args)(self.surface, WL_SURFACE_COMMIT, ptr::null(), (self.fns.version)(self.surface), 0);
            let _ = (self.fns.flush)(self.display);
        }
    }

    pub fn resize(&self, _width: i32, _height: i32) {
        // The allocation is monitor-sized by NativeVideoHost. The parent
        // clips it when windowed and the same surface survives fullscreen.
    }

    pub fn show(&self) {
        self.draw();
    }

    pub fn hide(&self) {
        unsafe {
            (self.fns.attach)(self.surface, WL_SURFACE_ATTACH, ptr::null(), (self.fns.version)(self.surface), 0, ptr::null_mut(), 0, 0);
            (self.fns.no_args)(self.surface, WL_SURFACE_COMMIT, ptr::null(), (self.fns.version)(self.surface), 0);
            let _ = (self.fns.flush)(self.display);
        }
    }
}

impl Drop for NativeOverlaySurface {
    fn drop(&mut self) {
        unsafe {
            (self.fns.no_args)(self.buffer, WL_BUFFER_DESTROY, ptr::null(), (self.fns.version)(self.buffer), 0);
            (self.fns.no_args)(self.subsurface, WL_SUBSURFACE_DESTROY, ptr::null(), (self.fns.version)(self.subsurface), 0);
            (self.fns.destroy)(self.subsurface);
            (self.fns.no_args)(self.surface, WL_SURFACE_DESTROY, ptr::null(), (self.fns.version)(self.surface), 0);
            (self.fns.destroy)(self.surface);
            (self.fns.destroy)(self.subcompositor);
            (self.fns.destroy)(self.shm);
            (self.fns.queue_destroy)(self.queue);
            libc::munmap(self.mapping as *mut c_void, self.mapping_len);
        }
    }
}
