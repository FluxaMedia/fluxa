// Linux native player surface — libmpv OpenGL render API via GLX/EGL.
//
// Architecture:
//   • A GtkGLArea is placed BEHIND the WebView inside a GtkOverlay.
//   • The WebView stays visible with a transparent background (Tauri transparent=true).
//   • The React overlay (ReactPlayerOverlay.tsx) renders on top and handles all
//     player controls — no GTK controls are built here.
//   • On Load/Hide we emit native-player-show / native-player-hide so the React
//     overlay activates / deactivates, mirroring the Windows and macOS surfaces.

use crate::DesktopState;
use glib::ControlFlow;
use gtk::prelude::*;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

// ─── Chapter / skip-segment utilities (polled by player_get_playback_info) ───

struct Chapter {
    title: String,
    start_ms: i64,
}


fn read_mpv_chapters(renderer: &crate::mpv_render::MpvRenderer) -> Vec<Chapter> {
    let count: usize = renderer
        .query_property("chapters")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    if count == 0 {
        return vec![];
    }
    (0..count)
        .filter_map(|i| {
            let title = renderer
                .query_property(&format!("chapter-list/{i}/title"))
                .unwrap_or_default();
            let start_secs: f64 = renderer
                .query_property(&format!("chapter-list/{i}/time"))
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0.0);
            Some(Chapter { title, start_ms: (start_secs * 1000.0).round() as i64 })
        })
        .collect()
}

fn chapters_to_json(chapters: &[Chapter]) -> String {
    let arr: Vec<serde_json::Value> = chapters
        .iter()
        .map(|c| serde_json::json!({ "title": c.title, "startTime": c.start_ms }))
        .collect();
    serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
}

fn classify_chapter_skip_type(title: &str) -> Option<&'static str> {
    let t = title.trim().to_lowercase();
    match t.as_str() {
        "op" | "opening" | "intro" | "introduction" | "op sequence" | "mixed-intro"
        | "opening sequence" | "opening theme" => return Some("intro"),
        "ed" | "ending" | "outro" | "credits" | "end credits" | "closing"
        | "ending theme" | "ending sequence" => return Some("outro"),
        "recap" | "previously" | "previously on" | "cold open" => return Some("recap"),
        _ => {}
    }
    if t.starts_with("op ") || t.starts_with("opening ") || t.contains("intro") || t.contains("opening") {
        return Some("intro");
    }
    if t.starts_with("ed ") || t.starts_with("ending ") || t.contains("ending")
        || t.contains("outro") || t.contains("credits")
    {
        return Some("outro");
    }
    if t.contains("recap") || t.contains("previously") {
        return Some("recap");
    }
    None
}

fn derive_skip_segments_from_chapters(chapters: &[Chapter]) -> Vec<serde_json::Value> {
    chapters
        .iter()
        .enumerate()
        .filter_map(|(i, ch)| {
            let seg_type = classify_chapter_skip_type(&ch.title)?;
            let end_ms = chapters.get(i + 1).map(|next| next.start_ms)?;
            if end_ms > ch.start_ms {
                Some(serde_json::json!({
                    "type": seg_type,
                    "startTime": ch.start_ms,
                    "endTime": end_ms,
                }))
            } else {
                None
            }
        })
        .collect()
}

// ─── Surface commands ─────────────────────────────────────────────────────────

enum SurfaceCommand {
    Load { url: String, start_at: Option<u64> },
    Hide,
    ShowLoading { title: String, episode_title: Option<String> },
    SetTitle { title: String, episode_title: Option<String> },
    SetArtwork { title: String, episode_title: Option<String> },
}

#[derive(Clone)]
pub struct NativePlayerSurface {
    sender: mpsc::Sender<SurfaceCommand>,
}

impl NativePlayerSurface {
    pub fn load(&self, url: String, start_at: Option<u64>, _total_duration: Option<u64>) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::Load { url, start_at })
            .map_err(|e| format!("native player surface is not available: {e}"))
    }

    pub fn hide(&self) {
        let _ = self.sender.send(SurfaceCommand::Hide);
    }

    pub fn show_loading(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::ShowLoading { title, episode_title });
    }

    pub fn set_title(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::SetTitle { title, episode_title });
    }

    pub fn set_artwork(
        &self,
        title: String,
        episode_title: Option<String>,
        _background: Option<(Vec<u8>, i32, i32)>,
        _logo: Option<(Vec<u8>, i32, i32)>,
    ) {
        let _ = self.sender.send(SurfaceCommand::SetArtwork { title, episode_title });
    }
}

// ─── Install ──────────────────────────────────────────────────────────────────

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let (sender, receiver) = mpsc::channel();
    let (setup_tx, setup_rx) = mpsc::channel::<Result<(), String>>();

    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main webview window was not found".to_string())?;

    window
        .with_webview(move |platform_webview| {
            let webview_widget = platform_webview.inner().upcast::<gtk::Widget>();
            let Some(parent_widget) = webview_widget.parent() else {
                let _ = setup_tx.send(Err("WebView parent widget not found — cannot attach player surface".to_string()));
                return;
            };
            let Ok(parent_box) = parent_widget.downcast::<gtk::Box>() else {
                let _ = setup_tx.send(Err(
                    "WebView parent is not a GTK Box; widget hierarchy may differ on this Tauri build".to_string(),
                ));
                return;
            };

            // GLArea for mpv OpenGL rendering — initially hidden, shown only when playing.
            let gl_area = gtk::GLArea::new();
            gl_area.set_hexpand(true);
            gl_area.set_vexpand(true);
            gl_area.set_has_alpha(false);
            gl_area.set_auto_render(false);
            gl_area.set_required_version(3, 2);
            // no_show_all prevents parent show_all() from revealing the video prematurely.
            gl_area.set_no_show_all(true);

            // Wrap in a GtkOverlay: GLArea (base/background) + WebView (overlay/foreground).
            // The WebView is transparent when the player is active (see App.tsx + transparent=true
            // in tauri.conf.json), so the video shows through from the GLArea behind it.
            let video_overlay = gtk::Overlay::new();
            video_overlay.set_hexpand(true);
            video_overlay.set_vexpand(true);

            webview_widget.set_hexpand(true);
            webview_widget.set_vexpand(true);

            // Reparent: move the WebView from parent_box into the overlay (on top of GLArea).
            parent_box.remove(&webview_widget);
            video_overlay.add(&gl_area);
            video_overlay.add_overlay(&webview_widget);
            webview_widget.set_halign(gtk::Align::Fill);
            webview_widget.set_valign(gtk::Align::Fill);
            parent_box.pack_start(&video_overlay, true, true, 0);
            parent_box.show_all();
            // gl_area stays hidden (no_show_all) until the first Load command.

            // Render callback: called by GTK when gl_area.queue_render() is invoked.
            let render_app = app_handle.clone();
            gl_area.connect_render(move |area, _ctx| {
                let state = render_app.state::<DesktopState>();
                if state.pending_hide.load(std::sync::atomic::Ordering::Acquire) {
                    // Returning Proceed (unhandled) with auto_render=false skips GTK's
                    // gdk_cairo_draw_from_gl readback path — prevents a multi-second freeze
                    // that occurs on some composited systems when the last video frame is read.
                    area.set_auto_render(false);
                    return glib::Propagation::Proceed;
                }
                area.make_current();
                if area.error().is_some() {
                    return glib::Propagation::Stop;
                }
                let w = area.allocated_width().max(2);
                let h = area.allocated_height().max(2);
                let Ok(mut renderer) = state.player_renderer.try_lock() else {
                    return glib::Propagation::Stop;
                };
                if let Some(r) = renderer.as_mut() {
                    if let Err(e) = r.render_opengl_frame(w, h) {
                        log::warn!("mpv OpenGL render failed: {e}");
                    }
                }
                glib::Propagation::Stop
            });

            let _ = setup_tx.send(Ok(()));

            let command_app = app_handle.clone();
            let command_gl_area = gl_area.clone();
            let mut visible = false;
            let mut pending_load: Option<(String, Option<u64>)> = None;
            let mut pending_load_retries: u32 = 0;
            let mut latch_grace_ticks: u32 = 0;
            let mut chapters_native_loaded = false;

            glib::timeout_add_local(Duration::from_millis(16), move || {
                // ── Drain surface commands ───────────────────────────────────────
                while let Ok(command) = receiver.try_recv() {
                    match command {
                        SurfaceCommand::Load { url, start_at } => {
                            command_gl_area.set_auto_render(true);
                            command_app.state::<DesktopState>().pending_hide
                                .store(false, std::sync::atomic::Ordering::Release);
                            *command_app.state::<DesktopState>().eof_next_fired.lock().unwrap() = false;
                            *command_app.state::<DesktopState>().chapters_json.lock().unwrap() = None;
                            command_gl_area.show();
                            visible = true;
                            latch_grace_ticks = 10;
                            chapters_native_loaded = false;
                            pending_load = Some((url, start_at));
                            pending_load_retries = 0;
                            let _ = command_app.emit("native-player-show", ());
                        }
                        SurfaceCommand::Hide => {
                            visible = false;
                            pending_load = None;
                            pending_load_retries = 0;
                            command_app.state::<DesktopState>().pending_hide
                                .store(true, std::sync::atomic::Ordering::Release);
                            command_gl_area.set_auto_render(false);
                            command_gl_area.hide();
                            let _ = command_app.emit("native-player-hide", ());
                            if let Ok(guard) = command_app.state::<DesktopState>().player_renderer.try_lock() {
                                if let Some(r) = guard.as_ref() {
                                    let _ = r.command_string("stop");
                                }
                            }
                        }
                        SurfaceCommand::ShowLoading { title, episode_title } => {
                            command_app.state::<DesktopState>().pending_hide
                                .store(false, std::sync::atomic::Ordering::Release);
                            command_gl_area.set_auto_render(true);
                            let _ = command_app.emit(
                                "native-player-title",
                                serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                            );
                        }
                        SurfaceCommand::SetTitle { title, episode_title } => {
                            let _ = command_app.emit(
                                "native-player-title",
                                serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                            );
                        }
                        SurfaceCommand::SetArtwork { title, episode_title } => {
                            // Artwork display is handled by PlayerLoadingOverlay in the WebView.
                            // We just forward the title so ReactPlayerOverlay can show it.
                            let _ = command_app.emit(
                                "native-player-title",
                                serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                            );
                        }
                    }
                }

                // ── Warm up OpenGL context before first play ─────────────────────
                // prepare_opengl_context() can block the GTK main thread for several
                // seconds on first run. Doing it here (idle, no user waiting) is better
                // than doing it inside prepare_and_load when the loading screen is shown.
                if pending_load.is_none() {
                    if let Ok(mut guard) = command_app.state::<DesktopState>().player_renderer.try_lock() {
                        if let Some(renderer) = guard.as_mut() {
                            if renderer.needs_opengl_context() {
                                command_gl_area.make_current();
                                if command_gl_area.error().is_none() {
                                    let _ = renderer.prepare_opengl_context();
                                }
                            }
                        }
                    }
                }

                // ── Retry deferred load ──────────────────────────────────────────
                if let Some((ref url, start_at)) = pending_load.clone() {
                    let state = command_app.state::<DesktopState>();
                    if state.player_renderer.try_lock().is_ok() {
                        match prepare_and_load(&command_app, &command_gl_area, url, start_at) {
                            Ok(()) => {
                                pending_load = None;
                                latch_grace_ticks = 10;
                                let hide_pending = command_app.state::<DesktopState>()
                                    .pending_hide.load(std::sync::atomic::Ordering::Acquire);
                                if hide_pending {
                                    visible = false;
                                    command_gl_area.hide();
                                }
                            }
                            Err(e) => {
                                pending_load = None;
                                visible = false;
                                command_gl_area.hide();
                                log::warn!("native OpenGL player load failed: {e}");
                                let _ = command_app.emit("native-player-error", e);
                            }
                        }
                    } else {
                        pending_load_retries += 1;
                        if pending_load_retries > 300 {
                            pending_load = None;
                            visible = false;
                            command_gl_area.hide();
                            let _ = command_app.emit("native-player-error", "player renderer busy".to_string());
                        }
                    }
                }

                // ── Per-frame tasks while playing ────────────────────────────────
                if visible {
                    let in_grace = if latch_grace_ticks > 0 {
                        latch_grace_ticks -= 1;
                        true
                    } else {
                        false
                    };

                    // Read native chapters from mpv once playback has started.
                    // Stored in DesktopState so player_get_playback_info() can return them
                    // to the React overlay for chapter-segmented seekbar rendering.
                    if !in_grace && !chapters_native_loaded {
                        let state = command_app.state::<DesktopState>();
                        let pos = state.player_renderer.try_lock().ok()
                            .and_then(|g| g.as_ref()
                                .and_then(|r| r.query_property("time-pos")
                                    .and_then(|v| v.trim().parse::<f64>().ok())))
                            .unwrap_or(0.0);
                        if pos > 0.05 {
                            chapters_native_loaded = true;
                            let chapters_already_set = state.chapters_json.lock().unwrap().is_some();
                            if !chapters_already_set {
                                if let Ok(guard) = state.player_renderer.try_lock() {
                                    if let Some(renderer) = guard.as_ref() {
                                        let native = read_mpv_chapters(renderer);
                                        if !native.is_empty() {
                                            *state.chapters_json.lock().unwrap() =
                                                Some(chapters_to_json(&native));
                                            let skip_already_set =
                                                state.skip_segments_json.lock().unwrap().is_some();
                                            if !skip_already_set {
                                                let derived = derive_skip_segments_from_chapters(&native);
                                                if !derived.is_empty() {
                                                    if let Ok(json) = serde_json::to_string(&derived) {
                                                        *state.skip_segments_json.lock().unwrap() = Some(json);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    check_player_events(&command_app);
                    command_gl_area.queue_render();
                }

                ControlFlow::Continue
            });
        })
        .map_err(|e| e.to_string())?;

    setup_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "native player surface setup timed out".to_string())
        .and_then(|r| r)
        .map(|()| NativePlayerSurface { sender })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn prepare_and_load(
    app_handle: &AppHandle,
    gl_area: &gtk::GLArea,
    url: &str,
    start_at: Option<u64>,
) -> Result<(), String> {
    if !gl_area.is_realized() {
        gl_area.realize();
    }
    gl_area.make_current();
    if let Some(error) = gl_area.error() {
        return Err(format!("OpenGL player surface context error: {error}"));
    }

    let state = app_handle.state::<DesktopState>();
    let mut renderer = state.player_renderer.try_lock()
        .map_err(|_| "player renderer busy — load deferred".to_string())?;
    if renderer.is_none() {
        *renderer = Some(crate::mpv_render::MpvRenderer::new()?);
    }
    let renderer = renderer
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?;
    renderer.prepare_opengl_context()?;
    renderer.load(url, start_at)
}

fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let eof = {
        let renderer = state.player_renderer.lock().unwrap();
        renderer.as_ref().map(|r| r.status().eof_reached()).unwrap_or(false)
    };
    if !eof {
        let mut fired = state.eof_next_fired.lock().unwrap();
        if *fired {
            *fired = false;
        }
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
