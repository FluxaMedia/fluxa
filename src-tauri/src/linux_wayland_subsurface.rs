use std::ffi::{c_void, CStr, CString};
use std::ptr;

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
        Err(format!("libwayland-client.so is missing {name}"))
    } else {
        Ok(addr)
    }
}

const WL_DISPLAY_GET_REGISTRY: u32 = 1;
const WL_REGISTRY_BIND: u32 = 0;
const WL_COMPOSITOR_CREATE_SURFACE: u32 = 0;
const WL_SUBCOMPOSITOR_GET_SUBSURFACE: u32 = 1;
const WL_SUBSURFACE_PLACE_BELOW: u32 = 3;
const WL_SUBSURFACE_SET_DESYNC: u32 = 5;
const WL_SURFACE_ATTACH: u32 = 1;
const WL_SURFACE_COMMIT: u32 = 6;
const WL_SURFACE_DESTROY: u32 = 0;
const WL_SUBSURFACE_DESTROY: u32 = 0;

type PfnMarshalNewNoArgs =
    unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *const c_void) -> *mut c_void;
type PfnMarshalNewTwoObjArgs = unsafe extern "C" fn(
    *mut c_void,
    u32,
    *const c_void,
    u32,
    u32,
    *const c_void,
    *mut c_void,
    *mut c_void,
) -> *mut c_void;
type PfnMarshalBind = unsafe extern "C" fn(
    *mut c_void,
    u32,
    *const c_void,
    u32,
    u32,
    u32,
    *const i8,
    u32,
    *const c_void,
) -> *mut c_void;
type PfnMarshalNoArgs =
    unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32) -> *mut c_void;
type PfnMarshalOneObjArg =
    unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, *mut c_void) -> *mut c_void;
type PfnMarshalAttach = unsafe extern "C" fn(
    *mut c_void,
    u32,
    *const c_void,
    u32,
    u32,
    *mut c_void,
    i32,
    i32,
) -> *mut c_void;
type PfnProxyAddListener = unsafe extern "C" fn(*mut c_void, *const c_void, *mut c_void) -> i32;
type PfnProxyDestroy = unsafe extern "C" fn(*mut c_void);
type PfnProxyGetVersion = unsafe extern "C" fn(*mut c_void) -> u32;
type PfnProxySetQueue = unsafe extern "C" fn(*mut c_void, *mut c_void);
type PfnDisplayCreateQueue = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type PfnDisplayRoundtripQueue = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32;
type PfnEventQueueDestroy = unsafe extern "C" fn(*mut c_void);

#[repr(C)]
struct WlRegistryListener {
    global: unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *const i8, u32),
    global_remove: unsafe extern "C" fn(*mut c_void, *mut c_void, u32),
}

struct FoundGlobal {
    name: u32,
    version: u32,
}

unsafe extern "C" fn on_global(
    data: *mut c_void,
    _registry: *mut c_void,
    name: u32,
    interface: *const i8,
    version: u32,
) {
    if interface.is_null() {
        return;
    }
    let iface_name = unsafe { CStr::from_ptr(interface) }.to_string_lossy();
    if iface_name == "wl_subcompositor" {
        let out = data as *mut Option<FoundGlobal>;
        unsafe { *out = Some(FoundGlobal { name, version }) };
    }
}

unsafe extern "C" fn on_global_remove(_data: *mut c_void, _registry: *mut c_void, _name: u32) {}

struct WaylandFns {
    marshal_new_no_args: PfnMarshalNewNoArgs,
    marshal_new_two_obj_args: PfnMarshalNewTwoObjArgs,
    marshal_bind: PfnMarshalBind,
    marshal_no_args: PfnMarshalNoArgs,
    marshal_one_obj_arg: PfnMarshalOneObjArg,
    marshal_attach: PfnMarshalAttach,
    proxy_add_listener: PfnProxyAddListener,
    proxy_destroy: PfnProxyDestroy,
    proxy_get_version: PfnProxyGetVersion,
    proxy_set_queue: PfnProxySetQueue,
    display_create_queue: PfnDisplayCreateQueue,
    display_roundtrip_queue: PfnDisplayRoundtripQueue,
    event_queue_destroy: PfnEventQueueDestroy,
}

impl WaylandFns {
    fn load() -> Result<Self, String> {
        let module = dlopen_first(&["libwayland-client.so.0", "libwayland-client.so"])
            .ok_or("libwayland-client.so.0 not found")?;
        macro_rules! sym {
            ($name:expr_2021) => {
                unsafe { std::mem::transmute(dlsym_typed(module, $name)?) }
            };
        }
        Ok(Self {
            marshal_new_no_args: sym!("wl_proxy_marshal_flags"),
            marshal_new_two_obj_args: sym!("wl_proxy_marshal_flags"),
            marshal_bind: sym!("wl_proxy_marshal_flags"),
            marshal_no_args: sym!("wl_proxy_marshal_flags"),
            marshal_one_obj_arg: sym!("wl_proxy_marshal_flags"),
            marshal_attach: sym!("wl_proxy_marshal_flags"),
            proxy_add_listener: sym!("wl_proxy_add_listener"),
            proxy_destroy: sym!("wl_proxy_destroy"),
            proxy_get_version: sym!("wl_proxy_get_version"),
            proxy_set_queue: sym!("wl_proxy_set_queue"),
            display_create_queue: sym!("wl_display_create_queue"),
            display_roundtrip_queue: sym!("wl_display_roundtrip_queue"),
            event_queue_destroy: sym!("wl_event_queue_destroy"),
        })
    }

    fn interface(&self, name: &str) -> Result<*const c_void, String> {
        let module = dlopen_first(&["libwayland-client.so.0", "libwayland-client.so"])
            .ok_or("libwayland-client.so.0 not found")?;
        dlsym_typed(module, name).map(|p| p as *const c_void)
    }
}

pub struct VideoSubsurface {
    fns: WaylandFns,
    surface: *mut c_void,
    subsurface: *mut c_void,
    subcompositor: *mut c_void,
    queue: *mut c_void,
}

impl VideoSubsurface {
    pub fn new(
        wl_display: *mut c_void,
        wl_compositor: *mut c_void,
        parent_wl_surface: *mut c_void,
    ) -> Result<Self, String> {
        let fns = WaylandFns::load()?;
        let registry_iface = fns.interface("wl_registry_interface")?;
        let subcompositor_iface = fns.interface("wl_subcompositor_interface")?;
        let surface_iface = fns.interface("wl_surface_interface")?;
        let subsurface_iface = fns.interface("wl_subsurface_interface")?;

        let queue = unsafe { (fns.display_create_queue)(wl_display) };
        if queue.is_null() {
            return Err("wl_display_create_queue failed".into());
        }

        let registry = unsafe {
            (fns.marshal_new_no_args)(
                wl_display,
                WL_DISPLAY_GET_REGISTRY,
                registry_iface,
                (fns.proxy_get_version)(wl_display),
                0,
                ptr::null(),
            )
        };
        if registry.is_null() {
            unsafe { (fns.event_queue_destroy)(queue) };
            return Err("wl_display_get_registry failed".into());
        }
        unsafe { (fns.proxy_set_queue)(registry, queue) };

        let mut found: Option<FoundGlobal> = None;
        let listener = WlRegistryListener {
            global: on_global,
            global_remove: on_global_remove,
        };
        let rc = unsafe {
            (fns.proxy_add_listener)(
                registry,
                &listener as *const _ as *const c_void,
                &mut found as *mut _ as *mut c_void,
            )
        };
        if rc != 0 {
            unsafe {
                (fns.proxy_destroy)(registry);
                (fns.event_queue_destroy)(queue);
            }
            return Err("wl_proxy_add_listener failed".into());
        }

        if unsafe { (fns.display_roundtrip_queue)(wl_display, queue) } < 0 {
            unsafe {
                (fns.proxy_destroy)(registry);
                (fns.event_queue_destroy)(queue);
            }
            return Err("wl_display_roundtrip_queue failed".into());
        }

        let Some(global) = found else {
            unsafe {
                (fns.proxy_destroy)(registry);
                (fns.event_queue_destroy)(queue);
            }
            return Err("compositor does not advertise wl_subcompositor".into());
        };

        let subcompositor = unsafe {
            (fns.marshal_bind)(
                registry,
                WL_REGISTRY_BIND,
                subcompositor_iface,
                global.version,
                0,
                global.name,
                b"wl_subcompositor\0".as_ptr() as *const i8,
                global.version,
                ptr::null(),
            )
        };
        unsafe { (fns.proxy_destroy)(registry) };
        if subcompositor.is_null() {
            unsafe { (fns.event_queue_destroy)(queue) };
            return Err("wl_registry_bind(wl_subcompositor) failed".into());
        }

        let surface = unsafe {
            (fns.marshal_new_no_args)(
                wl_compositor,
                WL_COMPOSITOR_CREATE_SURFACE,
                surface_iface,
                (fns.proxy_get_version)(wl_compositor),
                0,
                ptr::null(),
            )
        };
        if surface.is_null() {
            unsafe {
                (fns.proxy_destroy)(subcompositor);
                (fns.event_queue_destroy)(queue);
            }
            return Err("wl_compositor_create_surface failed".into());
        }

        let subsurface = unsafe {
            (fns.marshal_new_two_obj_args)(
                subcompositor,
                WL_SUBCOMPOSITOR_GET_SUBSURFACE,
                subsurface_iface,
                (fns.proxy_get_version)(subcompositor),
                0,
                ptr::null(),
                surface,
                parent_wl_surface,
            )
        };
        if subsurface.is_null() {
            unsafe {
                (fns.proxy_destroy)(surface);
                (fns.proxy_destroy)(subcompositor);
                (fns.event_queue_destroy)(queue);
            }
            return Err("wl_subcompositor_get_subsurface failed".into());
        }

        unsafe {
            (fns.marshal_no_args)(
                subsurface,
                WL_SUBSURFACE_SET_DESYNC,
                ptr::null(),
                (fns.proxy_get_version)(subsurface),
                0,
            );
            (fns.marshal_one_obj_arg)(
                subsurface,
                WL_SUBSURFACE_PLACE_BELOW,
                ptr::null(),
                (fns.proxy_get_version)(subsurface),
                0,
                parent_wl_surface,
            );
            (fns.marshal_no_args)(
                surface,
                WL_SURFACE_COMMIT,
                ptr::null(),
                (fns.proxy_get_version)(surface),
                0,
            );
        }

        Ok(Self {
            fns,
            surface,
            subsurface,
            subcompositor,
            queue,
        })
    }

    pub fn wl_surface(&self) -> *mut c_void {
        self.surface
    }

    pub fn hide(&self) {
        unsafe {
            (self.fns.marshal_attach)(
                self.surface,
                WL_SURFACE_ATTACH,
                ptr::null(),
                (self.fns.proxy_get_version)(self.surface),
                0,
                ptr::null_mut(),
                0,
                0,
            );
            (self.fns.marshal_no_args)(
                self.surface,
                WL_SURFACE_COMMIT,
                ptr::null(),
                (self.fns.proxy_get_version)(self.surface),
                0,
            );
        }
    }
}

impl Drop for VideoSubsurface {
    fn drop(&mut self) {
        unsafe {
            (self.fns.marshal_no_args)(
                self.subsurface,
                WL_SUBSURFACE_DESTROY,
                ptr::null(),
                (self.fns.proxy_get_version)(self.subsurface),
                0,
            );
            (self.fns.proxy_destroy)(self.subsurface);
            (self.fns.marshal_no_args)(
                self.surface,
                WL_SURFACE_DESTROY,
                ptr::null(),
                (self.fns.proxy_get_version)(self.surface),
                0,
            );
            (self.fns.proxy_destroy)(self.surface);
            (self.fns.proxy_destroy)(self.subcompositor);
            (self.fns.event_queue_destroy)(self.queue);
        }
    }
}
