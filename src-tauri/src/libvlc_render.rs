use crate::mpv_render::{PlayerEvent, PlayerStatus, PlayerTrackOption};
use crate::playback_engine::PlaybackEngine;
use libloading::Library;
use std::collections::VecDeque;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Arc, Mutex};

const LIBVLC_NOTHING_SPECIAL: c_int = 0;
const LIBVLC_PLAYING: c_int = 3;
const LIBVLC_PAUSED: c_int = 4;
const LIBVLC_ENDED: c_int = 6;

const LIBVLC_MEDIA_PLAYER_END_REACHED: c_int = 265;
const LIBVLC_MEDIA_PLAYER_ENCOUNTERED_ERROR: c_int = 266;

const LIBVLC_MEDIA_SLAVE_TYPE_SUBTITLE: c_int = 0;

#[repr(C)]
struct LibvlcEvent {
    event_type: c_int,
    p_obj: *mut c_void,
}

#[repr(C)]
struct LibvlcTrackDescription {
    i_id: c_int,
    psz_name: *mut c_char,
    p_next: *mut LibvlcTrackDescription,
}

type LibvlcNew = unsafe extern "C" fn(c_int, *const *const c_char) -> *mut c_void;
type LibvlcErrmsg = unsafe extern "C" fn() -> *const c_char;
type LibvlcLogCallback =
    unsafe extern "C" fn(*mut c_void, c_int, *const c_void, *const c_char, *mut c_void);
type LibvlcLogSet = unsafe extern "C" fn(*mut c_void, LibvlcLogCallback, *mut c_void);
type LibvlcRelease = unsafe extern "C" fn(*mut c_void);
type LibvlcMediaNewLocation = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;
type LibvlcMediaNewPath = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;
type LibvlcMediaAddOption = unsafe extern "C" fn(*mut c_void, *const c_char);
type LibvlcMediaRelease = unsafe extern "C" fn(*mut c_void);
type LibvlcMediaPlayerNewFromMedia = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type LibvlcMediaPlayerRelease = unsafe extern "C" fn(*mut c_void);
type LibvlcMediaPlayerSetMedia = unsafe extern "C" fn(*mut c_void, *mut c_void);
type LibvlcMediaPlayerPlay = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcMediaPlayerSetPause = unsafe extern "C" fn(*mut c_void, c_int);
type LibvlcMediaPlayerStop = unsafe extern "C" fn(*mut c_void);
type LibvlcMediaPlayerGetState = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcMediaPlayerGetTime = unsafe extern "C" fn(*mut c_void) -> i64;
type LibvlcMediaPlayerSetTime = unsafe extern "C" fn(*mut c_void, i64);
type LibvlcMediaPlayerGetLength = unsafe extern "C" fn(*mut c_void) -> i64;
type LibvlcMediaPlayerGetPosition = unsafe extern "C" fn(*mut c_void) -> f32;
type LibvlcAudioOutputSet = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type LibvlcAudioGetVolume = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcAudioSetVolume = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type LibvlcAudioGetMute = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcAudioSetMute = unsafe extern "C" fn(*mut c_void, c_int);
type LibvlcMediaPlayerAddSlave =
    unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int) -> c_int;
type LibvlcVideoGetSpu = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcVideoSetSpu = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type LibvlcVideoGetSpuDescription =
    unsafe extern "C" fn(*mut c_void) -> *mut LibvlcTrackDescription;
type LibvlcAudioGetTrack = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcAudioSetTrack = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type LibvlcAudioGetTrackDescription =
    unsafe extern "C" fn(*mut c_void) -> *mut LibvlcTrackDescription;
type LibvlcTrackDescriptionListRelease = unsafe extern "C" fn(*mut LibvlcTrackDescription);
type LibvlcMediaPlayerHasVout = unsafe extern "C" fn(*mut c_void) -> c_int;
type LibvlcVideoGetSize = unsafe extern "C" fn(*mut c_void, u32, *mut u32, *mut u32) -> c_int;
type LibvlcMediaPlayerEventManager = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type LibvlcEventCallback = unsafe extern "C" fn(*const LibvlcEvent, *mut c_void);
type LibvlcEventAttach =
    unsafe extern "C" fn(*mut c_void, c_int, LibvlcEventCallback, *mut c_void) -> c_int;

#[cfg(target_os = "windows")]
type LibvlcMediaPlayerSetHwnd = unsafe extern "C" fn(*mut c_void, *mut c_void);
#[cfg(target_os = "macos")]
type LibvlcMediaPlayerSetNsobject = unsafe extern "C" fn(*mut c_void, *mut c_void);
#[cfg(target_os = "linux")]
type LibvlcMediaPlayerSetXwindow = unsafe extern "C" fn(*mut c_void, u32);

struct VlcApi {
    _library: Library,
    lib_path: PathBuf,
    plugin_path: Option<PathBuf>,
    new: LibvlcNew,
    errmsg: LibvlcErrmsg,
    log_set: LibvlcLogSet,
    release: LibvlcRelease,
    media_new_location: LibvlcMediaNewLocation,
    media_new_path: LibvlcMediaNewPath,
    media_add_option: LibvlcMediaAddOption,
    media_release: LibvlcMediaRelease,
    media_player_new_from_media: LibvlcMediaPlayerNewFromMedia,
    media_player_release: LibvlcMediaPlayerRelease,
    media_player_set_media: LibvlcMediaPlayerSetMedia,
    media_player_play: LibvlcMediaPlayerPlay,
    media_player_set_pause: LibvlcMediaPlayerSetPause,
    media_player_stop: LibvlcMediaPlayerStop,
    media_player_get_state: LibvlcMediaPlayerGetState,
    media_player_get_time: LibvlcMediaPlayerGetTime,
    media_player_set_time: LibvlcMediaPlayerSetTime,
    media_player_get_length: LibvlcMediaPlayerGetLength,
    media_player_get_position: LibvlcMediaPlayerGetPosition,
    audio_output_set: LibvlcAudioOutputSet,
    audio_get_volume: LibvlcAudioGetVolume,
    audio_set_volume: LibvlcAudioSetVolume,
    audio_get_mute: LibvlcAudioGetMute,
    audio_set_mute: LibvlcAudioSetMute,
    media_player_add_slave: LibvlcMediaPlayerAddSlave,
    video_get_spu: LibvlcVideoGetSpu,
    video_set_spu: LibvlcVideoSetSpu,
    video_get_spu_description: LibvlcVideoGetSpuDescription,
    audio_get_track: LibvlcAudioGetTrack,
    audio_set_track: LibvlcAudioSetTrack,
    audio_get_track_description: LibvlcAudioGetTrackDescription,
    track_description_list_release: LibvlcTrackDescriptionListRelease,
    media_player_has_vout: LibvlcMediaPlayerHasVout,
    video_get_size: LibvlcVideoGetSize,
    media_player_event_manager: LibvlcMediaPlayerEventManager,
    event_attach: LibvlcEventAttach,
    #[cfg(target_os = "windows")]
    media_player_set_hwnd: LibvlcMediaPlayerSetHwnd,
    #[cfg(target_os = "macos")]
    media_player_set_nsobject: LibvlcMediaPlayerSetNsobject,
    #[cfg(target_os = "linux")]
    media_player_set_xwindow: LibvlcMediaPlayerSetXwindow,
}

unsafe impl Send for VlcApi {}

fn load_error(error: libloading::Error) -> String {
    error.to_string()
}

#[cfg(target_os = "windows")]
fn configure_windows_dll_path(lib_path: &str) -> Result<(), String> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetDllDirectoryW(path: *const u16) -> i32;
    }

    let lib_dir = Path::new(lib_path)
        .parent()
        .ok_or_else(|| format!("libvlc path has no parent directory: '{lib_path}'"))?;
    let wide_path = lib_dir
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    if unsafe { SetDllDirectoryW(wide_path.as_ptr()) } == 0 {
        return Err(format!(
            "failed to add the VLC directory to the Windows DLL search path: '{}'",
            lib_dir.display()
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn configure_windows_dll_path(_lib_path: &str) -> Result<(), String> {
    Ok(())
}
impl VlcApi {
    fn load() -> Result<Self, String> {
        let lib_path = find_libvlc_path();
        configure_windows_dll_path(&lib_path)?;
        let plugin_path = configure_plugin_path(&lib_path);
        #[cfg(target_os = "windows")]
        let library: Library = unsafe {
            const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x0000_0100;
            const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x0000_1000;
            libloading::os::windows::Library::load_with_flags(
                &lib_path,
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            )
        }
        .map(Into::into)
        .map_err(|e| format!("failed to load libvlc from '{lib_path}': {e}"))?;
        #[cfg(not(target_os = "windows"))]
        let library = unsafe { Library::new(&lib_path) }
            .map_err(|e| format!("failed to load libvlc from '{lib_path}': {e}"))?;
        unsafe {
            macro_rules! sym {
                ($name:literal) => {
                    *library.get($name).map_err(load_error)?
                };
            }
            Ok(Self {
                new: sym!(b"libvlc_new\0"),
                errmsg: sym!(b"libvlc_errmsg\0"),
                log_set: sym!(b"libvlc_log_set\0"),
                release: sym!(b"libvlc_release\0"),
                media_new_location: sym!(b"libvlc_media_new_location\0"),
                media_new_path: sym!(b"libvlc_media_new_path\0"),
                media_add_option: sym!(b"libvlc_media_add_option\0"),
                media_release: sym!(b"libvlc_media_release\0"),
                media_player_new_from_media: sym!(b"libvlc_media_player_new_from_media\0"),
                media_player_release: sym!(b"libvlc_media_player_release\0"),
                media_player_set_media: sym!(b"libvlc_media_player_set_media\0"),
                media_player_play: sym!(b"libvlc_media_player_play\0"),
                media_player_set_pause: sym!(b"libvlc_media_player_set_pause\0"),
                media_player_stop: sym!(b"libvlc_media_player_stop\0"),
                media_player_get_state: sym!(b"libvlc_media_player_get_state\0"),
                media_player_get_time: sym!(b"libvlc_media_player_get_time\0"),
                media_player_set_time: sym!(b"libvlc_media_player_set_time\0"),
                media_player_get_length: sym!(b"libvlc_media_player_get_length\0"),
                media_player_get_position: sym!(b"libvlc_media_player_get_position\0"),
                audio_output_set: sym!(b"libvlc_audio_output_set\0"),
                audio_get_volume: sym!(b"libvlc_audio_get_volume\0"),
                audio_set_volume: sym!(b"libvlc_audio_set_volume\0"),
                audio_get_mute: sym!(b"libvlc_audio_get_mute\0"),
                audio_set_mute: sym!(b"libvlc_audio_set_mute\0"),
                media_player_add_slave: sym!(b"libvlc_media_player_add_slave\0"),
                video_get_spu: sym!(b"libvlc_video_get_spu\0"),
                video_set_spu: sym!(b"libvlc_video_set_spu\0"),
                video_get_spu_description: sym!(b"libvlc_video_get_spu_description\0"),
                audio_get_track: sym!(b"libvlc_audio_get_track\0"),
                audio_set_track: sym!(b"libvlc_audio_set_track\0"),
                audio_get_track_description: sym!(b"libvlc_audio_get_track_description\0"),
                track_description_list_release: sym!(b"libvlc_track_description_list_release\0"),
                media_player_has_vout: sym!(b"libvlc_media_player_has_vout\0"),
                video_get_size: sym!(b"libvlc_video_get_size\0"),
                media_player_event_manager: sym!(b"libvlc_media_player_event_manager\0"),
                event_attach: sym!(b"libvlc_event_attach\0"),
                #[cfg(target_os = "windows")]
                media_player_set_hwnd: sym!(b"libvlc_media_player_set_hwnd\0"),
                #[cfg(target_os = "macos")]
                media_player_set_nsobject: sym!(b"libvlc_media_player_set_nsobject\0"),
                #[cfg(target_os = "linux")]
                media_player_set_xwindow: sym!(b"libvlc_media_player_set_xwindow\0"),
                lib_path: PathBuf::from(lib_path),
                plugin_path,
                _library: library,
            })
        }
    }

    fn last_error(&self) -> Option<String> {
        let message = unsafe { (self.errmsg)() };
        if message.is_null() {
            return None;
        }
        let message = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .trim()
            .to_string();
        (!message.is_empty()).then_some(message)
    }
}

fn find_libvlc_path() -> String {
    #[cfg(target_os = "windows")]
    let lib_names: &[&str] = &["libvlc.dll"];
    #[cfg(target_os = "macos")]
    let lib_names: &[&str] = &["libvlc.dylib", "libvlc.5.dylib"];
    #[cfg(target_os = "linux")]
    let lib_names: &[&str] = &["libvlc.so.5", "libvlc.so"];

    let mut search_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            search_dirs.push(exe_dir.to_path_buf());
            search_dirs.push(exe_dir.join("vlc"));
            search_dirs.push(exe_dir.join("lib"));
            search_dirs.push(exe_dir.join("lib").join("vlc"));
            #[cfg(target_os = "macos")]
            if let Some(contents_dir) = exe_dir.parent() {
                search_dirs.push(contents_dir.join("Resources").join("lib"));
                search_dirs.push(contents_dir.join("Resources").join("lib").join("vlc"));
                search_dirs.push(contents_dir.join("Frameworks"));
            }
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        search_dirs.push(PathBuf::from(&manifest_dir).join("lib"));
        search_dirs.push(PathBuf::from(&manifest_dir).join("lib").join("vlc"));
    }
    #[cfg(target_os = "macos")]
    {
        search_dirs.push(PathBuf::from("/opt/homebrew/lib"));
        search_dirs.push(PathBuf::from("/usr/local/lib"));
        search_dirs.push(PathBuf::from("/Applications/VLC.app/Contents/MacOS/lib"));
    }

    for dir in &search_dirs {
        for name in lib_names {
            let path = dir.join(name);
            if path.exists() {
                return path.to_string_lossy().into_owned();
            }
        }
    }

    #[cfg(target_os = "windows")]
    return "libvlc.dll".to_string();
    #[cfg(target_os = "macos")]
    return "libvlc.dylib".to_string();
    #[cfg(target_os = "linux")]
    return "libvlc.so.5".to_string();
}

fn configure_plugin_path(lib_path: &str) -> Option<PathBuf> {
    let lib_dir = Path::new(lib_path).parent()?;
    let plugin_dir = lib_dir.join("plugins");
    if plugin_dir.is_dir() {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { std::env::set_var("VLC_PLUGIN_PATH", &plugin_dir) };
        Some(plugin_dir)
    } else {
        None
    }
}
unsafe extern "C" fn vlc_log_callback(
    _data: *mut c_void,
    level: c_int,
    _context: *const c_void,
    format: *const c_char,
    _args: *mut c_void,
) {
    if format.is_null() {
        return;
    }
    let message = unsafe { CStr::from_ptr(format) }.to_string_lossy();
    match level {
        4 => log::error!("[libvlc] {message}"),
        3 => log::warn!("[libvlc] {message}"),
        2 => log::info!("[libvlc] {message}"),
        _ => log::debug!("[libvlc] {message}"),
    }
}

#[derive(Default)]
struct VlcEventQueue {
    events: VecDeque<PlayerEvent>,
}

unsafe extern "C" fn vlc_event_callback(event: *const LibvlcEvent, user_data: *mut c_void) {
    if event.is_null() || user_data.is_null() {
        return;
    }
    let event_type = unsafe { (*event).event_type };
    let queue = unsafe { &*(user_data as *const Mutex<VlcEventQueue>) };
    let mut queue = match queue.lock() {
        Ok(q) => q,
        Err(_) => return,
    };
    if event_type == LIBVLC_MEDIA_PLAYER_END_REACHED {
        queue.events.push_back(PlayerEvent::EndFile {
            eof: true,
            error: None,
        });
    } else if event_type == LIBVLC_MEDIA_PLAYER_ENCOUNTERED_ERROR {
        log::error!("[libvlc] media player encountered an asynchronous playback error");
        queue.events.push_back(PlayerEvent::EndFile {
            eof: false,
            error: Some("libvlc encountered an error during playback".to_string()),
        });
    }
}

pub struct LibvlcPlayer {
    api: VlcApi,
    instance: *mut c_void,
    media_player: *mut c_void,
    event_queue: Arc<Mutex<VlcEventQueue>>,
    current_url: Option<String>,
    pending_headers: Vec<(String, String)>,
    loaded: bool,
}

unsafe impl Send for LibvlcPlayer {}

impl LibvlcPlayer {
    pub fn new() -> Result<Self, String> {
        let api = VlcApi::load()?;
        let instance = unsafe { (api.new)(0, ptr::null()) };
        if instance.is_null() {
            let vlc_error = api
                .last_error()
                .unwrap_or_else(|| "LibVLC did not provide an error message".to_string());
            let plugin_path = api
                .plugin_path
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "not found next to libvlc".to_string());
            return Err(format!(
                "libvlc_new returned null: {vlc_error} (libvlc: '{}', plugins: '{}')",
                api.lib_path.display(),
                plugin_path
            ));
        }
        unsafe { (api.log_set)(instance, vlc_log_callback, ptr::null_mut()) };
        log::info!("LibVLC initialized with direct application-log callback");
        Ok(Self {
            api,
            instance,
            media_player: ptr::null_mut(),
            event_queue: Arc::new(Mutex::new(VlcEventQueue::default())),
            current_url: None,
            pending_headers: Vec::new(),
            loaded: false,
        })
    }

    #[cfg(target_os = "windows")]
    pub fn attach_hwnd(&mut self, hwnd: *mut c_void) -> Result<(), String> {
        self.ensure_media_player()?;
        unsafe { (self.api.media_player_set_hwnd)(self.media_player, hwnd) };
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn attach_nsobject(&mut self, nsview: *mut c_void) -> Result<(), String> {
        self.ensure_media_player()?;
        unsafe { (self.api.media_player_set_nsobject)(self.media_player, nsview) };
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn attach_xwindow(&mut self, xid: u32) -> Result<(), String> {
        self.ensure_media_player()?;
        unsafe { (self.api.media_player_set_xwindow)(self.media_player, xid) };
        Ok(())
    }

    fn ensure_media_player(&mut self) -> Result<(), String> {
        if !self.media_player.is_null() {
            return Ok(());
        }
        let dummy_url = CString::new("vlc://nop").unwrap();
        let media = unsafe { (self.api.media_new_location)(self.instance, dummy_url.as_ptr()) };
        if media.is_null() {
            return Err("libvlc_media_new_location failed".to_string());
        }
        let mp = unsafe { (self.api.media_player_new_from_media)(media) };
        unsafe { (self.api.media_release)(media) };
        if mp.is_null() {
            return Err("libvlc_media_player_new_from_media failed".to_string());
        }
        let mgr = unsafe { (self.api.media_player_event_manager)(mp) };
        if !mgr.is_null() {
            let user_data = Arc::into_raw(self.event_queue.clone()) as *mut c_void;
            unsafe {
                (self.api.event_attach)(
                    mgr,
                    LIBVLC_MEDIA_PLAYER_END_REACHED,
                    vlc_event_callback,
                    user_data,
                );
                (self.api.event_attach)(
                    mgr,
                    LIBVLC_MEDIA_PLAYER_ENCOUNTERED_ERROR,
                    vlc_event_callback,
                    user_data,
                );
            }
        }
        self.media_player = mp;
        #[cfg(target_os = "windows")]
        {
            let output = CString::new("mmdevice").unwrap();
            let result = unsafe { (self.api.audio_output_set)(self.media_player, output.as_ptr()) };
            if result != 0 {
                log::warn!("LibVLC could not select the Windows mmdevice audio output");
            }
        }
        unsafe {
            (self.api.audio_set_mute)(self.media_player, 0);
            (self.api.audio_set_volume)(self.media_player, 100);
        }
        Ok(())
    }

    fn state(&self) -> c_int {
        if self.media_player.is_null() {
            return LIBVLC_NOTHING_SPECIAL;
        }
        unsafe { (self.api.media_player_get_state)(self.media_player) }
    }
}

impl PlaybackEngine for LibvlcPlayer {
    fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        self.ensure_media_player()?;
        self.current_url = Some(url.to_string());

        let media = if url.contains("://") {
            let c_url = CString::new(url).map_err(|e| e.to_string())?;
            unsafe { (self.api.media_new_location)(self.instance, c_url.as_ptr()) }
        } else {
            let c_path = CString::new(url).map_err(|e| e.to_string())?;
            unsafe { (self.api.media_new_path)(self.instance, c_path.as_ptr()) }
        };
        if media.is_null() {
            return Err("libvlc failed to create media for this URL".to_string());
        }

        for (key, value) in &self.pending_headers {
            let opt = match key.to_lowercase().as_str() {
                "user-agent" => Some(format!(":http-user-agent={value}")),
                "referer" | "referrer" => Some(format!(":http-referrer={value}")),
                _ => None,
            };
            if let Some(opt) = opt {
                if let Ok(c_opt) = CString::new(opt) {
                    unsafe { (self.api.media_add_option)(media, c_opt.as_ptr()) };
                }
            }
        }

        unsafe { (self.api.media_player_set_media)(self.media_player, media) };
        unsafe { (self.api.media_release)(media) };

        let result = unsafe { (self.api.media_player_play)(self.media_player) };
        if result != 0 {
            return Err("libvlc_media_player_play failed".to_string());
        }
        if let Some(start_at) = start_at.filter(|&s| s > 0) {
            unsafe {
                (self.api.media_player_set_time)(self.media_player, (start_at * 1000) as i64)
            };
        }
        self.loaded = true;
        Ok(())
    }

    fn command_string(&self, command: &str) -> Result<(), String> {
        if self.media_player.is_null() {
            return Err("libvlc media player not created yet".to_string());
        }

        let set_time_seconds = |seconds: f64| {
            let length_ms = unsafe { (self.api.media_player_get_length)(self.media_player) };
            let mut target_ms = (seconds * 1000.0).round() as i64;
            target_ms = target_ms.max(0);
            if length_ms > 0 {
                target_ms = target_ms.min(length_ms);
            }
            unsafe { (self.api.media_player_set_time)(self.media_player, target_ms) };
        };

        match command {
            "stop" => {
                unsafe { (self.api.media_player_stop)(self.media_player) };
                Ok(())
            }
            "cycle pause" => {
                let pause = if self.state() == LIBVLC_PAUSED { 0 } else { 1 };
                unsafe { (self.api.media_player_set_pause)(self.media_player, pause) };
                Ok(())
            }
            "set pause yes" => {
                unsafe { (self.api.media_player_set_pause)(self.media_player, 1) };
                Ok(())
            }
            "set pause no" => {
                unsafe { (self.api.media_player_set_pause)(self.media_player, 0) };
                Ok(())
            }
            "cycle mute" => {
                let muted = unsafe { (self.api.audio_get_mute)(self.media_player) } != 0;
                unsafe { (self.api.audio_set_mute)(self.media_player, if muted { 0 } else { 1 }) };
                Ok(())
            }
            "set mute yes" => {
                unsafe { (self.api.audio_set_mute)(self.media_player, 1) };
                Ok(())
            }
            "set mute no" => {
                unsafe { (self.api.audio_set_mute)(self.media_player, 0) };
                Ok(())
            }
            other if other.starts_with("set volume ") => {
                let volume = other
                    .split_whitespace()
                    .nth(2)
                    .and_then(|value| value.parse::<f64>().ok())
                    .ok_or_else(|| format!("unrecognized volume command: {other}"))?;
                let result = unsafe {
                    (self.api.audio_set_volume)(
                        self.media_player,
                        volume.round().clamp(0.0, 100.0) as c_int,
                    )
                };
                if result == 0 {
                    Ok(())
                } else {
                    Err("libvlc failed to set volume".to_string())
                }
            }
            other if other.starts_with("add volume ") => {
                let delta = other
                    .split_whitespace()
                    .nth(2)
                    .and_then(|value| value.parse::<f64>().ok())
                    .ok_or_else(|| format!("unrecognized volume command: {other}"))?;
                let current =
                    unsafe { (self.api.audio_get_volume)(self.media_player) }.max(0) as f64;
                let result = unsafe {
                    (self.api.audio_set_volume)(
                        self.media_player,
                        (current + delta).round().clamp(0.0, 100.0) as c_int,
                    )
                };
                if result == 0 {
                    Ok(())
                } else {
                    Err("libvlc failed to adjust volume".to_string())
                }
            }
            other if other.starts_with("set aid ") => {
                let track_id = other
                    .split_whitespace()
                    .nth(2)
                    .and_then(|value| value.parse::<c_int>().ok())
                    .ok_or_else(|| format!("unrecognized audio-track command: {other}"))?;
                let result = unsafe { (self.api.audio_set_track)(self.media_player, track_id) };
                if result == 0 {
                    Ok(())
                } else {
                    Err(format!("libvlc failed to select audio track {track_id}"))
                }
            }
            other if other.starts_with("set sid ") => {
                let track_id = other
                    .split_whitespace()
                    .nth(2)
                    .and_then(|value| value.parse::<c_int>().ok())
                    .ok_or_else(|| format!("unrecognized subtitle-track command: {other}"))?;
                let result = unsafe { (self.api.video_set_spu)(self.media_player, track_id) };
                if result == 0 {
                    Ok(())
                } else {
                    Err(format!("libvlc failed to select subtitle track {track_id}"))
                }
            }
            other if other.starts_with("set time-pos ") => {
                let seconds = other
                    .split_whitespace()
                    .nth(2)
                    .and_then(|value| value.parse::<f64>().ok())
                    .ok_or_else(|| format!("unrecognized time-pos command: {other}"))?;
                set_time_seconds(seconds);
                Ok(())
            }
            other if other.starts_with("seek ") => {
                let parts = other.split_whitespace().collect::<Vec<_>>();
                let amount = parts
                    .get(1)
                    .and_then(|value| value.parse::<f64>().ok())
                    .ok_or_else(|| format!("unrecognized seek command: {other}"))?;
                let mode = parts.get(2).copied().unwrap_or("relative");
                let current_seconds =
                    unsafe { (self.api.media_player_get_time)(self.media_player) } as f64 / 1000.0;
                let length_seconds =
                    unsafe { (self.api.media_player_get_length)(self.media_player) } as f64
                        / 1000.0;
                let target_seconds = match mode {
                    "relative" | "relative+exact" => current_seconds + amount,
                    "absolute" | "absolute+exact" => amount,
                    "absolute-percent" => {
                        if length_seconds <= 0.0 {
                            return Err(
                                "libvlc cannot seek by percentage before duration is known"
                                    .to_string(),
                            );
                        }
                        length_seconds * amount.clamp(0.0, 100.0) / 100.0
                    }
                    _ => return Err(format!("unsupported libvlc seek mode: {mode}")),
                };
                set_time_seconds(target_seconds);
                Ok(())
            }
            other => Err(format!("libvlc engine does not support command: {other}")),
        }
    }
    fn command_args(&self, args: &[&str]) -> Result<(), String> {
        Err(format!(
            "libvlc engine does not support command_args: {}",
            args.join(" ")
        ))
    }

    fn apply_options(&self, _options: &[(String, String)]) -> Result<(), String> {
        Ok(())
    }

    fn set_http_headers(&self, headers: &[(String, String)]) -> Result<(), String> {
        Err(format!(
            "libvlc engine applies HTTP headers at load time only ({} pending)",
            headers.len()
        ))
    }

    fn add_subtitle(
        &self,
        url: &str,
        _title: Option<&str>,
        _language: Option<&str>,
    ) -> Result<(), String> {
        if self.media_player.is_null() {
            return Err("libvlc media player not created yet".to_string());
        }
        let c_url = CString::new(url).map_err(|e| e.to_string())?;
        let result = unsafe {
            (self.api.media_player_add_slave)(
                self.media_player,
                LIBVLC_MEDIA_SLAVE_TYPE_SUBTITLE,
                c_url.as_ptr(),
                1,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err("libvlc_media_player_add_slave failed".to_string())
        }
    }

    fn query_property(&self, name: &str) -> Option<String> {
        if self.media_player.is_null() {
            return None;
        }
        match name {
            "path" => self.current_url.clone(),
            "time-pos" => Some(
                (unsafe { (self.api.media_player_get_time)(self.media_player) } as f64 / 1000.0)
                    .to_string(),
            ),
            "duration" => Some(
                (unsafe { (self.api.media_player_get_length)(self.media_player) } as f64 / 1000.0)
                    .to_string(),
            ),
            _ => None,
        }
    }

    fn status(&self) -> PlayerStatus {
        let state = self.state();
        let time_ms = if self.media_player.is_null() {
            0
        } else {
            unsafe { (self.api.media_player_get_time)(self.media_player) }
        };
        let length_ms = if self.media_player.is_null() {
            0
        } else {
            unsafe { (self.api.media_player_get_length)(self.media_player) }
        };
        let position = if self.media_player.is_null() {
            0.0
        } else {
            unsafe { (self.api.media_player_get_position)(self.media_player) }
        };
        if !self.media_player.is_null()
            && (state == LIBVLC_PLAYING || state == LIBVLC_PAUSED)
            && unsafe { (self.api.audio_get_track)(self.media_player) } < 0
        {
            let list = unsafe { (self.api.audio_get_track_description)(self.media_player) };
            let mut node = list;
            while !node.is_null() {
                let entry = unsafe { &*node };
                if entry.i_id > 0 {
                    let _ = unsafe { (self.api.audio_set_track)(self.media_player, entry.i_id) };
                    break;
                }
                node = entry.p_next;
            }
            if !list.is_null() {
                unsafe { (self.api.track_description_list_release)(list) };
            }
        }
        let volume = if self.media_player.is_null() {
            100
        } else {
            unsafe { (self.api.audio_get_volume)(self.media_player) }
        };
        let mute = if self.media_player.is_null() {
            false
        } else {
            unsafe { (self.api.audio_get_mute)(self.media_player) != 0 }
        };
        let has_video = !self.media_player.is_null()
            && unsafe { (self.api.media_player_has_vout)(self.media_player) } != 0;
        let mut video_width = 0u32;
        let mut video_height = 0u32;
        let has_video_size = has_video
            && unsafe {
                (self.api.video_get_size)(self.media_player, 0, &mut video_width, &mut video_height)
            } == 0
            && video_width > 0
            && video_height > 0;
        let first_frame_presented =
            has_video_size && (state == LIBVLC_PLAYING || state == LIBVLC_PAUSED);

        PlayerStatus {
            loaded: self.loaded,
            path: self.current_url.clone(),
            media_title: None,
            time_pos: Some((time_ms as f64 / 1000.0).to_string()),
            duration: Some((length_ms as f64 / 1000.0).to_string()),
            percent_pos: Some((position * 100.0).to_string()),
            pause: Some(if state == LIBVLC_PAUSED { "yes" } else { "no" }.to_string()),
            mute: Some(if mute { "yes" } else { "no" }.to_string()),
            volume: Some(volume.to_string()),
            core_idle: None,
            eof_reached: Some(if state == LIBVLC_ENDED { "yes" } else { "no" }.to_string()),
            vo_configured: Some(if has_video { "yes" } else { "no" }.to_string()),
            video_codec: None,
            video_format: None,
            width: has_video_size.then(|| video_width.to_string()),
            height: has_video_size.then(|| video_height.to_string()),
            cache_speed: None,
            demuxer_cache_duration: None,
            hwdec_current: None,
            fps: None,
            frame_drop_count: None,
            decoder_frame_drop_count: None,
            avsync: None,
            video_bitrate: None,
            audio_bitrate: None,
            audio_codec: None,
            audio_samplerate: None,
            audio_channels: None,
            color_primaries: None,
            color_matrix: None,
            color_gamma: None,
            video_out_primaries: None,
            video_out_matrix: None,
            video_out_gamma: None,
            sig_peak: None,
            hdr_active: false,
            container_fps: None,
            display_fps: None,
            mistimed_frame_count: None,
            vo_delayed_frame_count: None,
            paused_for_cache: None,
            cache_buffering_state: None,
            seeking: None,
            file_format: None,
            frames_rendered: if first_frame_presented { 2 } else { 0 },
            first_frame_presented,
            has_video_track: has_video,
            track_list_ready: state == LIBVLC_PLAYING || state == LIBVLC_PAUSED || length_ms > 0,
            resuming: false,
        }
    }

    fn track_options(&self, track_type: &str) -> Vec<PlayerTrackOption> {
        if self.media_player.is_null() {
            return Vec::new();
        }
        let (list_ptr, current_id) = match track_type {
            "audio" => (
                unsafe { (self.api.audio_get_track_description)(self.media_player) },
                unsafe { (self.api.audio_get_track)(self.media_player) },
            ),
            "sub" => (
                unsafe { (self.api.video_get_spu_description)(self.media_player) },
                unsafe { (self.api.video_get_spu)(self.media_player) },
            ),
            _ => return Vec::new(),
        };
        if list_ptr.is_null() {
            return Vec::new();
        }
        let mut tracks = Vec::new();
        let mut node = list_ptr;
        let mut index = 0;
        while !node.is_null() {
            let entry = unsafe { &*node };
            if entry.i_id >= 0 {
                let label = if entry.psz_name.is_null() {
                    format!("Track {index}")
                } else {
                    unsafe { CStr::from_ptr(entry.psz_name) }
                        .to_string_lossy()
                        .into_owned()
                };
                tracks.push(PlayerTrackOption {
                    id: entry.i_id.to_string(),
                    label,
                    selected: entry.i_id == current_id,
                    lang: None,
                    source: None,
                    external: false,
                    format: None,
                });
                index += 1;
            }
            node = entry.p_next;
        }
        unsafe { (self.api.track_description_list_release)(list_ptr) };
        tracks
    }

    fn title(&self) -> Option<String> {
        None
    }

    fn poll_events(&mut self) -> Vec<PlayerEvent> {
        let mut queue = match self.event_queue.lock() {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };
        queue.events.drain(..).collect()
    }
}

impl Drop for LibvlcPlayer {
    fn drop(&mut self) {
        unsafe {
            if !self.media_player.is_null() {
                (self.api.media_player_stop)(self.media_player);
                (self.api.media_player_release)(self.media_player);
            }
            if !self.instance.is_null() {
                (self.api.release)(self.instance);
            }
        }
    }
}
