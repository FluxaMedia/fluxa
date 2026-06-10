// Windows native player surface — libmpv OpenGL render API via WGL.
//
// Architecture mirrors linux_player_surface.rs:
//   • A child HWND (CS_OWNDC, no message loop needed) is created inside the
//     Tauri window and placed at HWND_BOTTOM so it sits behind WebView2.
//   • A dedicated render thread owns the WGL context and calls
//     mpv_render_context_render every 16 ms, then SwapBuffers.
//   • A status-polling loop detects EOF / errors and emits Tauri events so
//     the frontend can act identically to the Linux code path.
//   • Player controls live in the WebView overlay (transparent background CSS).

use crate::DesktopState;
use std::sync::{mpsc, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use windows_sys::Win32::Foundation::{FALSE, HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::GetDC;
use windows_sys::Win32::Graphics::OpenGL::{
    ChoosePixelFormat, SetPixelFormat, SwapBuffers, PIXELFORMATDESCRIPTOR,
    PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA,
    wglCreateContext, wglDeleteContext, wglMakeCurrent,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, RegisterClassExW, SetWindowPos, ShowWindow,
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, HWND_BOTTOM, SW_HIDE, SW_SHOW, SWP_NOACTIVATE,
    WNDCLASSEXW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};


enum SurfaceCommand {
    Load { url: String, start_at: Option<u64>, total_duration: Option<u64> },
    Hide,
    ShowLoading { title: String, episode_title: Option<String> },
    SetTitle { title: String, episode_title: Option<String> },
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
    pub fn load(&self, url: String, start_at: Option<u64>, total_duration: Option<u64>) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::Load { url, start_at, total_duration })
            .map_err(|e| format!("surface unavailable: {e}"))
    }
    pub fn hide(&self) {
        let _ = self.sender.send(SurfaceCommand::Hide);
    }
    pub fn show_loading(&self, title: String, episode_title: Option<String>) {
        let _ = self
            .sender
            .send(SurfaceCommand::ShowLoading { title, episode_title });
    }
    pub fn set_title(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender
            .send(SurfaceCommand::SetTitle { title, episode_title });
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

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let (sender, receiver) = mpsc::channel::<SurfaceCommand>();
    let (setup_tx, setup_rx) = mpsc::channel::<Result<(), String>>();

    // Get the Tauri window's HWND via raw_window_handle.
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let parent_hwnd: HWND = match window
        .window_handle()
        .map_err(|e| e.to_string())?
        .as_ref()
    {
        RawWindowHandle::Win32(h) => h.hwnd.get() as HWND,
        _ => return Err("unexpected window handle type on Windows".to_string()),
    };

    let app = app_handle.clone();

    std::thread::spawn(move || {
        // Register the OpenGL-capable window class once per process.
        static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();
        CLASS_REGISTERED.get_or_init(|| {
            let class_name: Vec<u16> = "FluxaPlayerGL\0".encode_utf16().collect();
            let mut wc: WNDCLASSEXW = unsafe { std::mem::zeroed() };
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
            wc.lpfnWndProc = Some(DefWindowProcW);
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
                WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
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
            let _ = setup_tx.send(Err("CreateWindowExW failed for mpv surface".to_string()));
            return;
        }

        // Place the child window behind WebView2 in Z-order.
        unsafe {
            SetWindowPos(child_hwnd, HWND_BOTTOM, 0, 0, init_w, init_h, SWP_NOACTIVATE);
        }

        // ── WGL context setup ──────────────────────────────────────────────────
        let hdc = unsafe { GetDC(child_hwnd) };
        if hdc == 0 {
            let _ = setup_tx.send(Err("GetDC failed".to_string()));
            return;
        }

        let mut pfd: PIXELFORMATDESCRIPTOR = unsafe { std::mem::zeroed() };
        pfd.nSize = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cDepthBits = 24;
        pfd.cStencilBits = 8;

        let pf_idx = unsafe { ChoosePixelFormat(hdc, &pfd) };
        if pf_idx == 0 {
            let _ = setup_tx.send(Err("ChoosePixelFormat failed".to_string()));
            return;
        }
        if unsafe { SetPixelFormat(hdc, pf_idx, &pfd) } == FALSE {
            let _ = setup_tx.send(Err("SetPixelFormat failed".to_string()));
            return;
        }

        let hglrc = unsafe { wglCreateContext(hdc) };
        if hglrc == 0 {
            let _ = setup_tx.send(Err("wglCreateContext failed".to_string()));
            return;
        }
        if unsafe { wglMakeCurrent(hdc, hglrc) } == FALSE {
            unsafe { wglDeleteContext(hglrc) };
            let _ = setup_tx.send(Err("wglMakeCurrent failed".to_string()));
            return;
        }

        // ── mpv renderer init ──────────────────────────────────────────────────
        {
            let state = app.state::<DesktopState>();
            let mut renderer = state.player_renderer.lock().unwrap();
            if renderer.is_none() {
                match crate::mpv_render::MpvRenderer::new() {
                    Ok(r) => *renderer = Some(r),
                    Err(e) => {
                        unsafe {
                            wglMakeCurrent(0 as _, 0 as _);
                            wglDeleteContext(hglrc);
                        }
                        let _ = setup_tx.send(Err(format!("mpv init failed: {e}")));
                        return;
                    }
                }
            }
            if let Some(r) = renderer.as_mut() {
                if let Err(e) = r.prepare_opengl_context() {
                    unsafe {
                        wglMakeCurrent(0 as _, 0 as _);
                        wglDeleteContext(hglrc);
                    }
                    let _ = setup_tx.send(Err(format!("mpv GL context failed: {e}")));
                    return;
                }
            }
        }

        let _ = setup_tx.send(Ok(()));

        // ── Render + command loop ──────────────────────────────────────────────
        let mut visible = false;
        let mut last_size = (init_w, init_h);

        loop {
            // Drain command channel.
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SurfaceCommand::Load { url, start_at, .. } => {
                        unsafe { ShowWindow(child_hwnd, SW_SHOW) };
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
                        visible = true;
                        let _ = app.emit("native-player-show", ());
                        let state = app.state::<DesktopState>();
                        *state.eof_next_fired.lock().unwrap() = false;
                        let mut renderer = state.player_renderer.lock().unwrap();
                        if let Some(r) = renderer.as_mut() {
                            if let Err(e) = r.load(&url, start_at) {
                                drop(renderer);
                                let _ = app.emit("native-player-error", e);
                                visible = false;
                                unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                            }
                        }
                    }
                    SurfaceCommand::Hide => {
                        visible = false;
                        unsafe { ShowWindow(child_hwnd, SW_HIDE) };
                        let _ = app.emit("native-player-hide", ());
                        let state = app.state::<DesktopState>();
                        let guard = state.player_renderer.lock().unwrap();
                        if let Some(r) = guard.as_ref() {
                            let _ = r.command_string("stop");
                        }
                    }
                    SurfaceCommand::ShowLoading { title, episode_title } => {
                        // The WebView overlay renders the loading screen; just push the
                        // title so it can show instantly before the file URL is resolved.
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetTitle { title, episode_title } => {
                        // Emit to the WebView overlay so it can update its UI.
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetArtwork { title, .. } => {
                        let _ = app.emit("native-player-title", serde_json::json!({ "title": title }));
                    }
                }
            }

            if visible {
                // Keep child window sized to the Tauri window's client area.
                let mut r2: RECT = unsafe { std::mem::zeroed() };
                unsafe { GetClientRect(parent_hwnd, &mut r2) };
                let nw = (r2.right - r2.left).max(2);
                let nh = (r2.bottom - r2.top).max(2);
                if (nw, nh) != last_size {
                    last_size = (nw, nh);
                    unsafe {
                        SetWindowPos(child_hwnd, HWND_BOTTOM, 0, 0, nw, nh, SWP_NOACTIVATE);
                    }
                }

                // Render frame.
                {
                    let state = app.state::<DesktopState>();
                    let mut renderer = state.player_renderer.lock().unwrap();
                    if let Some(r) = renderer.as_mut() {
                        let _ = r.render_opengl_frame(nw, nh);
                    }
                }
                unsafe { SwapBuffers(hdc) };

                check_player_events(&app);
            }

            std::thread::sleep(Duration::from_millis(16));
        }
    });

    setup_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "Windows player surface setup timed out".to_string())
        .and_then(|r| r)
        .map(|()| NativePlayerSurface { sender })
}

fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let status = {
        let renderer = state.player_renderer.lock().unwrap();
        renderer.as_ref().map(|r| r.status())
    };
    let Some(status) = status else { return };
    if !status.eof_reached() {
        return;
    }
    let mut fired = state.eof_next_fired.lock().unwrap();
    if *fired {
        return;
    }
    *fired = true;
    drop(fired);

    let next_sub = state.next_ep_subtitle.lock().unwrap().clone();
    let auto_play = *state.auto_play_next_episode.lock().unwrap();
    if !next_sub.is_empty() && auto_play {
        let _ = app.emit("native-player-next-episode", ());
    } else {
        let _ = app.emit("native-player-close-requested", ());
    }
}

// Image scaling helpers — mirrors the Linux versions so artwork decoding works
// on Windows too.

pub fn scale_artwork_cover(bytes: Vec<u8>, target_w: u32, target_h: u32) -> Option<(Vec<u8>, i32, i32)> {
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
