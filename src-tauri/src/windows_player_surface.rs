// Windows native player surface — libmpv OpenGL render API via ANGLE/EGL.
//
// Architecture mirrors linux_player_surface.rs:
//   • A child HWND (CS_OWNDC, no message loop needed) is created inside the
//     Tauri window and placed at HWND_BOTTOM so it sits behind WebView2.
//   • A dedicated render thread owns the ANGLE/EGL context (see windows_egl.rs)
//     and calls mpv_render_context_render every 16 ms, then eglSwapBuffers.
//   • A status-polling loop detects EOF / errors and emits Tauri events so
//     the frontend can act identically to the Linux code path.
//   • Player controls live in the WebView overlay (transparent background CSS).

use crate::mpv_render::VulkanTargetImage;
use crate::playback_engine::{PlaybackEngine, PlayerEngine};
use crate::windows_d3d11::D3d11Context;
use crate::windows_egl::{self, EglContext};
use crate::windows_vulkan::VulkanContext;
use crate::DesktopState;
use fluxa_core::FluxaCore;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use windows::core::Interface;
use windows_sys::Win32::Foundation::{FALSE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{
    DwmEnableBlurBehindWindow, DWM_BB_BLURREGION, DWM_BB_ENABLE, DWM_BLURBEHIND,
};
use windows_sys::Win32::Graphics::Gdi::HDC;
use windows_sys::Win32::Graphics::Gdi::{
    CreateRectRgn, DeleteObject, GetDC, GetDeviceCaps, VREFRESH,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::ColorSystem::GetICMProfileW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, LoadCursorW,
    PeekMessageW, RegisterClassExW, SetCursor, SetWindowPos, ShowWindow, TranslateMessage,
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, HWND_BOTTOM, IDC_ARROW, MSG, PM_REMOVE, SWP_NOACTIVATE,
    SW_HIDE, SW_SHOW, WM_SETCURSOR, WNDCLASSEXW, WS_CHILD, WS_CLIPCHILDREN,
};

static CURSOR_HIDDEN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum RenderBackend {
    OpenGl,
    Vulkan,
    D3d11,
}

fn hdr_output_enabled(app: &AppHandle) -> bool {
    let state = app.state::<DesktopState>();
    crate::storage::read_pref_bool(state, "hdrEnabled").unwrap_or(true)
}

fn read_render_backend(app: &AppHandle) -> RenderBackend {
    let state = app.state::<DesktopState>();
    match crate::storage::read_pref_field(state, "renderBackend").as_deref() {
        Some("vulkan") => RenderBackend::Vulkan,
        Some("d3d11") => RenderBackend::D3d11,
        _ => RenderBackend::OpenGl,
    }
}

enum RenderContext {
    Egl(EglContext),
    D3d11(D3d11Context),
    Vulkan(VulkanContext),
}

impl RenderContext {
    fn poll_resize(&self) {
        if let RenderContext::Egl(egl) = self {
            egl.poll_resize();
        }
    }

    fn warm_up_after_show(&self) {
        if let RenderContext::Egl(egl) = self {
            // ANGLE can create its window surface while the child HWND is hidden,
            // but DXVA2/EGL interop on some drivers is not usable until that HWND
            // has been presented once. Prime it before mpv starts decoding.
            egl.poll_resize();
            // Never let the one-time prime wait for a compositor/vblank. Some
            // virtual display drivers can leave that wait pending indefinitely.
            let _ = egl.set_swap_interval(0);
            egl.swap_buffers();
            let _ = egl.set_swap_interval(1);
        }
    }

    fn is_hdr(&self) -> bool {
        match self {
            RenderContext::Egl(_) => false,
            RenderContext::D3d11(ctx) => ctx.is_hdr(),
            RenderContext::Vulkan(ctx) => ctx.is_hdr(),
        }
    }

    fn render_and_present(
        &mut self,
        renderer: &mut crate::mpv_render::MpvRenderer,
        width: i32,
        height: i32,
        vsync_enabled: bool,
    ) -> Result<(), String> {
        match self {
            RenderContext::Egl(egl) => {
                let render_result = renderer.render_opengl_frame(width, height);
                egl.swap_buffers();
                renderer.report_swap();
                render_result
            }
            RenderContext::D3d11(ctx) => {
                ctx.resize(width, height)?;
                if renderer.needs_d3d11_context() {
                    renderer.create_d3d11_context(ctx.device_ptr())?;
                }
                let back_buffer = ctx.back_buffer()?;
                renderer.render_d3d11_frame(back_buffer.as_raw())?;
                ctx.present(vsync_enabled)?;
                renderer.report_swap();
                Ok(())
            }
            RenderContext::Vulkan(ctx) => {
                if renderer.needs_vulkan_context() {
                    let (instance, phys_device, device, queue_index, queue_count) =
                        ctx.device_handles();
                    renderer.create_vulkan_context(
                        instance,
                        phys_device,
                        device,
                        queue_index,
                        queue_count,
                        std::ptr::null_mut(),
                        &[],
                    )?;
                }
                ctx.resize(width, height)?;
                let image_usage = ctx.image_usage();
                let result = ctx.render_and_present(
                    |image, format, w, h, wait_semaphore, signal_semaphore| {
                        let mut target = VulkanTargetImage {
                            image,
                            format,
                            w: w as i32,
                            h: h as i32,
                            usage: image_usage,
                            layout: 0,
                            wait_semaphore,
                            signal_semaphore,
                        };
                        renderer
                            .render_vulkan_frame(&mut target)
                            .map(|_| target.layout)
                    },
                );
                if result.is_ok() {
                    renderer.report_swap();
                }
                result
            }
        }
    }
}

unsafe extern "system" fn player_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SETCURSOR && CURSOR_HIDDEN.load(Ordering::Acquire) {
        unsafe { SetCursor(0) };
        return 1;
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn set_window_blur_behind(hwnd: HWND, enable: bool) {
    let region = if enable {
        unsafe { CreateRectRgn(0, 0, -1, -1) }
    } else {
        0
    };
    let bb = DWM_BLURBEHIND {
        dwFlags: if enable {
            DWM_BB_ENABLE | DWM_BB_BLURREGION
        } else {
            DWM_BB_ENABLE
        },
        fEnable: if enable { 1 } else { 0 },
        hRgnBlur: region,
        fTransitionOnMaximized: 0,
    };
    unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };
    if region != 0 {
        unsafe { DeleteObject(region) };
    }
}

fn schedule_parent_blur(app: &AppHandle, parent_hwnd: HWND, enable: bool) {
    let app = app.clone();
    let _ = app.run_on_main_thread(move || {
        unsafe { set_window_blur_behind(parent_hwnd, enable) };
    });
}

fn main_window_client_size(app: &AppHandle) -> Option<(i32, i32)> {
    let state = app.state::<DesktopState>();
    let packed = state
        .main_window_size
        .load(std::sync::atomic::Ordering::Acquire);
    if packed == 0 {
        return None;
    }
    let width = (packed >> 32) as u32 as i32;
    let height = (packed & 0xFFFF_FFFF) as u32 as i32;
    Some((width.max(2), height.max(2)))
}

fn query_icm_profile(hdc: HDC) -> Option<Vec<u8>> {
    let mut size: u32 = 0;
    unsafe { GetICMProfileW(hdc, &mut size, std::ptr::null_mut()) };
    if size == 0 {
        return None;
    }
    let mut buf: Vec<u16> = vec![0; size as usize];
    let ok = unsafe { GetICMProfileW(hdc, &mut size, buf.as_mut_ptr()) };
    if ok == FALSE {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let path = String::from_utf16_lossy(&buf[..len]);
    std::fs::read(path).ok()
}

fn ensure_renderer_for_surface(
    app: &AppHandle,
    hdc: HDC,
    backend: RenderBackend,
) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    let mut renderer = state.player_renderer.lock().unwrap();
    if renderer.is_none() {
        log::warn!("player surface: renderer missing, recreating before load");
        let mut fresh =
            crate::mpv_render::MpvRenderer::new().map_err(|e| format!("mpv init failed: {e}"))?;
        match backend {
            RenderBackend::OpenGl => {
                fresh
                    .prepare_opengl_context()
                    .map_err(|e| format!("mpv GL context failed: {e}"))?;
            }
            RenderBackend::Vulkan => {
                fresh
                    .set_option("gpu-api", "vulkan")
                    .map_err(|e| format!("mpv gpu-api=vulkan failed: {e}"))?;
            }
            RenderBackend::D3d11 => {
                fresh
                    .set_option("gpu-api", "d3d11")
                    .map_err(|e| format!("mpv gpu-api=d3d11 failed: {e}"))?;
            }
        }
        if hdc != 0 {
            if let Some(icc) = query_icm_profile(hdc) {
                if let Err(e) = fresh.set_icc_profile(&icc) {
                    log::warn!("failed to set ICC profile: {e}");
                }
            }
        }
        *renderer = Some(fresh);
    }
    if hdc != 0 {
        if let Some(r) = renderer.as_ref() {
            let hz = unsafe { GetDeviceCaps(hdc, VREFRESH as i32) };
            if hz > 1 {
                if let Err(e) = r.set_option("display-fps-override", &hz.to_string()) {
                    log::warn!("failed to set display-fps-override: {e}");
                }
            }
        }
    }
    Ok(())
}

enum SurfaceCommand {
    Load {
        url: String,
        start_at: Option<u64>,
        total_duration: Option<u64>,
    },
    Hide,
    CommandArgs {
        commands: Vec<Vec<String>>,
        sender: mpsc::Sender<Result<(), String>>,
    },
    PlayerCommand(String),
    Status(mpsc::Sender<crate::mpv_render::PlayerStatus>),
    TrackOptions {
        track_type: String,
        sender: mpsc::Sender<Vec<crate::mpv_render::PlayerTrackOption>>,
    },
    AddSubtitle {
        url: String,
        title: Option<String>,
        language: Option<String>,
        sender: mpsc::Sender<Result<(), String>>,
    },
    SetCursorVisible(bool),
    ShowLoading {
        title: String,
        episode_title: Option<String>,
    },
    SetTitle {
        title: String,
        episode_title: Option<String>,
    },
    SetArtwork {
        title: String,
        episode_title: Option<String>,
        background: Option<(Vec<u8>, i32, i32)>,
        logo: Option<(Vec<u8>, i32, i32)>,
    },
}

#[derive(Clone)]
pub struct NativePlayerSurface {
    sender: mpsc::Sender<SurfaceCommand>,
}

impl NativePlayerSurface {
    pub fn load(
        &self,
        url: String,
        start_at: Option<u64>,
        total_duration: Option<u64>,
    ) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::Load {
                url,
                start_at,
                total_duration,
            })
            .map_err(|e| format!("surface unavailable: {e}"))
    }
    pub fn hide(&self) {
        let _ = self.sender.send(SurfaceCommand::Hide);
    }
    pub fn command(&self, command: String) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::PlayerCommand(command))
            .map_err(|e| format!("surface unavailable: {e}"))
    }
    pub fn command_args(&self, commands: Vec<Vec<String>>) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(SurfaceCommand::CommandArgs { commands, sender })
            .map_err(|e| format!("surface unavailable: {e}"))?;
        receiver.recv_timeout(Duration::from_secs(5))
            .map_err(|e| format!("player command unavailable: {e}"))?
    }
    pub fn status(&self) -> Result<crate::mpv_render::PlayerStatus, String> {
        let (sender, receiver) = mpsc::channel();
        self.sender
            .send(SurfaceCommand::Status(sender))
            .map_err(|e| format!("surface unavailable: {e}"))?;
        receiver
            .recv_timeout(Duration::from_millis(250))
            .map_err(|e| format!("player status unavailable: {e}"))
    }
    pub fn track_options(
        &self,
        track_type: String,
    ) -> Result<Vec<crate::mpv_render::PlayerTrackOption>, String> {
        let (sender, receiver) = mpsc::channel();
        self.sender
            .send(SurfaceCommand::TrackOptions { track_type, sender })
            .map_err(|e| format!("surface unavailable: {e}"))?;
        receiver
            .recv_timeout(Duration::from_secs(2))
            .map_err(|e| format!("player track options unavailable: {e}"))
    }
    pub fn add_subtitle(
        &self,
        url: String,
        title: Option<String>,
        language: Option<String>,
    ) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        self.sender
            .send(SurfaceCommand::AddSubtitle {
                url,
                title,
                language,
                sender,
            })
            .map_err(|e| format!("surface unavailable: {e}"))?;
        receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| format!("player add subtitle unavailable: {e}"))?
    }
    pub fn set_cursor_visible(&self, visible: bool) {
        let _ = self.sender.send(SurfaceCommand::SetCursorVisible(visible));
    }
    pub fn show_loading(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::ShowLoading {
            title,
            episode_title,
        });
    }
    pub fn set_title(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::SetTitle {
            title,
            episode_title,
        });
    }
    pub fn set_artwork(
        &self,
        title: String,
        episode_title: Option<String>,
        background: Option<(Vec<u8>, i32, i32)>,
        logo: Option<(Vec<u8>, i32, i32)>,
    ) {
        let _ = self.sender.send(SurfaceCommand::SetArtwork {
            title,
            episode_title,
            background,
            logo,
        });
    }
}

enum InstallSlot {
    NotStarted,
    InProgress(mpsc::Receiver<Result<NativePlayerSurface, String>>),
    Done(Result<NativePlayerSurface, String>),
}

static INSTALL: Mutex<InstallSlot> = Mutex::new(InstallSlot::NotStarted);

fn reset_install_slot() {
    *INSTALL.lock().unwrap() = InstallSlot::NotStarted;
}

pub fn hdr_display_supported(app: &AppHandle) -> bool {
    let Some(window) = app.get_webview_window("main") else {
        return false;
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(handle) = window.window_handle() else {
        return false;
    };
    let RawWindowHandle::Win32(handle) = handle.as_ref() else {
        return false;
    };
    D3d11Context::new(handle.hwnd.get() as isize, 2, 2, true)
        .map(|ctx| ctx.is_hdr())
        .unwrap_or(false)
}
pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let mut slot = INSTALL.lock().unwrap();
    let rx = match std::mem::replace(&mut *slot, InstallSlot::NotStarted) {
        InstallSlot::Done(result) => {
            *slot = InstallSlot::Done(result.clone());
            return result;
        }
        InstallSlot::InProgress(rx) => rx,
        InstallSlot::NotStarted => {
            let (result_tx, result_rx) = mpsc::channel();
            spawn_install_thread(app_handle, result_tx);
            result_rx
        }
    };
    match rx.recv_timeout(Duration::from_secs(20)) {
        Ok(result) => {
            *slot = match &result {
                Ok(_) => InstallSlot::Done(result.clone()),
                Err(_) => InstallSlot::NotStarted,
            };
            result
        }
        Err(RecvTimeoutError::Timeout) => {
            *slot = InstallSlot::InProgress(rx);
            Err("Windows player surface setup timed out".to_string())
        }
        Err(RecvTimeoutError::Disconnected) => {
            *slot = InstallSlot::NotStarted;
            Err("Windows player surface install thread exited unexpectedly".to_string())
        }
    }
}

fn spawn_install_thread(
    app_handle: AppHandle,
    setup_tx: mpsc::Sender<Result<NativePlayerSurface, String>>,
) {
    let (sender, receiver) = mpsc::channel::<SurfaceCommand>();

    let window = match app_handle.get_webview_window("main") {
        Some(w) => w,
        None => {
            let _ = setup_tx.send(Err("main window not found".to_string()));
            return;
        }
    };

    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let parent_hwnd: HWND = match window.window_handle() {
        Ok(handle) => match handle.as_ref() {
            RawWindowHandle::Win32(h) => h.hwnd.get() as HWND,
            _ => {
                let _ = setup_tx.send(Err("unexpected window handle type on Windows".to_string()));
                return;
            }
        },
        Err(e) => {
            let _ = setup_tx.send(Err(e.to_string()));
            return;
        }
    };

    let app = app_handle;

    std::thread::spawn(move || {
        log::info!("player surface: install thread starting");
        // Register the OpenGL-capable window class once per process.
        static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();
        CLASS_REGISTERED.get_or_init(|| {
            let class_name: Vec<u16> = "FluxaPlayerGL\0".encode_utf16().collect();
            let mut wc: WNDCLASSEXW = unsafe { std::mem::zeroed() };
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
            wc.lpfnWndProc = Some(player_wnd_proc);
            wc.hInstance = unsafe { GetModuleHandleW(std::ptr::null()) };
            wc.lpszClassName = class_name.as_ptr();
            unsafe { RegisterClassExW(&wc) };
        });

        // Initial size from parent.
        let mut rect: RECT = unsafe { std::mem::zeroed() };
        unsafe { GetClientRect(parent_hwnd, &mut rect) };
        let init_w = (rect.right - rect.left).max(2);
        let init_h = (rect.bottom - rect.top).max(2);

        // Create the child HWND that mpv will render into.
        let class_name: Vec<u16> = "FluxaPlayerGL\0".encode_utf16().collect();
        let child_hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                std::ptr::null(),
                // No WS_VISIBLE — shown only when player is active.
                WS_CHILD | WS_CLIPCHILDREN,
                0,
                0,
                init_w,
                init_h,
                parent_hwnd,
                0,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        if child_hwnd == 0 {
            log::error!("player surface: CreateWindowExW failed for mpv surface");
            let _ = setup_tx.send(Err("CreateWindowExW failed for mpv surface".to_string()));
            return;
        }

        // Place the child window behind WebView2 in Z-order.
        unsafe {
            SetWindowPos(
                child_hwnd,
                HWND_BOTTOM,
                0,
                0,
                init_w,
                init_h,
                SWP_NOACTIVATE,
            );
        }

        let backend = read_render_backend(&app);
        let hdr_enabled = hdr_output_enabled(&app);
        log::info!(
            "player surface: experimental render backend = {}",
            match backend {
                RenderBackend::OpenGl => "opengl",
                RenderBackend::Vulkan => "vulkan",
                RenderBackend::D3d11 => "d3d11",
            }
        );

        let mut render_ctx = match backend {
            RenderBackend::OpenGl => match windows_egl::create_window_context(child_hwnd) {
                Ok(ctx) => RenderContext::Egl(ctx),
                Err(e) => {
                    log::error!("player surface: ANGLE/EGL context creation failed: {e}");
                    let _ = setup_tx.send(Err(format!("ANGLE/EGL context creation failed: {e}")));
                    return;
                }
            },
            RenderBackend::D3d11 => {
                match D3d11Context::new(child_hwnd as isize, init_w, init_h, hdr_enabled) {
                    Ok(ctx) => RenderContext::D3d11(ctx),
                    Err(e) => {
                        log::error!("player surface: D3D11 context creation failed: {e}");
                        let _ = setup_tx.send(Err(format!("D3D11 context creation failed: {e}")));
                        return;
                    }
                }
            }
            RenderBackend::Vulkan => {
                match VulkanContext::new(child_hwnd as isize, init_w, init_h) {
                    Ok(ctx) => RenderContext::Vulkan(ctx),
                    Err(e) => {
                        log::error!("player surface: Vulkan context creation failed: {e}");
                        let _ = setup_tx.send(Err(format!("Vulkan context creation failed: {e}")));
                        return;
                    }
                }
            }
        };
        if render_ctx.is_hdr() {
            log::info!("player surface: HDR-capable swap chain in use");
        }

        let vsync_enabled = match &render_ctx {
            RenderContext::Egl(egl) => {
                let enabled = egl.set_swap_interval(1);
                if !enabled {
                    log::warn!(
                        "eglSwapInterval unavailable; falling back to timer-paced rendering"
                    );
                }
                enabled
            }
            RenderContext::D3d11(_) | RenderContext::Vulkan(_) => true,
        };

        let hdc = unsafe { GetDC(child_hwnd) };

        if let Err(e) = ensure_renderer_for_surface(&app, hdc, backend) {
            log::error!("player surface: renderer setup failed: {e}");
            let _ = setup_tx.send(Err(e));
            return;
        }
        log::info!("player surface: MpvRenderer ready");

        log::info!("player surface: setup complete, entering render loop");
        let _ = setup_tx.send(Ok(NativePlayerSurface {
            sender: sender.clone(),
        }));

        // Render + command loop
        let mut visible = false;
        let mut last_size = (init_w, init_h);
        let mut last_render_error: Option<String> = None;
        let mut consecutive_render_errors = 0u32;
        let mut surface_warmed_up = false;

        'render: loop {
            unsafe {
                let mut msg: MSG = std::mem::zeroed();
                while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            // Drain command channel.
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SurfaceCommand::Load { url, start_at, .. } => {
                        log::info!("player surface: loading url={url} start_at={start_at:?}");
                        schedule_parent_blur(&app, parent_hwnd, true);
                        unsafe {
                            ShowWindow(child_hwnd, SW_SHOW);
                        }
                        unsafe {
                            SetWindowPos(
                                child_hwnd,
                                HWND_BOTTOM,
                                0,
                                0,
                                last_size.0,
                                last_size.1,
                                SWP_NOACTIVATE,
                            )
                        };
                        if !surface_warmed_up && backend == RenderBackend::OpenGl {
                            render_ctx.warm_up_after_show();
                            surface_warmed_up = true;
                            log::info!(
                                "player surface: OpenGL window surface warmed before first load"
                            );
                        }
                        visible = true;
                        let _ = app.emit("native-player-show", ());
                        let state = app.state::<DesktopState>();

                        state.pending_hide.store(false, Ordering::Release);
                        state.player_overlay.lock().unwrap().eof_next_fired = false;
                        if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
                            let result = (|| {
                                let mut players = state.player_renderer_vlc.lock().unwrap();
                                if players.is_none() {
                                    *players = Some(crate::libvlc_render::LibvlcPlayer::new()?);
                                }
                                let player = players
                                    .as_mut()
                                    .ok_or_else(|| "libVLC player is unavailable".to_string())?;
                                player.attach_hwnd(child_hwnd as *mut std::ffi::c_void)?;
                                player.load(&url, start_at)
                            })();
                            if let Err(error) = result {
                                log::error!("player surface: libVLC load failed: {error}");
                                let _ = app.emit("native-player-error", error);
                                visible = false;
                                unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                            }
                            continue;
                        }
                        if let Err(e) = ensure_renderer_for_surface(&app, hdc, backend) {
                            log::error!(
                                "player surface: renderer recreate failed before load: {e}"
                            );
                            let _ = app.emit("native-player-error", e);
                            visible = false;
                            unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                            continue;
                        }
                        let mut renderer = state.player_renderer.lock().unwrap();
                        if let Some(r) = renderer.as_mut() {
                            if render_ctx.is_hdr() {
                                let _ = r.set_option("target-trc", "linear");
                                let _ = r.set_option("target-prim", "bt.709");
                            }
                            if let Err(e) = r.load(&url, start_at) {
                                log::error!("player surface: load() failed: {e}");
                                drop(renderer);
                                let _ = app.emit("native-player-error", e);
                                visible = false;
                                unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                            } else {
                                log::info!("player surface: load() command accepted by mpv");
                            }
                        } else {
                            log::error!(
                                "player surface: Load command received but renderer is None"
                            );
                            let _ = app.emit(
                                "native-player-error",
                                "player renderer is unavailable".to_string(),
                            );
                            visible = false;
                            unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                        }
                    }
                    SurfaceCommand::CommandArgs { commands, sender } => {
                        let state = app.state::<DesktopState>();
                        let renderer = state.player_renderer.lock().unwrap();
                        let result = renderer.as_ref()
                            .ok_or_else(|| "player renderer is not initialized".to_string())
                            .and_then(|r| {
                                for command in &commands {
                                    let args = command.iter().map(String::as_str).collect::<Vec<_>>();
                                    r.command_args(&args)?;
                                }
                                Ok(())
                            });
                        if let Err(error) = &result {
                            log::error!("player surface: command batch failed: {error}");
                        }
                        let _ = sender.send(result);
                    }
                    SurfaceCommand::PlayerCommand(command) => {
                        let state = app.state::<DesktopState>();

                        let mut renderer = state.player_renderer.lock().unwrap();
                        if let Some(r) = renderer.as_mut() {
                            if let Err(error) = r.user_command(&command) {
                                log::warn!("player surface: command failed ({command}): {error}");
                            }
                        }
                    }
                    SurfaceCommand::Status(sender) => {
                        let state = app.state::<DesktopState>();

                        let renderer = state.player_renderer.lock().unwrap();
                        if let Some(r) = renderer.as_ref() {
                            let mut status = r.status();
                            status.hdr_active = render_ctx.is_hdr();
                            let _ = sender.send(status);
                        }
                    }
                    SurfaceCommand::TrackOptions { track_type, sender } => {
                        let state = app.state::<DesktopState>();
                        let renderer = state.player_renderer.lock().unwrap();
                        let tracks = renderer
                            .as_ref()
                            .map(|r| r.track_options(&track_type))
                            .unwrap_or_default();
                        let _ = sender.send(tracks);
                    }
                    SurfaceCommand::AddSubtitle {
                        url,
                        title,
                        language,
                        sender,
                    } => {
                        let state = app.state::<DesktopState>();
                        let renderer = state.player_renderer.lock().unwrap();
                        let result = renderer
                            .as_ref()
                            .ok_or_else(|| "player renderer is not initialized".to_string())
                            .and_then(|r| {
                                r.add_subtitle(&url, title.as_deref(), language.as_deref())
                            });
                        if let Err(error) = &result {
                            log::error!("player surface: add subtitle failed: {error}");
                        } else {
                            log::info!("player surface: external subtitle added: {url}");
                        }
                        let _ = sender.send(result);
                    }
                    SurfaceCommand::Hide => {
                        visible = false;
                        unsafe {
                            ShowWindow(child_hwnd, SW_HIDE);
                        }
                        schedule_parent_blur(&app, parent_hwnd, true);
                        let _ = app.emit("native-player-hide", ());
                        let state = app.state::<DesktopState>();

                        let guard = state.player_renderer.lock().unwrap();
                        if let Some(r) = guard.as_ref() {
                            let _ = r.command_args(&["stop"]);
                        }
                    }
                    SurfaceCommand::ShowLoading {
                        title,
                        episode_title,
                    } => {
                        // The WebView overlay renders the loading screen; just push the
                        // title so it can show instantly before the file URL is resolved.
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetTitle {
                        title,
                        episode_title,
                    } => {
                        // Emit to the WebView overlay so it can update its UI.
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetArtwork { title, .. } => {
                        let _ =
                            app.emit("native-player-title", serde_json::json!({ "title": title }));
                    }
                    SurfaceCommand::SetCursorVisible(cursor_visible) => {
                        CURSOR_HIDDEN.store(!cursor_visible, Ordering::Release);
                        unsafe {
                            SetCursor(if cursor_visible {
                                LoadCursorW(0, IDC_ARROW)
                            } else {
                                0
                            });
                        }
                    }
                }
            }

            if visible {
                let state = app.state::<DesktopState>();
                if state.pending_hide.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }

                let (nw, nh) = main_window_client_size(&app).unwrap_or(last_size);
                if (nw, nh) != last_size {
                    last_size = (nw, nh);
                    log::info!("player surface: resizing render surface to {nw}x{nh}");
                    unsafe {
                        SetWindowPos(child_hwnd, HWND_BOTTOM, 0, 0, nw, nh, SWP_NOACTIVATE);
                    }
                }

                render_ctx.poll_resize();
                if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }
                let swap_start = std::time::Instant::now();
                {
                    let Ok(mut renderer) = state.player_renderer.try_lock() else {
                        std::thread::sleep(Duration::from_millis(16));
                        continue;
                    };
                    if let Some(r) = renderer.as_mut() {
                        if let Err(e) = render_ctx.render_and_present(r, nw, nh, vsync_enabled) {
                            consecutive_render_errors = consecutive_render_errors.saturating_add(1);
                            if last_render_error.as_deref() != Some(e.as_str()) {
                                log::error!("player surface: render failed: {e}");
                                last_render_error = Some(e.clone());
                            }
                            if consecutive_render_errors >= 30 {
                                log::error!("player surface: too many render failures; switching to software video rendering");
                                unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                                let state = app.state::<DesktopState>();

                                *state.native_player_surface.lock().unwrap() = None;
                                reset_install_slot();
                                r.reset_render_context();
                                let _ = app.emit(
                                    "native-player-software-rendering",
                                    format!("Windows native player render failed repeatedly: {e}"),
                                );
                                drop(renderer);
                                break 'render;
                            }
                        } else {
                            last_render_error = None;
                            consecutive_render_errors = 0;
                        }
                    }
                }
                if !vsync_enabled || swap_start.elapsed() < Duration::from_millis(4) {
                    std::thread::sleep(Duration::from_millis(16));
                }

                check_player_events(&app);
            } else {
                std::thread::sleep(Duration::from_millis(16));
            }
        }

        drop(render_ctx);
        unsafe { DestroyWindow(child_hwnd) };
        log::info!("player surface: render thread exiting, native render context released");
    });
}

fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let events = {
        let Ok(mut renderer) = state.player_renderer.try_lock() else {
            return;
        };
        match renderer.as_mut() {
            Some(r) => r.poll_events(),
            None => return,
        }
    };
    for event in events {
        let crate::mpv_render::PlayerEvent::EndFile { eof, error } = event;
        log::info!("player surface: mpv END_FILE event eof={eof} error={error:?}");
        if let Some(message) = error {
            log::error!("player surface: stream failed to play: {message}");
            let _ = app.emit("native-player-error", message);
            continue;
        }
        if !eof {
            continue;
        }
        let mut overlay = state.player_overlay.lock().unwrap();
        if !overlay.take_eof_next() {
            continue;
        }
        let next_sub = overlay.next_ep_subtitle.clone();
        let auto_play = overlay.auto_play_next_episode;
        drop(overlay);
        if FluxaCore::should_play_next_episode(!next_sub.is_empty(), auto_play) {
            log::info!("player surface: eof reached, auto-playing next episode");
            let _ = app.emit("native-player-next-episode", ());
        } else {
            log::info!("player surface: eof reached, closing player");
            let _ = app.emit("native-player-close-requested", ());
        }
    }
}

// Image scaling helpers — mirrors the Linux versions so artwork decoding works
// on Windows too.

pub fn scale_artwork_cover(
    bytes: Vec<u8>,
    target_w: u32,
    target_h: u32,
) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let filled = img.resize_to_fill(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgba = filled.to_rgba8();
    Some((rgba.into_raw(), target_w as i32, target_h as i32))
}

pub fn scale_artwork_fit(bytes: Vec<u8>, max_w: u32, max_h: u32) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let resized = img.resize(max_w, max_h, image::imageops::FilterType::Triangle);
    let (rw, rh) = (resized.width(), resized.height());
    let rgba = resized.to_rgba8();
    Some((rgba.into_raw(), rw as i32, rh as i32))
}
