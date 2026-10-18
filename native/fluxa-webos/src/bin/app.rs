use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::mpsc::channel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fluxa_host::{BridgeVideo, VideoStatus, FluxaHost, KeyInput, MouseButton, NativeSurface, VideoBridge};
use fluxa_renderer::platform::GraphicsBackend;
use fluxa_webos::bridge::pump;
use fluxa_webos::capabilities;
use fluxa_webos::demux::FfmpegDemuxer;
use fluxa_webos::crash;
use fluxa_webos::diagnostics::{Report, note, trace};
use fluxa_webos::player::Player;
use fluxa_webos::remote::{Remote, RepeatFilter, map};
use fluxa_webos::{lifecycle, screensaver};
use fluxa_webos::sdl::{Event, Sdl};
use fluxa_webos::{LoadOptions, StarfishPipeline};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};

const BACKENDS: &[GraphicsBackend] = &[GraphicsBackend::Gles];
const APP_ID: &str = "com.fluxa.app";

fn data_dir() -> PathBuf {
    std::env::var_os("FLUXA_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".fluxa")))
        .unwrap_or_else(|| PathBuf::from("/tmp/fluxa"))
}

fn target(display: usize, surface: usize) -> Result<wgpu::SurfaceTargetUnsafe, String> {
    let display = NonNull::new(display as *mut c_void).ok_or("no wl_display")?;
    let surface = NonNull::new(surface as *mut c_void).ok_or("no wl_surface")?;
    Ok(wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display))),
        raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface)),
    })
}

fn key(host: &FluxaHost, remote: Remote) {
    use fluxa_host::Key;
    let media = |name: &str| host.media_command(name, 0.0);
    match remote {
        Remote::Up => host.key_down(KeyInput::Key(Key::Up)),
        Remote::Down => host.key_down(KeyInput::Key(Key::Down)),
        Remote::Left => host.key_down(KeyInput::Key(Key::Left)),
        Remote::Right => host.key_down(KeyInput::Key(Key::Right)),
        Remote::Ok => host.key_down(KeyInput::Key(Key::Enter)),
        Remote::Back => host.key_down(KeyInput::Key(Key::Back)),
        Remote::Backspace => host.key_down(KeyInput::Backspace),
        Remote::Play => media("play"),
        Remote::Pause => media("pause"),
        Remote::PlayPause => media("toggle"),
        Remote::FastForward => media("fastForward"),
        Remote::Rewind => media("rewind"),
        Remote::Stop => media("stop"),
        Remote::PageUp
        | Remote::Exit
        | Remote::PageDown
        | Remote::PointerHidden
        | Remote::Red
        | Remote::Green
        | Remote::Yellow
        | Remote::Blue => {}
    }
}

fn run_player(
    caps: capabilities::Capabilities,
    options: LoadOptions,
    bridge: Arc<Mutex<VideoBridge>>,
    playing: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    let (events, rx) = channel();
    let mut player = match StarfishPipeline::open(options, events) {
        Ok(pipeline) => Player::new(FfmpegDemuxer::with_capabilities(caps), pipeline, rx),
        Err(error) => {
            trace(format!("starfish unavailable: {error:?}"));
            crash::capture(&format!("starfish unavailable: {error:?}"));
            eprintln!("starfish: {error:?}");
            while !stop.load(Ordering::Relaxed) {
                if let Ok(mut bridge) = bridge.lock() {
                    if bridge.take_requests().iter().any(|request| request["type"] == "load") {
                        bridge.set_status(VideoStatus {
                            error: Some("The TV's media pipeline could not be opened".into()),
                            ..VideoStatus::default()
                        });
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            return;
        }
    };
    while !stop.load(Ordering::Relaxed) {
        pump(&mut player, &bridge);
        let status = player.status();
        playing.store(status.duration > 0.0 && !status.paused, Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(2));
    }
    player.handle(&serde_json::json!({ "type": "stop" }));
}

fn main() {
    let _sentry = crash::install_default(env!("CARGO_PKG_VERSION"));
    let report = Report::collect();
    let os_major = report.webos_major();

    let sdl = match Sdl::open("Fluxa", 1920, 1080, true) {
        Ok(sdl) => sdl,
        Err(error) => {
            eprintln!("window: {error}");
            std::process::exit(1);
        }
    };
    let handles = match sdl.wayland_handles() {
        Ok(handles) => handles,
        Err(error) => {
            trace(format!("wayland handles: {error}"));
            eprintln!("window handles: {error}");
            std::process::exit(1);
        }
    };

    let data = data_dir();
    let _ = std::fs::create_dir_all(&data);
    let crash_file = data.join("crash.txt");
    crash::report_previous_crash(&crash_file);
    crash::install_signal_handlers(&crash_file);
    crash::report_compatibility(true, &data.join("compat-reported"), env!("CARGO_PKG_VERSION"));
    let host = FluxaHost::new(1.0, Some(data.join("artwork-cache")));
    host.set_form_factor("tv");
    host.set_platform("webos");
    let bridge = Arc::new(Mutex::new(VideoBridge::default()));
    host.set_video_backend(Box::new(BridgeVideo(bridge.clone())));
    if let Err(error) = host.start_session(data) {
        eprintln!("session: {error}");
    }

    let (width, height) = sdl.size();
    let (display, window) = (handles.display as usize, handles.surface as usize);
    let surface = unsafe { NativeSurface::new(BACKENDS, move || target(display, window)) };
    host.surface_created(surface, width, height);

    let mut caps = capabilities::query(os_major.unwrap_or(5));
    let webos_major = os_major.or(caps.platform_code).unwrap_or(5);
    if webos_major < 4 {
        caps.dolby_vision = Some(false);
    }
    note("fluxa.webos_major", webos_major.to_string());
    note("tv.supports_dts", format!("{:?}", caps.dts));
    note("tv.earc_eac3", caps.eac3_out.to_string());
    note("tv.supports_hdr", format!("{:?}", caps.hdr));
    note("tv.supports_dolby_vision", format!("{:?}", caps.dolby_vision));

    let options = LoadOptions {
        app_id: APP_ID.into(),
        webos_major,
        start_pts_ns: 0,
        window_id: sdl.exported_window(),
        max_resolution: None,
    };
    let playing = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let player = {
        let (bridge, playing, stop) = (bridge.clone(), playing.clone(), stop.clone());
        std::thread::Builder::new()
            .name("player".into())
            .spawn(move || run_player(caps, options, bridge, playing, stop))
            .expect("player thread")
    };

    let mut inhibitor = None;
    let lifecycle = lifecycle::watch();
    let mut grace = lifecycle::Grace::default();
    let mut repeats = RepeatFilter::default();
    'run: loop {
        while let Some(event) = sdl.poll(if host.render_due() { 0 } else { 8 }) {
            match event {
                Event::Quit => {
                    if host.request_close() {
                        break 'run;
                    }
                }
                Event::Resized(width, height) => host.surface_changed(width, height),
                Event::Minimized => {
                    host.media_command("pause", 0.0);
                    grace.minimized(Instant::now());
                }
                Event::Restored => grace.restored(),
                Event::Key(raw) => {
                    if let Some(remote) =
                        map(&raw).filter(|remote| raw.pressed && repeats.accept(*remote, raw.repeat, Instant::now()))
                    {
                        if remote == Remote::Exit {
                            break 'run;
                        }
                        key(&host, remote);
                    }
                }
                Event::Text(text) => host.text_input(&text),
                Event::MouseMoved(x, y) => host.mouse_moved(x, y),
                Event::MouseButton { button, pressed, x, y } => {
                    let button = match button {
                        3 => MouseButton::Secondary,
                        2 => MouseButton::Middle,
                        _ => MouseButton::Primary,
                    };
                    host.mouse_button(button, pressed, x, y);
                }
                Event::Wheel(delta) => host.scroll(delta * 40.0),
            }
        }
        if lifecycle.as_ref().is_some_and(|lifecycle| lifecycle.relaunched()) {
            grace.restored();
            sdl.raise();
        }
        if grace.expired(Instant::now()) {
            host.media_command("stop", 0.0);
        }
        if !playing.load(Ordering::Relaxed) {
            inhibitor = None;
        } else if inhibitor.is_none() {
            inhibitor = screensaver::inhibit(APP_ID);
        }
        if host.render_due() {
            host.render();
        }
        if host.take_exit_request() {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    stop.store(true, Ordering::Relaxed);
    let _ = player.join();
    host.surface_destroyed();
}
