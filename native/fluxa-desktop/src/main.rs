use std::{
    ffi::{CStr, c_void},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

#[cfg(target_os = "linux")]
use ash::vk::Handle as _;
use egui::{Align2, Color32, Context, FontId, RichText, Sense, TextureId, Vec2};
use egui_winit::State as EguiWinitState;
use fluxa_app::FluxaRuntime;
use fluxa_artwork::{ArtworkFetcher, Priority as ArtworkFetchPriority};
use fluxa_core::FluxaCore;
use fluxa_effects::{EffectCompletion, EffectExecutor};
use fluxa_effects::{Storage, storage as platform};
use fluxa_renderer::egui_wgpu_backend::{EguiWgpuBackend, ScreenDescriptor};
use fluxa_renderer::{
    AnimationClock, Color as SceneColor, Easing, Rect as SceneRect, RenderNode, RenderScene,
    SceneRenderer, Tween,
};
use fluxa_ui::{
    AnimatedTexture, ArtworkPriority, DetailModel, HomeAssets, HomeModel as SharedHomeModel,
    SettingsModel, UiFormFactor, UiFormFactorJson, Viewport, detail_model_from_core_snapshot,
    draw_detail, draw_home, draw_settings, home_model_from_core_snapshot,
    settings_model_from_core_snapshot,
};
use gilrs::{Button as GilrsButton, EventType as GilrsEventType, Gilrs};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use wgpu;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowId},
};

mod font_manager;
mod screen_hosts;
mod svg_icons;
#[cfg(target_os = "linux")]
mod wayland_presentation;
mod widgets;

#[cfg(target_os = "linux")]
use wayland_presentation::{FeedbackEvent, WaylandPresentation};

use fluxa_mpv as native_mpv;
#[cfg(test)]
use fluxa_mpv as mpv_render;

use screen_hosts::{
    draw_shared_calendar_screen, draw_shared_detail_screen, draw_shared_discover_screen,
    draw_shared_library_screen, draw_shared_settings_screen, enter_shared_desktop_page,
    shared_home_model,
};
use svg_icons::SvgIconRegistry;
use widgets::NativeTheme;

const ARCHIVO_BYTES: &[u8] =
    include_bytes!("../../../apps/android/app/src/main/res/font/archivo.ttf");
const BACKGROUND_BYTES: &[u8] =
    include_bytes!("../../../apps/desktop/public/welcome-background.png");
// Keep card artwork bounded while allowing hero surfaces to retain enough
// detail on high-density desktop displays. egui's portable texture budget is
// 2048 on the supported renderer path, so this avoids the old 1024px hero
// upscale that made wide backdrops visibly pixelated.
const MAX_SAFE_TEXTURE_SIDE: u32 = 2048;
// Decoding happens off the UI thread, so several already-prepared images can
// be handed to WGPU per frame without making the window appear stuck.
const MAX_ARTWORK_UPLOADS_PER_FRAME: usize = 8;
// The shared UI prefetches the complete initial home document. Keep enough
// native textures resident for all shelves so a loaded poster/logo is not
// evicted and immediately requested again on the next frame.
const MAX_ARTWORK_TEXTURES: usize = 256;

fn benchmark_target_fps() -> Option<u32> {
    std::env::var("FLUXA_NATIVE_BENCH_FPS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|fps| (1..=360).contains(fps))
}

fn active_redraw_interval(window: &Window) -> Duration {
    let refresh_millihz = window
        .current_monitor()
        .and_then(|monitor| monitor.refresh_rate_millihertz())
        .unwrap_or(60_000)
        // Keep background animation redraws smooth on high-refresh displays,
        // while avoiding a full 144/180 Hz repaint when content is static.
        // Input-triggered redraws and explicit FPS benchmarks bypass this cap.
        .clamp(30_000, 120_000);
    Duration::from_secs_f64(1_000.0 / f64::from(refresh_millihz))
}

struct NativeUi {
    context: Context,
    state: EguiWinitState,
    instance: wgpu::Instance,
    renderer: EguiWgpuBackend,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pending_resize: Option<winit::dpi::PhysicalSize<u32>>,
    resize_reconfigure_until: Option<Instant>,
    artwork: ArtworkRegistry,
    icons: SvgIconRegistry,
    scene_renderer: SceneRenderer,
    scene: RenderScene,
    home_model_cache: Option<CachedHomeModel>,
    frame_diagnostics: FrameDiagnostics,
    gpu_profiler: Option<GpuFrameProfiler>,
    benchmark_scroll_started: Option<Instant>,
    animations: UiAnimations,
    background_texture: egui::TextureHandle,
    gamepad: Option<Gilrs>,
    dev_reload: DevUiReload,
    player: NativePlayer,
    #[cfg(target_os = "linux")]
    wayland_presentation: Option<WaylandPresentation>,
    #[cfg(target_os = "linux")]
    next_presentation_frame_id: u64,
    #[cfg(target_os = "linux")]
    presentation_error_logged: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct PerfStage {
    total_ms: f64,
    max_ms: f64,
}

impl PerfStage {
    fn add(&mut self, value_ms: f64) {
        self.total_ms += value_ms;
        self.max_ms = self.max_ms.max(value_ms);
    }

    fn average_ms(self, samples: u64) -> f64 {
        if samples == 0 {
            0.0
        } else {
            self.total_ms / samples as f64
        }
    }
}

#[derive(Default)]
struct FrameDiagnostics {
    interval_started: Option<Instant>,
    last_frame_started: Option<Instant>,
    target_frame_ms: Option<f64>,
    frames: u64,
    late_intervals: u64,
    presentation_presented: u64,
    presentation_discarded: u64,
    presentation_refresh_total_ms: f64,
    presentation_sequence_skips: u64,
    presentation_nonzero_sequences: u64,
    presentation_interval_under_1x: u64,
    presentation_interval_1x: u64,
    presentation_interval_2x: u64,
    presentation_interval_3x: u64,
    presentation_interval_4x_plus: u64,
    presentation_interval_unknown_refresh: u64,
    presentation_vsync: u64,
    presentation_hw_clock: u64,
    presentation_hw_completion: u64,
    presentation_zero_copy: u64,
    last_presentation_ns: Option<u64>,
    last_presentation_sequence: Option<u64>,
    presentation_intervals_ms: Vec<f64>,
    request_to_presentation_ms: Vec<f64>,
    pending_presentation_frames: HashMap<u64, (f64, f64)>,
    presented_acquire_ms: Vec<f64>,
    presented_submit_ms: Vec<f64>,
    player_frames: u64,
    cpu_frame: PerfStage,
    run_ui: PerfStage,
    snapshot_share: PerfStage,
    home_inputs: PerfStage,
    home_model: PerfStage,
    home_draw: PerfStage,
    tessellate: PerfStage,
    texture_upload: PerfStage,
    egui_buffer_upload: PerfStage,
    scene_prepare: PerfStage,
    scene_upload: PerfStage,
    scene_encode: PerfStage,
    egui_encode: PerfStage,
    submit: PerfStage,
    present: PerfStage,
    frame_interval: PerfStage,
    surface_acquire: PerfStage,
    encoder_create: PerfStage,
    encoder_finish: PerfStage,
    gpu_scene: PerfStage,
    gpu_egui: PerfStage,
    gpu_samples: u64,
    meshes: u64,
    callbacks: u64,
    vertices: u64,
    indices: u64,
    scene_instances: u64,
    draw_calls: u64,
    texture_switches: u64,
    upload_bytes: u64,
    texture_upload_bytes: u64,
    home_model_cache_hits: u64,
    frame_timings: Vec<FrameTimings>,
}

#[derive(Clone, Copy, Default)]
struct FrameTimings {
    cpu_frame_ms: f64,
    run_ui_ms: f64,
    snapshot_share_ms: f64,
    home_inputs_ms: f64,
    home_model_ms: f64,
    home_draw_ms: f64,
    tessellate_ms: f64,
    texture_upload_ms: f64,
    egui_buffer_upload_ms: f64,
    surface_acquire_ms: f64,
    encoder_finish_ms: f64,
    queue_submit_ms: f64,
    present_ms: f64,
    frame_interval_ms: f64,
}

struct FrameSample {
    cpu_frame_ms: f64,
    run_ui_ms: f64,
    snapshot_share_ms: f64,
    home_inputs_ms: f64,
    home_model_ms: f64,
    home_draw_ms: f64,
    tessellate_ms: f64,
    texture_upload_ms: f64,
    egui_buffer_upload_ms: f64,
    scene: fluxa_renderer::SceneRenderStats,
    egui_encode_ms: f64,
    surface_acquire_ms: f64,
    encoder_create_ms: f64,
    encoder_finish_ms: f64,
    queue_submit_ms: f64,
    present_ms: f64,
    frame_started_at: Instant,
    meshes: usize,
    callbacks: usize,
    vertices: usize,
    indices: usize,
    draw_calls: usize,
    texture_switches: usize,
    egui_upload_bytes: usize,
    texture_upload_bytes: usize,
    player_active: bool,
    home_model_cache_hit: bool,
    #[cfg(target_os = "linux")]
    presentation_frame_id: Option<u64>,
}

#[derive(Default, Clone, Copy)]
struct UiWorkStats {
    snapshot_share_ms: f64,
    home_inputs_ms: f64,
    home_model_ms: f64,
    home_draw_ms: f64,
    home_model_cache_hit: bool,
}

struct CachedHomeModel {
    revision: u64,
    model: SharedHomeModel,
    billboard: Value,
}

impl FrameDiagnostics {
    fn record_presentation(&mut self, event: FeedbackEvent) {
        match event {
            FeedbackEvent::Presented {
                frame_id,
                requested_ns,
                presented_ns,
                refresh_ns,
                sequence,
                flags,
                ..
            } => {
                self.presentation_presented += 1;
                self.presentation_nonzero_sequences += u64::from(sequence != 0);
                if let Some((acquire_ms, submit_ms)) =
                    self.pending_presentation_frames.remove(&frame_id)
                {
                    self.presented_acquire_ms.push(acquire_ms);
                    self.presented_submit_ms.push(submit_ms);
                }
                if presented_ns >= requested_ns {
                    self.request_to_presentation_ms
                        .push((presented_ns - requested_ns) as f64 / 1_000_000.0);
                }
                if let Some(previous_ns) = self.last_presentation_ns
                    && presented_ns >= previous_ns
                {
                    let interval_ns = presented_ns - previous_ns;
                    self.presentation_intervals_ms
                        .push(interval_ns as f64 / 1_000_000.0);
                    if refresh_ns == 0 {
                        self.presentation_interval_unknown_refresh += 1;
                    } else {
                        let refresh_ratio = interval_ns as f64 / f64::from(refresh_ns);
                        if refresh_ratio < 0.5 {
                            self.presentation_interval_under_1x += 1;
                        } else if refresh_ratio < 1.5 {
                            self.presentation_interval_1x += 1;
                        } else if refresh_ratio < 2.5 {
                            self.presentation_interval_2x += 1;
                        } else if refresh_ratio < 3.5 {
                            self.presentation_interval_3x += 1;
                        } else {
                            self.presentation_interval_4x_plus += 1;
                        }
                    }
                }
                if sequence != 0
                    && let Some(previous_sequence) = self.last_presentation_sequence
                    && previous_sequence != 0
                    && sequence > previous_sequence.saturating_add(1)
                {
                    self.presentation_sequence_skips += sequence - previous_sequence - 1;
                }
                self.last_presentation_ns = Some(presented_ns);
                self.last_presentation_sequence = Some(sequence);
                self.presentation_refresh_total_ms += f64::from(refresh_ns) / 1_000_000.0;
                self.presentation_vsync += u64::from(flags & 0x1 != 0);
                self.presentation_hw_clock += u64::from(flags & 0x2 != 0);
                self.presentation_hw_completion += u64::from(flags & 0x4 != 0);
                self.presentation_zero_copy += u64::from(flags & 0x8 != 0);
            }
            FeedbackEvent::Discarded { frame_id } => {
                self.presentation_discarded += 1;
                self.pending_presentation_frames.remove(&frame_id);
            }
        }
    }

    fn record(&mut self, sample: FrameSample, gpu: Option<(f64, f64)>) {
        let now = Instant::now();
        let interval_started = *self.interval_started.get_or_insert(now);
        let frame_interval_ms = self
            .last_frame_started
            .replace(sample.frame_started_at)
            .map(|previous| {
                sample
                    .frame_started_at
                    .duration_since(previous)
                    .as_secs_f64()
                    * 1_000.0
            })
            .unwrap_or_default();
        self.frames += 1;
        if self
            .target_frame_ms
            .is_some_and(|target| frame_interval_ms > target * 1.5)
        {
            self.late_intervals += 1;
        }
        self.player_frames += u64::from(sample.player_active);
        self.home_model_cache_hits += u64::from(sample.home_model_cache_hit);
        self.cpu_frame.add(sample.cpu_frame_ms);
        self.run_ui.add(sample.run_ui_ms);
        self.snapshot_share.add(sample.snapshot_share_ms);
        self.home_inputs.add(sample.home_inputs_ms);
        self.home_model.add(sample.home_model_ms);
        self.home_draw.add(sample.home_draw_ms);
        self.tessellate.add(sample.tessellate_ms);
        self.texture_upload.add(sample.texture_upload_ms);
        self.egui_buffer_upload.add(sample.egui_buffer_upload_ms);
        self.scene_prepare.add(sample.scene.prepare_ms);
        self.scene_upload.add(sample.scene.upload_ms);
        self.scene_encode.add(sample.scene.encode_ms);
        self.egui_encode.add(sample.egui_encode_ms);
        self.submit.add(sample.queue_submit_ms);
        self.present.add(sample.present_ms);
        self.frame_interval.add(frame_interval_ms);
        self.surface_acquire.add(sample.surface_acquire_ms);
        self.encoder_create.add(sample.encoder_create_ms);
        self.encoder_finish.add(sample.encoder_finish_ms);
        if let Some((scene_ms, egui_ms)) = gpu {
            self.gpu_scene.add(scene_ms);
            self.gpu_egui.add(egui_ms);
            self.gpu_samples += 1;
        }
        self.meshes += sample.meshes as u64;
        self.callbacks += sample.callbacks as u64;
        self.vertices += sample.vertices as u64;
        self.indices += sample.indices as u64;
        self.scene_instances += sample.scene.instance_count as u64;
        self.draw_calls += sample.draw_calls as u64;
        self.texture_switches += sample.texture_switches as u64;
        self.upload_bytes += (sample.egui_upload_bytes + sample.scene.instance_upload_bytes) as u64;
        self.texture_upload_bytes += sample.texture_upload_bytes as u64;
        #[cfg(target_os = "linux")]
        if let Some(frame_id) = sample.presentation_frame_id {
            self.pending_presentation_frames.insert(
                frame_id,
                (sample.surface_acquire_ms, sample.queue_submit_ms),
            );
        }
        self.frame_timings.push(FrameTimings {
            cpu_frame_ms: sample.cpu_frame_ms,
            run_ui_ms: sample.run_ui_ms,
            snapshot_share_ms: sample.snapshot_share_ms,
            home_inputs_ms: sample.home_inputs_ms,
            home_model_ms: sample.home_model_ms,
            home_draw_ms: sample.home_draw_ms,
            tessellate_ms: sample.tessellate_ms,
            texture_upload_ms: sample.texture_upload_ms,
            egui_buffer_upload_ms: sample.egui_buffer_upload_ms,
            surface_acquire_ms: sample.surface_acquire_ms,
            encoder_finish_ms: sample.encoder_finish_ms,
            queue_submit_ms: sample.queue_submit_ms,
            present_ms: sample.present_ms,
            frame_interval_ms,
        });

        if now.duration_since(interval_started) < Duration::from_secs(2) {
            return;
        }
        let frames = self.frames;
        let avg = |stage: PerfStage| stage.average_ms(frames);
        let max = |stage: PerfStage| stage.max_ms;
        let divisor = frames.max(1) as f64;
        let percentiles = |select: fn(&FrameTimings) -> f64| {
            let mut values: Vec<f64> = self.frame_timings.iter().map(select).collect();
            values.sort_by(f64::total_cmp);
            let at = |fraction: f64| {
                if values.is_empty() {
                    0.0
                } else {
                    values[((values.len() as f64 * fraction).ceil() as usize)
                        .saturating_sub(1)
                        .min(values.len() - 1)]
                }
            };
            (at(0.50), at(0.95), at(0.99))
        };
        let (cpu_p50, cpu_p95, cpu_p99) = percentiles(|frame| frame.cpu_frame_ms);
        let (ui_p50, ui_p95, ui_p99) = percentiles(|frame| frame.run_ui_ms);
        let (snapshot_p50, snapshot_p95, snapshot_p99) =
            percentiles(|frame| frame.snapshot_share_ms);
        let (home_inputs_p50, home_inputs_p95, home_inputs_p99) =
            percentiles(|frame| frame.home_inputs_ms);
        let (home_model_p50, home_model_p95, home_model_p99) =
            percentiles(|frame| frame.home_model_ms);
        let (home_draw_p50, home_draw_p95, home_draw_p99) = percentiles(|frame| frame.home_draw_ms);
        let (tess_p50, tess_p95, tess_p99) = percentiles(|frame| frame.tessellate_ms);
        let (texture_p50, texture_p95, texture_p99) = percentiles(|frame| frame.texture_upload_ms);
        let (buffer_p50, buffer_p95, buffer_p99) = percentiles(|frame| frame.egui_buffer_upload_ms);
        let (finish_p50, finish_p95, finish_p99) = percentiles(|frame| frame.encoder_finish_ms);
        let (submit_p50, submit_p95, submit_p99) = percentiles(|frame| frame.queue_submit_ms);
        let (acquire_p50, acquire_p95, acquire_p99) = percentiles(|frame| frame.surface_acquire_ms);
        let (present_p50, present_p95, present_p99) = percentiles(|frame| frame.present_ms);
        let (interval_p50, interval_p95, interval_p99) =
            percentiles(|frame| frame.frame_interval_ms);
        let percentile_values = |values: &[f64]| {
            let mut values = values.to_vec();
            values.sort_by(f64::total_cmp);
            let at = |fraction: f64| {
                if values.is_empty() {
                    0.0
                } else {
                    values[((values.len() as f64 * fraction).ceil() as usize)
                        .saturating_sub(1)
                        .min(values.len() - 1)]
                }
            };
            (at(0.50), at(0.95), at(0.99))
        };
        let (presented_interval_p50, presented_interval_p95, presented_interval_p99) =
            percentile_values(&self.presentation_intervals_ms);
        let (request_present_p50, request_present_p95, request_present_p99) =
            percentile_values(&self.request_to_presentation_ms);
        let (presented_acquire_p50, presented_acquire_p95, presented_acquire_p99) =
            percentile_values(&self.presented_acquire_ms);
        let (presented_submit_p50, presented_submit_p95, presented_submit_p99) =
            percentile_values(&self.presented_submit_ms);
        eprintln!(
            "[fluxa-native] perf: {frames} frames/2s player={}/{} cpu={:.3}/{:.3}ms cpu_p50/p95/p99={:.3}/{:.3}/{:.3}ms run_ui={:.3}/{:.3}ms run_ui_p50/p95/p99={:.3}/{:.3}/{:.3}ms snapshot_share={:.3}/{:.3}ms snapshot_p50/p95/p99={:.3}/{:.3}/{:.3}ms home_inputs={:.3}/{:.3}ms inputs_p50/p95/p99={:.3}/{:.3}/{:.3}ms home_model_build={:.3}/{:.3}ms model_p50/p95/p99={:.3}/{:.3}/{:.3}ms model_cache_hits={}/{} home_draw={:.3}/{:.3}ms home_draw_p50/p95/p99={:.3}/{:.3}/{:.3}ms tess={:.3}/{:.3}ms tess_p50/p95/p99={:.3}/{:.3}/{:.3}ms surface_acquire={:.3}/{:.3}ms acquire_p50/p95/p99={:.3}/{:.3}/{:.3}ms encoder_create={:.3}/{:.3}ms encoder_finish={:.3}/{:.3}ms finish_p50/p95/p99={:.3}/{:.3}/{:.3}ms queue_submit={:.3}/{:.3}ms submit_p50/p95/p99={:.3}/{:.3}/{:.3}ms present={:.3}/{:.3}ms present_p50/p95/p99={:.3}/{:.3}/{:.3}ms frame_interval={:.3}/{:.3}ms interval_p50/p95/p99={:.3}/{:.3}/{:.3}ms target_ms={} late_intervals={}/{} wp_presented={} discarded={} presentation_interval_p50/p95/p99={:.3}/{:.3}/{:.3}ms request_to_present_p50/p95/p99={:.3}/{:.3}/{:.3}ms presented_frame_acquire_p50/p95/p99={:.3}/{:.3}/{:.3}ms presented_frame_submit_p50/p95/p99={:.3}/{:.3}/{:.3}ms refresh_avg={:.3}ms seq_skips={} flags(vsync/hw_clock/hw_completion/zero_copy)={}/{}/{}/{} texture_upload={:.3}/{:.3}ms texture_p50/p95/p99={:.3}/{:.3}/{:.3}ms egui_buffer_upload={:.3}/{:.3}ms buffer_p50/p95/p99={:.3}/{:.3}/{:.3}ms scene_prepare={:.3}/{:.3}ms scene_upload={:.3}/{:.3}ms scene_encode={:.3}/{:.3}ms egui_encode={:.3}/{:.3}ms gpu_scene={:.1}us gpu_egui={:.1}us gpu_samples={} meshes/frame={:.1} callbacks/frame={:.1} vertices/frame={:.0} indices/frame={:.0} instances/frame={:.1} draws/frame={:.1} texture_switches/frame={:.1} mesh_upload={:.1}KiB/frame texture_upload={:.1}KiB/frame",
            self.player_frames,
            frames,
            avg(self.cpu_frame),
            max(self.cpu_frame),
            cpu_p50,
            cpu_p95,
            cpu_p99,
            avg(self.run_ui),
            max(self.run_ui),
            ui_p50,
            ui_p95,
            ui_p99,
            avg(self.snapshot_share),
            max(self.snapshot_share),
            snapshot_p50,
            snapshot_p95,
            snapshot_p99,
            avg(self.home_inputs),
            max(self.home_inputs),
            home_inputs_p50,
            home_inputs_p95,
            home_inputs_p99,
            avg(self.home_model),
            max(self.home_model),
            home_model_p50,
            home_model_p95,
            home_model_p99,
            self.home_model_cache_hits,
            frames,
            avg(self.home_draw),
            max(self.home_draw),
            home_draw_p50,
            home_draw_p95,
            home_draw_p99,
            avg(self.tessellate),
            max(self.tessellate),
            tess_p50,
            tess_p95,
            tess_p99,
            avg(self.surface_acquire),
            max(self.surface_acquire),
            acquire_p50,
            acquire_p95,
            acquire_p99,
            avg(self.encoder_create),
            max(self.encoder_create),
            avg(self.encoder_finish),
            max(self.encoder_finish),
            finish_p50,
            finish_p95,
            finish_p99,
            avg(self.submit),
            max(self.submit),
            submit_p50,
            submit_p95,
            submit_p99,
            avg(self.present),
            max(self.present),
            present_p50,
            present_p95,
            present_p99,
            avg(self.frame_interval),
            max(self.frame_interval),
            interval_p50,
            interval_p95,
            interval_p99,
            self.target_frame_ms
                .map(|target| format!("{target:.3}"))
                .unwrap_or_else(|| "none".to_owned()),
            self.late_intervals,
            frames,
            self.presentation_presented,
            self.presentation_discarded,
            presented_interval_p50,
            presented_interval_p95,
            presented_interval_p99,
            request_present_p50,
            request_present_p95,
            request_present_p99,
            presented_acquire_p50,
            presented_acquire_p95,
            presented_acquire_p99,
            presented_submit_p50,
            presented_submit_p95,
            presented_submit_p99,
            if self.presentation_presented == 0 {
                0.0
            } else {
                self.presentation_refresh_total_ms / self.presentation_presented as f64
            },
            self.presentation_sequence_skips,
            self.presentation_vsync,
            self.presentation_hw_clock,
            self.presentation_hw_completion,
            self.presentation_zero_copy,
            avg(self.texture_upload),
            max(self.texture_upload),
            texture_p50,
            texture_p95,
            texture_p99,
            avg(self.egui_buffer_upload),
            max(self.egui_buffer_upload),
            buffer_p50,
            buffer_p95,
            buffer_p99,
            avg(self.scene_prepare),
            max(self.scene_prepare),
            avg(self.scene_upload),
            max(self.scene_upload),
            avg(self.scene_encode),
            max(self.scene_encode),
            avg(self.egui_encode),
            max(self.egui_encode),
            avg(self.gpu_scene) * 1_000.0,
            avg(self.gpu_egui) * 1_000.0,
            self.gpu_samples,
            self.meshes as f64 / divisor,
            self.callbacks as f64 / divisor,
            self.vertices as f64 / divisor,
            self.indices as f64 / divisor,
            self.scene_instances as f64 / divisor,
            self.draw_calls as f64 / divisor,
            self.texture_switches as f64 / divisor,
            self.upload_bytes as f64 / divisor / 1024.0,
            self.texture_upload_bytes as f64 / divisor / 1024.0,
        );
        if self.presentation_presented > 0 {
            let known_refresh_intervals = self.presentation_interval_under_1x
                + self.presentation_interval_1x
                + self.presentation_interval_2x
                + self.presentation_interval_3x
                + self.presentation_interval_4x_plus;
            let percentage = |count: u64| {
                if known_refresh_intervals == 0 {
                    0.0
                } else {
                    count as f64 * 100.0 / known_refresh_intervals as f64
                }
            };
            eprintln!(
                "[fluxa-native] presentation sequence: nonzero={}/{} skips={}",
                self.presentation_nonzero_sequences,
                self.presentation_presented,
                self.presentation_sequence_skips,
            );
            eprintln!(
                "[fluxa-native] presentation cadence: refresh_known={known_refresh_intervals} unknown_refresh={} buckets(<1x/1x/2x/3x/4x+)={:.1}/{:.1}/{:.1}/{:.1}/{:.1}%",
                self.presentation_interval_unknown_refresh,
                percentage(self.presentation_interval_under_1x),
                percentage(self.presentation_interval_1x),
                percentage(self.presentation_interval_2x),
                percentage(self.presentation_interval_3x),
                percentage(self.presentation_interval_4x_plus),
            );
        }
        let last_frame_started = self.last_frame_started;
        let target_frame_ms = self.target_frame_ms;
        let last_presentation_ns = self.last_presentation_ns;
        let last_presentation_sequence = self.last_presentation_sequence;
        let pending_presentation_frames = std::mem::take(&mut self.pending_presentation_frames);
        *self = Self {
            interval_started: Some(now),
            last_frame_started,
            target_frame_ms,
            last_presentation_ns,
            last_presentation_sequence,
            pending_presentation_frames,
            ..Self::default()
        };
    }
}

struct GpuFrameProfiler {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,
    sender: Sender<Result<(), String>>,
    receiver: Receiver<Result<(), String>>,
    frames: u64,
    map_pending: bool,
    timestamp_period_ns: f64,
}

impl GpuFrameProfiler {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("fluxa-frame-timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: 4,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluxa-frame-timestamp-resolve"),
            size: 4 * std::mem::size_of::<u64>() as u64,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluxa-frame-timestamp-readback"),
            size: 4 * std::mem::size_of::<u64>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let (sender, receiver) = mpsc::channel();
        Self {
            query_set,
            resolve_buffer,
            readback_buffer,
            sender,
            receiver,
            frames: 0,
            map_pending: false,
            timestamp_period_ns: queue.get_timestamp_period() as f64,
        }
    }

    fn begin_frame(&mut self) -> bool {
        self.frames += 1;
        self.frames % 60 == 0 && !self.map_pending
    }

    fn timestamp_writes(&self, start: u32, end: u32) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(start),
            end_of_pass_write_index: Some(end),
        }
    }

    fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(&self.query_set, 0..4, &self.resolve_buffer, 0);
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.readback_buffer,
            0,
            4 * std::mem::size_of::<u64>() as u64,
        );
    }

    fn map_after_submit(&mut self) {
        let sender = self.sender.clone();
        self.readback_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
            });
        self.map_pending = true;
    }

    fn poll(&mut self, device: &wgpu::Device) -> Option<(f64, f64)> {
        if !self.map_pending {
            return None;
        }
        let _ = device.poll(wgpu::PollType::Poll);
        let result = self.receiver.try_recv().ok()?;
        self.map_pending = false;
        if let Err(error) = result {
            eprintln!("[fluxa-native] GPU timestamp readback failed: {error}");
            return None;
        }
        let mapped = self.readback_buffer.slice(..).get_mapped_range();
        let mut values = [0_u64; 4];
        for (value, bytes) in values.iter_mut().zip(mapped.chunks_exact(8)) {
            *value = u64::from_ne_bytes(bytes.try_into().expect("timestamp is eight bytes"));
        }
        let to_ms = |ticks: u64| ticks as f64 * self.timestamp_period_ns / 1_000_000.0;
        let scene_ms = to_ms(values[1].wrapping_sub(values[0]));
        let egui_ms = to_ms(values[3].wrapping_sub(values[2]));
        drop(mapped);
        self.readback_buffer.unmap();
        Some((scene_ms, egui_ms))
    }
}

struct NativePlayer {
    #[cfg(target_os = "linux")]
    gpu: Option<NativeGpuPlayer>,
    #[cfg(target_os = "linux")]
    pending: Option<PendingNativeGpuPlayer>,
    blocked_url: Option<String>,
    error: Option<String>,
    torrent_link: Option<String>,
    torrent_status_receiver: Option<Receiver<Value>>,
    torrent_status: Option<Value>,
    overlay_visible: bool,
    overlay_last_activity: Instant,
    overlay_pointer: Option<egui::Pos2>,
    overlay_last_status_poll: Instant,
    overlay_position: f64,
    overlay_duration: f64,
    overlay_paused: bool,
    overlay_muted: bool,
    overlay_volume: f64,
    overlay_scrubbing: bool,
    overlay_last_seek: Instant,
    video_texture_id: Option<TextureId>,
}

#[cfg(target_os = "linux")]
struct NativeGpuPlayer {
    // mpv and wgpu use the same Vulkan device. mpv writes directly into this
    // wgpu-owned image; the UI samples it in the same parent surface pass.
    render: native_mpv::MpvRenderState,
    client: native_mpv::MpvClientHandle,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    loaded_url: String,
    width: i32,
    height: i32,
    image_layout: i32,
    vulkan_sync: NativeVulkanSync,
    last_diagnostic: std::time::Instant,
    first_frame_presented: bool,
    last_logged_hwdec: Option<String>,
    decode_status_logged: bool,
    last_hwdec_check: std::time::Instant,
    last_decode_log: std::time::Instant,
}

#[cfg(target_os = "linux")]
struct NativeVulkanSync {
    device: ash::Device,
    semaphores: [ash::vk::Semaphore; 2],
    previous_signal: Option<ash::vk::Semaphore>,
    next_signal_index: usize,
}

#[cfg(target_os = "linux")]
impl NativeVulkanSync {
    fn new(device: &wgpu::Device) -> Result<Self, String> {
        let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
            .ok_or_else(|| "wgpu Vulkan device handle unavailable for mpv semaphores".to_owned())?;
        let vk_device = hal_device.raw_device().clone();
        let create_info = ash::vk::SemaphoreCreateInfo::default();
        let first = unsafe { vk_device.create_semaphore(&create_info, None) }
            .map_err(|error| format!("create first mpv Vulkan semaphore: {error:?}"))?;
        let second = match unsafe { vk_device.create_semaphore(&create_info, None) } {
            Ok(semaphore) => semaphore,
            Err(error) => {
                unsafe { vk_device.destroy_semaphore(first, None) };
                return Err(format!("create second mpv Vulkan semaphore: {error:?}"));
            }
        };
        Ok(Self {
            device: vk_device,
            semaphores: [first, second],
            previous_signal: None,
            next_signal_index: 0,
        })
    }

    fn next_pair(&mut self) -> (ash::vk::Semaphore, ash::vk::Semaphore) {
        let signal = self.semaphores[self.next_signal_index];
        self.next_signal_index = (self.next_signal_index + 1) % self.semaphores.len();
        let wait = self
            .previous_signal
            .replace(signal)
            .unwrap_or(ash::vk::Semaphore::null());
        (wait, signal)
    }
}

#[cfg(target_os = "linux")]
impl Drop for NativeVulkanSync {
    fn drop(&mut self) {
        unsafe {
            // libmpv and wgpu share this Vulkan device. Wait for both queues
            // before destroying semaphores that may still be referenced by
            // their submitted work.
            let _ = self.device.device_wait_idle();
            for semaphore in self.semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
        }
    }
}

#[cfg(target_os = "linux")]
struct PendingNativeGpuPlayer {
    receiver: Receiver<Result<NativeGpuPlayer, String>>,
    loaded_url: String,
}

#[cfg(target_os = "linux")]
impl Drop for NativeGpuPlayer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.vulkan_sync.device.device_wait_idle();
        }
    }
}

impl Default for NativePlayer {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            gpu: None,
            #[cfg(target_os = "linux")]
            pending: None,
            blocked_url: None,
            error: None,
            torrent_link: None,
            torrent_status_receiver: None,
            torrent_status: None,
            overlay_visible: true,
            overlay_last_activity: Instant::now(),
            overlay_pointer: None,
            overlay_last_status_poll: Instant::now() - Duration::from_secs(1),
            overlay_position: 0.0,
            overlay_duration: 0.0,
            overlay_paused: false,
            overlay_muted: false,
            overlay_volume: 100.0,
            overlay_scrubbing: false,
            overlay_last_seek: Instant::now() - Duration::from_secs(1),
            video_texture_id: None,
        }
    }
}

impl NativePlayer {
    fn active(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            return self
                .gpu
                .as_ref()
                .is_some_and(|player| player.first_frame_presented);
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn active_or_initializing(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            return self.gpu.is_some() || self.pending.is_some();
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn stop(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(player) = self.gpu.take() {
            self.blocked_url = Some(player.loaded_url.clone());
            let _ = player.client.command(&["stop"]);
        }
        #[cfg(target_os = "linux")]
        if let Some(pending) = self.pending.take() {
            self.blocked_url = Some(pending.loaded_url.clone());
        }
        self.error = None;
        self.torrent_link = None;
        self.torrent_status_receiver = None;
        self.torrent_status = None;
        self.video_texture_id = None;
    }

    fn invalidate_video_surface(&mut self, reason: &str) {
        #[cfg(target_os = "linux")]
        {
            // The video is now an image in the parent wgpu pass. There is no
            // Wayland child surface to detach or reattach during focus and
            // fullscreen transitions.
            eprintln!("[fluxa-native] player: parent-surface video transition after {reason}");
        }
    }

    fn sync(
        &mut self,
        _window: &Window,
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        runtime: &FluxaRuntime,
        executor: &EffectExecutor,
        playback_requested: bool,
        _width: u32,
        _height: u32,
    ) {
        self.poll_torrent_status(runtime, executor, playback_requested);
        let url = runtime
            .snapshot()
            .pointer("/player/resolvedUrl")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        #[cfg(debug_assertions)]
        let url =
            url.map(|resolved| std::env::var("FLUXA_NATIVE_TEST_PLAYER_URL").unwrap_or(resolved));
        let Some(url) = url else {
            return;
        };
        if self.blocked_url.as_deref() == Some(url.as_str()) {
            return;
        }
        if self.blocked_url.as_deref() != Some(url.as_str()) {
            self.blocked_url = None;
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(pending) = self.pending.as_ref() {
                if pending.loaded_url != url {
                    return;
                }
                match pending.receiver.try_recv() {
                    Ok(Ok(player)) => {
                        let pending = self.pending.take().expect("pending player exists");
                        let _ = pending;
                        self.gpu = Some(player);
                        self.error = None;
                    }
                    Ok(Err(error)) => {
                        eprintln!("[fluxa-native] player: shared Vulkan setup failed: {error}");
                        let _ = self.pending.take();
                        self.blocked_url = Some(url);
                        self.error = Some(error);
                        return;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => return,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        eprintln!("[fluxa-native] player: Vulkan initialization worker stopped");
                        let _ = self.pending.take();
                        self.blocked_url = Some(url);
                        self.error = Some("Vulkan player initialization worker stopped".to_owned());
                        return;
                    }
                }
            }
            let needs_new_player = self
                .gpu
                .as_ref()
                .is_none_or(|player| player.loaded_url != url);
            if needs_new_player {
                self.stop();
                match PendingNativeGpuPlayer::begin(instance, device, &url, 1920, 1080) {
                    Ok(pending) => {
                        self.pending = Some(pending);
                        self.error = None;
                    }
                    Err(error) => {
                        self.blocked_url = Some(url);
                        self.error = Some(error);
                    }
                }
                return;
            }
            let Some(player) = self.gpu.as_mut() else {
                return;
            };
            match player.render() {
                Ok(()) => self.error = None,
                Err(error) => {
                    if self.error.as_deref() != Some(error.as_str()) {
                        eprintln!(
                            "[fluxa-native] player: Vulkan frame failed; stopping this player: {error}"
                        );
                    }
                    self.gpu.take();
                    self.blocked_url = Some(url);
                    self.error = Some(error);
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (instance, device, _width, _height, _window);
            self.error = Some("GPU libmpv host is not wired for this platform yet".to_owned());
        }
    }

    fn poll_torrent_status(
        &mut self,
        runtime: &FluxaRuntime,
        executor: &EffectExecutor,
        playback_requested: bool,
    ) {
        if !playback_requested {
            self.torrent_link = None;
            self.torrent_status_receiver = None;
            self.torrent_status = None;
            return;
        }
        while let Some(status) = self
            .torrent_status_receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok())
        {
            self.torrent_status = Some(status);
        }

        let snapshot = runtime.snapshot().clone();
        let player = snapshot.get("player");
        let streams = player
            .and_then(|value| value.get("currentStreams"))
            .and_then(Value::as_array);
        let selected = player
            .and_then(|value| value.get("currentStreamIndex"))
            .and_then(Value::as_u64)
            .and_then(|index| streams?.get(index as usize));
        let Some(stream) = selected else {
            self.torrent_link = None;
            self.torrent_status_receiver = None;
            self.torrent_status = None;
            return;
        };
        let Some(link) = FluxaCore::stream_magnet_link_json(&stream.to_string()) else {
            self.torrent_link = None;
            self.torrent_status_receiver = None;
            self.torrent_status = None;
            return;
        };
        if self.torrent_link.as_deref() == Some(link.as_str()) {
            return;
        }
        let file_id = stream
            .get("fileIdx")
            .or_else(|| stream.get("fileIndex"))
            .and_then(Value::as_u64)
            .map(|index| index as usize);
        self.torrent_link = Some(link.clone());
        self.torrent_status = None;
        self.torrent_status_receiver = Some(executor.poll_torrent_status(link, file_id));
    }

    fn ensure_video_texture_id(&mut self, renderer: &mut EguiWgpuBackend, device: &wgpu::Device) {
        #[cfg(target_os = "linux")]
        if self.video_texture_id.is_none()
            && let Some(player) = self.gpu.as_ref()
        {
            self.video_texture_id = Some(renderer.register_native_texture(
                device,
                &player.view,
                wgpu::FilterMode::Linear,
            ));
        }
    }
}

#[cfg(target_os = "linux")]
impl PendingNativeGpuPlayer {
    fn begin(
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        url: &str,
        width: i32,
        height: i32,
    ) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        let instance = instance.clone();
        let device = device.clone();
        let loaded_url = url.to_owned();
        let worker_url = loaded_url.clone();
        std::thread::spawn(move || {
            let result = NativeGpuPlayer::new(&instance, &device, &worker_url, width, height);
            let _ = sender.send(result);
        });
        Ok(Self {
            receiver,
            loaded_url,
        })
    }
}

#[cfg(target_os = "linux")]
impl NativeGpuPlayer {
    fn new(
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        url: &str,
        width: i32,
        height: i32,
    ) -> Result<Self, String> {
        let started_at = std::time::Instant::now();
        let local_stream =
            url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:");
        let (mut client, mut render) = if local_stream {
            native_mpv::MpvClientHandle::new_without_ytdl()?
        } else {
            native_mpv::MpvClientHandle::new()?
        };
        let _ = client.set_log_level("debug");
        eprintln!(
            "[fluxa-native] player: libmpv client created in {:?}",
            started_at.elapsed()
        );
        let handles = wgpu_vulkan_handles(instance, device)?;
        if let Err(error) = render.create_vulkan_context(
            handles.instance,
            handles.phys_device,
            handles.device,
            handles.queue_index,
            1,
            handles.get_proc_address,
            &handles.extensions,
        ) {
            // Drain libplacebo/mpv diagnostics before returning the compact
            // libmpv error code. These messages identify the exact imported
            // device feature or extension that wgpu did not enable.
            let _ = client.poll_events();
            for line in client.recent_log_lines() {
                eprintln!("[fluxa-native] player: mpv: {line}");
            }
            return Err(error);
        }
        eprintln!(
            "[fluxa-native] player: libmpv Vulkan context attached to wgpu device in {:?}",
            started_at.elapsed()
        );

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fluxa-mpv-zero-copy-frame"),
            size: wgpu::Extent3d {
                width: width.max(2) as u32,
                height: height.max(2) as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Match the format used by the working native Vulkan swapchain.
            // libmpv's Vulkan target path is already validated for BGRA8.
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let vulkan_sync = NativeVulkanSync::new(device)?;
        client.load(url, None)?;
        eprintln!(
            "[fluxa-native] player: libmpv Vulkan zero-copy image ready {}x{}",
            width, height
        );
        Ok(Self {
            texture,
            view,
            client,
            render,
            loaded_url: url.to_owned(),
            width,
            height,
            vulkan_sync,
            // VK_IMAGE_LAYOUT_UNDEFINED. mpv returns the layout it leaves the
            // image in; that value must be fed back on the next render call.
            image_layout: 0,
            last_diagnostic: std::time::Instant::now(),
            first_frame_presented: false,
            last_logged_hwdec: None,
            decode_status_logged: false,
            last_hwdec_check: std::time::Instant::now(),
            last_decode_log: std::time::Instant::now(),
        })
    }

    fn render(&mut self) -> Result<(), String> {
        for event in self.client.poll_events() {
            if let native_mpv::PlayerEvent::EndFile {
                error: Some(error), ..
            } = event
            {
                eprintln!("[fluxa-native] player: mpv ended before first frame: {error}");
                for line in self.client.recent_log_lines() {
                    eprintln!("[fluxa-native] player: mpv: {line}");
                }
                return Err(error);
            }
        }
        if !self.decode_status_logged
            || self.last_hwdec_check.elapsed() >= std::time::Duration::from_secs(1)
        {
            self.last_hwdec_check = std::time::Instant::now();
            let hwdec_current = self.client.query_property("hwdec-current");
            if !self.decode_status_logged
                || hwdec_current != self.last_logged_hwdec
                || self.last_decode_log.elapsed() >= std::time::Duration::from_secs(5)
            {
                let codec = self.client.query_property("video-codec");
                let format = self.client.query_property("video-format");
                eprintln!(
                    "[fluxa-native] player: decode status hwdec-current={} video-codec={} video-format={}",
                    hwdec_current.as_deref().unwrap_or("not-reported"),
                    codec.as_deref().unwrap_or("not-reported"),
                    format.as_deref().unwrap_or("not-reported"),
                );
                self.last_logged_hwdec = hwdec_current;
                self.decode_status_logged = true;
                self.last_decode_log = std::time::Instant::now();
            }
        }
        if !self.render.vulkan_frame_ready() {
            if !self.first_frame_presented
                && self.last_diagnostic.elapsed() >= std::time::Duration::from_secs(5)
            {
                eprintln!(
                    "[fluxa-native] player: waiting for first video frame; cache={:?}, position={:?}, mpv={:?}",
                    self.client.query_property("paused-for-cache"),
                    self.client.query_property("time-pos"),
                    self.client.recent_log_lines().last(),
                );
                self.last_diagnostic = std::time::Instant::now();
            }
            return Ok(());
        }

        let hal_texture = unsafe {
            self.texture
                .as_hal::<wgpu::hal::api::Vulkan>()
                .ok_or_else(|| "wgpu Vulkan texture handle unavailable".to_owned())?
        };
        let image = unsafe { hal_texture.raw_handle() }.as_raw();
        let (wait_semaphore, signal_semaphore) = self.vulkan_sync.next_pair();
        let mut target = native_mpv::VulkanTargetImage {
            image,
            // VK_FORMAT_B8G8R8A8_UNORM
            format: 44,
            w: self.width,
            h: self.height,
            // Match the working native Vulkan target contract. The wgpu
            // texture itself also has TEXTURE_BINDING usage so the parent UI
            // can sample it after mpv finishes rendering.
            usage: (ash::vk::ImageUsageFlags::COLOR_ATTACHMENT | ash::vk::ImageUsageFlags::SAMPLED)
                .as_raw(),
            layout: self.image_layout,
            wait_semaphore: wait_semaphore.as_raw(),
            signal_semaphore: signal_semaphore.as_raw(),
        };
        if let Err(error) = self.render.render_vulkan_frame(&mut target) {
            eprintln!(
                "[fluxa-native] player: Vulkan target rejected: image={:#x} format={} usage={:#x} layout={} size={}x{} wait={:#x} signal={:#x}",
                target.image,
                target.format,
                target.usage,
                target.layout,
                target.w,
                target.h,
                target.wait_semaphore,
                target.signal_semaphore,
            );
            for line in self.client.recent_log_lines() {
                eprintln!("[fluxa-native] player: mpv: {line}");
            }
            return Err(error);
        }
        self.image_layout = target.layout;
        self.render.report_swap();
        if !self.first_frame_presented {
            eprintln!("[fluxa-native] player: first Vulkan frame written to wgpu texture");
            self.first_frame_presented = true;
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
struct WgpuVulkanHandles {
    instance: *mut c_void,
    phys_device: *mut c_void,
    device: *mut c_void,
    queue_index: u32,
    get_proc_address: *mut c_void,
    extensions: Vec<*const i8>,
}

#[cfg(target_os = "linux")]
fn open_wgpu_vulkan_device(
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'_>,
) -> Result<(wgpu::Device, wgpu::Queue), String> {
    let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu adapter is not backed by Vulkan HAL".to_owned())?;
    let required_external_extensions = [
        CStr::from_bytes_with_nul(b"VK_KHR_external_memory_fd\0")
            .expect("static Vulkan extension name"),
        CStr::from_bytes_with_nul(b"VK_KHR_external_semaphore_fd\0")
            .expect("static Vulkan extension name"),
    ];
    let physical_device = hal_adapter.physical_device_capabilities();
    let missing = required_external_extensions
        .iter()
        .filter(|extension| !physical_device.supports_extension(extension))
        .map(|extension| extension.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "Vulkan adapter {} cannot provide CUDA/NVDEC zero-copy interop; missing device extension(s): {}",
            adapter.get_info().name,
            missing.join(", ")
        ));
    }

    let extensions_to_enable = required_external_extensions.to_vec();
    let callback = Box::new(
        move |args: wgpu::hal::vulkan::CreateDeviceCallbackArgs<'_, '_, '_>| {
            for extension in &extensions_to_enable {
                if !args.extensions.contains(extension) {
                    args.extensions.push(extension);
                }
            }
        },
    );
    let hal_device = unsafe {
        hal_adapter.open_with_callback(
            descriptor.required_features,
            &descriptor.required_limits,
            &descriptor.memory_hints,
            Some(callback),
        )
    }
    .map_err(|error| format!("failed to open Vulkan device with external FD interop: {error}"))?;

    let enabled_extensions = hal_device.device.enabled_device_extensions();
    for required in required_external_extensions {
        if !enabled_extensions
            .iter()
            .any(|enabled| *enabled == required)
        {
            return Err(format!(
                "wgpu HAL did not retain required Vulkan interop extension {}",
                required.to_string_lossy()
            ));
        }
    }
    eprintln!(
        "[fluxa-native] Vulkan external interop enabled: VK_KHR_external_memory_fd, VK_KHR_external_semaphore_fd"
    );
    unsafe { adapter.create_device_from_hal(hal_device, descriptor) }
        .map_err(|error| format!("failed to create wgpu device from Vulkan HAL: {error}"))
}

#[cfg(target_os = "linux")]
fn wgpu_vulkan_handles(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
) -> Result<WgpuVulkanHandles, String> {
    let hal_instance = unsafe { instance.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu is not using the Vulkan backend".to_owned())?;
    let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu Vulkan device handle unavailable".to_owned())?;
    let shared = hal_instance.shared_instance();
    let extensions = hal_device
        .enabled_device_extensions()
        .iter()
        .map(|extension| extension.as_ptr())
        .collect();
    Ok(WgpuVulkanHandles {
        instance: shared.raw_instance().handle().as_raw() as *mut c_void,
        phys_device: hal_device.raw_physical_device().as_raw() as *mut c_void,
        device: hal_device.raw_device().handle().as_raw() as *mut c_void,
        queue_index: hal_device.queue_family_index(),
        get_proc_address: shared.entry().static_fn().get_instance_proc_addr as *mut c_void,
        extensions,
    })
}

struct DevUiReload {
    enabled: bool,
    path: PathBuf,
    modified: Option<SystemTime>,
}

impl DevUiReload {
    fn new() -> Self {
        let enabled =
            cfg!(debug_assertions) && std::env::var_os("FLUXA_NATIVE_DISABLE_HOT_RELOAD").is_none();
        let path = std::env::var_os("FLUXA_NATIVE_TOKENS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../shared/contracts/ui-tokens.json")
            });
        let modified = std::fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok();
        Self {
            enabled,
            path,
            modified,
        }
    }

    fn poll(&mut self, context: &Context) {
        if !self.enabled {
            return;
        }
        let modified = std::fs::metadata(&self.path)
            .and_then(|metadata| metadata.modified())
            .ok();
        if modified.is_none() || modified == self.modified {
            return;
        }
        self.modified = modified;
        match fluxa_renderer::theme::reload_shared_ui_tokens_from_path(&self.path) {
            Ok(()) => {
                eprintln!(
                    "[fluxa-native] hot-reloaded shared UI tokens from {}",
                    self.path.display()
                );
                context.request_repaint();
            }
            Err(error) => eprintln!(
                "[fluxa-native] UI token hot reload failed for {}: {error}",
                self.path.display()
            ),
        }
    }
}

struct DesktopHomeAssets<'a> {
    background: TextureId,
    artwork: &'a mut ArtworkRegistry,
}

fn desktop_gamepad_key(button: GilrsButton) -> Option<egui::Key> {
    match button {
        GilrsButton::South | GilrsButton::Start => Some(egui::Key::Enter),
        GilrsButton::East | GilrsButton::Select => Some(egui::Key::Escape),
        GilrsButton::DPadUp => Some(egui::Key::ArrowUp),
        GilrsButton::DPadDown => Some(egui::Key::ArrowDown),
        GilrsButton::DPadLeft => Some(egui::Key::ArrowLeft),
        GilrsButton::DPadRight => Some(egui::Key::ArrowRight),
        _ => None,
    }
}

impl HomeAssets for DesktopHomeAssets<'_> {
    fn background(&self) -> TextureId {
        self.background
    }
    fn texture(&mut self, url: Option<&str>) -> Option<TextureId> {
        self.artwork.texture(url)
    }
    fn texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<TextureId> {
        self.artwork.texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        )
    }
    fn animated_texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        self.artwork
            .animated_texture_for_priority(url, target_size, priority)
    }
    fn prefetch_animated_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        self.artwork.prefetch_animated_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        );
    }
    fn prefetch_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        let _ = self.artwork.texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        );
    }
    fn texture_size(&self, url: Option<&str>) -> Option<[u32; 2]> {
        self.artwork.size(url)
    }
    fn cached_texture(&self, url: Option<&str>) -> Option<TextureId> {
        url.and_then(fluxa_artwork::normalize_url)
            .and_then(|url| self.artwork.latest_keys.get(&url))
            .and_then(|key| self.artwork.textures.get(key))
            .map(|texture| texture.id)
    }

    fn icon(&self, name: &str) -> Option<TextureId> {
        self.artwork.icon(name)
    }

    fn active_profile_name(&self) -> Option<&str> {
        Some(&self.artwork.active_profile_name)
    }

    fn active_profile_avatar_url(&self) -> Option<&str> {
        self.artwork.active_profile_avatar_url.as_deref()
    }

    fn accent_color(&self) -> Option<Color32> {
        Some(self.artwork.accent_color)
    }
}

struct AnimatedValue {
    target: f32,
    tween: Tween,
}

struct UiAnimations {
    clock: AnimationClock,
    values: HashMap<String, AnimatedValue>,
    active: bool,
    last_page: String,
}

impl Default for UiAnimations {
    fn default() -> Self {
        Self {
            clock: AnimationClock::default(),
            values: HashMap::new(),
            active: false,
            last_page: String::new(),
        }
    }
}

impl UiAnimations {
    fn begin_frame(&mut self) {
        self.clock.tick();
        self.active = false;
    }

    fn value(
        &mut self,
        key: impl Into<String>,
        target: f32,
        duration_ms: u64,
        easing: Easing,
    ) -> f32 {
        let key = key.into();
        let delta = self.clock.delta();
        let entry = self.values.entry(key).or_insert_with(|| AnimatedValue {
            target,
            tween: Tween::new(target, target, Duration::ZERO, easing),
        });
        if (entry.target - target).abs() > f32::EPSILON {
            let current = entry.tween.tick(Duration::ZERO);
            entry.target = target;
            entry.tween = Tween::new(current, target, Duration::from_millis(duration_ms), easing);
        }
        let value = entry.tween.tick(delta);
        self.active |= !entry.tween.finished();
        value
    }

    fn page_opacity(&mut self, page: &str) -> f32 {
        if self.last_page != page {
            self.last_page = page.to_owned();
            self.values.insert(
                "route-opacity".to_owned(),
                AnimatedValue {
                    target: 0.0,
                    tween: Tween::new(0.0, 0.0, Duration::ZERO, Easing::Linear),
                },
            );
        }
        self.value("route-opacity", 1.0, 220, Easing::EaseOutCubic)
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

struct ArtworkRegistry {
    fetcher: ArtworkFetcher,
    disabled: bool,
    textures: HashMap<String, NativeArtworkTexture>,
    animations: HashMap<String, NativeArtworkAnimation>,
    animation_checked: HashSet<String>,
    active_animations: HashSet<String>,
    animation_slots: HashSet<String>,
    animation_start_times: HashMap<String, Instant>,
    latest_keys: HashMap<String, String>,
    icon_textures: HashMap<&'static str, TextureId>,
    active_profile_name: String,
    active_profile_avatar_url: Option<String>,
    accent_color: Color32,
    access_counter: u64,
}

struct NativeArtworkTexture {
    _texture: Option<wgpu::Texture>,
    id: TextureId,
    size: [u32; 2],
    last_used: u64,
}

struct NativeArtworkAnimation {
    frames: Vec<fluxa_artwork::AnimatedAtlasFrame>,
    frame_index: usize,
    ready_frame_count: usize,
    started_at: Option<Instant>,
    next_frame_at: Instant,
}

impl ArtworkRegistry {
    fn new() -> Self {
        let cache_dir = platform::data_dir()
            .ok()
            .map(|directory| directory.join("artwork-cache"));
        Self {
            fetcher: ArtworkFetcher::new(cache_dir, 6),
            disabled: std::env::var_os("FLUXA_NATIVE_DISABLE_ARTWORK").is_some(),
            textures: HashMap::new(),
            animations: HashMap::new(),
            animation_checked: HashSet::new(),
            active_animations: HashSet::new(),
            animation_slots: HashSet::new(),
            animation_start_times: HashMap::new(),
            latest_keys: HashMap::new(),
            icon_textures: HashMap::new(),
            active_profile_name: "Profile".to_owned(),
            active_profile_avatar_url: None,
            accent_color: Color32::WHITE,
            access_counter: 0,
        }
    }

    fn set_icon_textures(&mut self, icons: &SvgIconRegistry) {
        self.icon_textures = fluxa_renderer::svg_icons::ICONS
            .iter()
            .filter_map(|(name, _)| icons.texture(name).map(|id| (*name, id)))
            .collect();
    }

    fn icon(&self, name: &str) -> Option<TextureId> {
        self.icon_textures.get(name).copied()
    }

    fn set_active_profile(&mut self, profile: Option<&Value>) {
        let profile = profile.unwrap_or(&Value::Null);
        self.active_profile_name = ["displayName", "name"]
            .into_iter()
            .filter_map(|key| profile.get(key).and_then(Value::as_str))
            .find(|value| !value.trim().is_empty())
            .unwrap_or("Profile")
            .to_owned();
        self.active_profile_avatar_url = ["avatarUrl", "avatar", "picture", "image"]
            .into_iter()
            .filter_map(|key| profile.get(key).and_then(Value::as_str))
            .find(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned);
        if let Some(url) = self.active_profile_avatar_url.clone() {
            let _ = self.texture_for_priority(Some(&url), [96, 96], ArtworkFetchPriority::Prefetch);
        }
    }

    fn set_accent_from_snapshot(&mut self, snapshot: &Value) {
        let Some(value) = snapshot
            .pointer("/settings/values/accentColorArgb")
            .and_then(Value::as_str)
        else {
            return;
        };
        let hex = value.trim_start_matches('#');
        let Ok(rgb) = u32::from_str_radix(hex, 16) else {
            return;
        };
        self.accent_color = if hex.len() == 8 {
            Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
        } else if hex.len() == 6 {
            Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
        } else {
            return;
        };
    }

    fn poll(
        &mut self,
        context: &Context,
        renderer: &mut EguiWgpuBackend,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        for prepared in self.fetcher.poll(MAX_ARTWORK_UPLOADS_PER_FRAME) {
            let url = prepared.source_url;
            let target_size = prepared.target_size;
            let animation_requested = prepared.animation_requested;
            let animation_started_at = prepared.animation_started_at;
            let animation_atlas = prepared.animation_atlas;
            let image = animation_atlas
                .as_ref()
                .map(|atlas| &atlas.image)
                .unwrap_or(&prepared.image);
            if image.width() == 0
                || image.height() == 0
                || image.width() > MAX_SAFE_TEXTURE_SIDE
                || image.height() > MAX_SAFE_TEXTURE_SIDE
            {
                eprintln!(
                    "[fluxa-native] refusing artwork outside safe texture bounds: {}x{}",
                    image.width(),
                    image.height()
                );
                continue;
            }
            let image_size = [image.width(), image.height()];
            let base_key = fluxa_artwork::request_key(&url, target_size);
            let key = if animation_requested {
                format!("{base_key}#animated-atlas")
            } else {
                base_key
            };
            let animation_started_at = self
                .animation_start_times
                .remove(&key)
                .or(animation_started_at);
            let size = wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fluxa-artwork"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                image.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.width()),
                    rows_per_image: Some(image.height()),
                },
                size,
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let id = renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear);
            if !animation_requested {
                self.latest_keys.insert(url.clone(), key.clone());
            }
            if animation_requested {
                self.animation_checked.insert(key.clone());
            }
            if let Some(previous) = self.textures.insert(
                key.clone(),
                NativeArtworkTexture {
                    _texture: Some(texture),
                    id,
                    size: [image.width(), image.height()],
                    last_used: self.access_counter,
                },
            ) {
                renderer.free_texture(&previous.id);
            }
            if let Some(atlas) = animation_atlas.filter(|atlas| atlas.frames.len() > 1) {
                let frames = atlas.frames;
                let now = Instant::now();
                let (frame_index, next_frame_at) = animation_started_at
                    .and_then(|started_at| {
                        fluxa_artwork::animation_frame_at(&frames, started_at, now)
                    })
                    .unwrap_or((0, now + frames[0].duration.max(Duration::from_millis(1))));
                self.animations.insert(
                    key.clone(),
                    NativeArtworkAnimation {
                        ready_frame_count: frames.len(),
                        frames,
                        frame_index,
                        started_at: animation_started_at,
                        next_frame_at,
                    },
                );
            }
            while self.textures.len() > MAX_ARTWORK_TEXTURES {
                let Some(oldest_url) = self
                    .textures
                    .iter()
                    .min_by_key(|(_, texture)| texture.last_used)
                    .map(|(url, _)| url.clone())
                else {
                    break;
                };
                if let Some(oldest) = self.textures.remove(&oldest_url) {
                    renderer.free_texture(&oldest.id);
                }
                self.animations.remove(&oldest_url);
                self.animation_checked.remove(&oldest_url);
                self.animation_start_times.remove(&oldest_url);
            }
            context.request_repaint();
        }
        let now = Instant::now();
        let active = self.active_animations.iter().cloned().collect::<Vec<_>>();
        let mut next_frame_in = None;
        for key in active {
            let Some(animation) = self.animations.get_mut(&key) else {
                continue;
            };
            if let Some(started_at) = animation.started_at
                && now >= animation.next_frame_at
            {
                let (frame_index, next_frame_at) =
                    fluxa_artwork::animation_frame_at(&animation.frames, started_at, now)
                        .unwrap_or((animation.frame_index, now + Duration::from_millis(100)));
                if frame_index != animation.frame_index {
                    animation.frame_index = frame_index;
                    context.request_repaint();
                }
                animation.next_frame_at = next_frame_at;
            }
            let wait = animation.next_frame_at.saturating_duration_since(now);
            next_frame_in = Some(next_frame_in.map_or(wait, |current: Duration| current.min(wait)));
        }
        if let Some(wait) = next_frame_in {
            context.request_repaint_after(wait.max(Duration::from_millis(1)));
        }
    }

    fn begin_frame(&mut self) {
        self.active_animations.clear();
        self.animation_slots.clear();
    }

    fn animated_texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        if self.disabled {
            return None;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_SAFE_TEXTURE_SIDE),
            target_size[1].min(MAX_SAFE_TEXTURE_SIDE),
        ]);
        let source_url = fluxa_artwork::normalize_url(url?)?;
        let base_key = fluxa_artwork::request_key(&source_url, target_size);
        let key = format!("{base_key}#animated-atlas");
        if self.animation_checked.contains(&key) && !self.animations.contains_key(&key) {
            return None;
        }
        if !self.animation_slots.contains(&key) {
            if self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
            {
                return None;
            }
            self.animation_slots.insert(key.clone());
        }
        self.access_counter = self.access_counter.wrapping_add(1);
        if let Some(texture) = self.textures.get_mut(&key) {
            texture.last_used = self.access_counter;
        }
        if self.animations.contains_key(&key) {
            let started_at = self
                .animations
                .get(&key)
                .and_then(|animation| animation.started_at)
                .or_else(|| self.animation_start_times.remove(&key))
                .unwrap_or_else(Instant::now);
            let now = Instant::now();
            let animation = self.animations.get_mut(&key)?;
            animation.started_at.get_or_insert(started_at);
            let (timeline_frame_index, next_frame_at) =
                fluxa_artwork::animation_frame_at(&animation.frames, animation.started_at?, now)?;
            let frame_index =
                timeline_frame_index.min(animation.ready_frame_count.saturating_sub(1));
            animation.frame_index = frame_index;
            animation.next_frame_at = next_frame_at;
            let frame = &animation.frames[frame_index];
            let texture = self.textures.get(&key)?;
            self.active_animations.insert(key.clone());
            return Some(AnimatedTexture {
                texture: texture.id,
                uv: egui::Rect::from_min_max(
                    egui::pos2(frame.uv[0], frame.uv[1]),
                    egui::pos2(frame.uv[2], frame.uv[3]),
                ),
                image_size: frame.image_size,
            });
        }
        if self.animation_checked.contains(&key) {
            return None;
        }
        let priority = match priority {
            ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
            ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
            ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
        };
        self.animation_start_times
            .entry(key)
            .or_insert_with(Instant::now);
        let _ = self
            .fetcher
            .request_animated(Some(&source_url), target_size, priority);
        None
    }

    fn prefetch_animated_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) {
        if self.disabled {
            return;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_SAFE_TEXTURE_SIDE),
            target_size[1].min(MAX_SAFE_TEXTURE_SIDE),
        ]);
        let Some(source_url) = url.and_then(fluxa_artwork::normalize_url) else {
            return;
        };
        let key = format!(
            "{}#animated-atlas",
            fluxa_artwork::request_key(&source_url, target_size)
        );
        if self.textures.contains_key(&key)
            || self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
        {
            return;
        }
        let _ = self
            .fetcher
            .prefetch_animated(Some(&source_url), target_size, priority);
    }

    fn texture(&mut self, url: Option<&str>) -> Option<TextureId> {
        self.texture_for_priority(
            url,
            [MAX_SAFE_TEXTURE_SIDE; 2],
            ArtworkFetchPriority::Visible,
        )
    }

    fn texture_for(&mut self, url: Option<&str>, target_size: [u32; 2]) -> Option<TextureId> {
        self.texture_for_priority(url, target_size, ArtworkFetchPriority::Visible)
    }

    fn texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) -> Option<TextureId> {
        if self.disabled {
            return None;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_SAFE_TEXTURE_SIDE),
            target_size[1].min(MAX_SAFE_TEXTURE_SIDE),
        ]);
        let url = fluxa_artwork::normalize_url(url?)?;
        let key = fluxa_artwork::request_key(&url, target_size);
        self.access_counter = self.access_counter.wrapping_add(1);
        if let Some(texture) = self.textures.get_mut(&key) {
            texture.last_used = self.access_counter;
            return Some(texture.id);
        }
        let _ = self.fetcher.request(Some(&url), target_size, priority);
        None
    }

    fn size(&self, url: Option<&str>) -> Option<[u32; 2]> {
        url.and_then(fluxa_artwork::normalize_url)
            .and_then(|url| self.latest_keys.get(&url))
            .and_then(|key| self.textures.get(key))
            .map(|texture| texture.size)
    }

    fn has_pending(&self) -> bool {
        self.fetcher.has_pending()
    }

    fn has_active_animation(&self) -> bool {
        !self.active_animations.is_empty()
    }
}

impl NativeUi {
    async fn new(window: Arc<Window>) -> Result<Self, String> {
        let mut descriptor =
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(window.clone()));
        descriptor.backends = native_wgpu_backends();
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| error.to_string())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| error.to_string())?;
        let adapter_info = adapter.get_info();
        eprintln!(
            "[fluxa-native] wgpu adapter: {:?} / {}",
            adapter_info.backend, adapter_info.name
        );
        let mut required_features = wgpu::Features::empty();
        #[cfg(target_os = "linux")]
        {
            // libplacebo's imported gpu-next Vulkan device requires
            // bufferDeviceAddress. wgpu-hal enables that device feature via
            // its Vulkan ray-query capability; Fluxa itself does not create
            // or use acceleration structures. Keep the UI usable on Vulkan
            // adapters that expose BDA but not ray queries; video interop can
            // report its own unsupported-device error when playback is used.
            let feature = wgpu::Features::EXPERIMENTAL_RAY_QUERY;
            if adapter.features().contains(feature) {
                required_features |= feature;
            } else {
                eprintln!(
                    "[fluxa-native] {} has no wgpu ray-query/BDA interop; starting UI without GPU libmpv interop",
                    adapter_info.name
                );
            }
        }
        let gpu_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        if gpu_timestamps {
            required_features |= wgpu::Features::TIMESTAMP_QUERY;
        } else {
            eprintln!(
                "[fluxa-native] GPU timestamp queries unavailable; GPU execution timings omitted"
            );
        }
        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("fluxa-egui-device"),
            required_features,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            #[cfg(target_os = "linux")]
            experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
            #[cfg(not(target_os = "linux"))]
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        };
        #[cfg(target_os = "linux")]
        let (device, queue) = open_wgpu_vulkan_device(&adapter, &device_descriptor)?;
        #[cfg(not(target_os = "linux"))]
        let (device, queue) = adapter
            .request_device(&device_descriptor)
            .await
            .map_err(|error| error.to_string())?;
        let gpu_profiler = gpu_timestamps.then(|| GpuFrameProfiler::new(&device, &queue));
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or_else(|| "surface has no compatible format".to_owned())?;
        let size = window.inner_size();
        let benchmark_uncapped = std::env::var_os("FLUXA_NATIVE_BENCH_UNCAPPED").is_some();
        let present_mode = if benchmark_uncapped
            && capabilities
                .present_modes
                .contains(&wgpu::PresentMode::Immediate)
        {
            wgpu::PresentMode::Immediate
        } else if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            // Mailbox replaces a queued frame instead of blocking
            // get_current_texture behind a FIFO queue. That keeps input/UI
            // responsive if the compositor misses a presentation deadline.
            wgpu::PresentMode::Mailbox
        } else if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::AutoVsync)
        {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::Fifo
        };
        if benchmark_uncapped && present_mode != wgpu::PresentMode::Immediate {
            eprintln!(
                "[fluxa-native] immediate present unsupported; throughput run falls back to FIFO"
            );
        }
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: capabilities
                .alpha_modes
                .iter()
                .copied()
                .find(|mode| *mode == wgpu::CompositeAlphaMode::PreMultiplied)
                .unwrap_or(capabilities.alpha_modes[0]),
            view_formats: vec![],
            // This is a small GUI renderer; one frame in flight minimizes
            // how long the UI can sit behind queued presentation work.
            desired_maximum_frame_latency: 1,
        };
        eprintln!(
            "[fluxa-native] surface present mode: {:?}",
            config.present_mode
        );
        surface.configure(&device, &config);

        let context = Context::default();
        // egui normally treats a HiDPI logical point as multiple physical
        // pixels. The Web renderer's CSS pixels are the design reference for
        // this desktop port, so neutralize the platform scale here. Without
        // this, a 590px Web top bar became ~800 physical pixels in Gamescope.
        let native_scale = window.scale_factor() as f32;
        context.set_zoom_factor((1.0 / native_scale.max(1.0)).clamp(0.5, 1.0));
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "archivo".to_owned(),
            egui::FontData::from_static(ARCHIVO_BYTES).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .splice(0..0, ["archivo"].map(str::to_owned));
        fonts
            .families
            .entry(egui::FontFamily::Name("archivo".into()))
            .or_default()
            .splice(0..0, ["archivo"].map(str::to_owned));
        let loaded_user_fonts = font_manager::FontManager::from_environment().load_into(&mut fonts);
        if loaded_user_fonts > 0 {
            eprintln!("[fluxa-native] loaded {loaded_user_fonts} user font(s)");
        }
        context.set_fonts(fonts);
        let state = EguiWinitState::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let mut renderer = EguiWgpuBackend::new(&device, format);
        let icons = SvgIconRegistry::new(&device, &queue, &mut renderer);
        let mut artwork = ArtworkRegistry::new();
        artwork.set_icon_textures(&icons);
        let scene_renderer = SceneRenderer::new(&device, format);
        let background_image = image::load_from_memory(BACKGROUND_BYTES)
            .map(|image| {
                let image = image.to_rgba8();
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                )
            })
            .unwrap_or_else(|_| egui::ColorImage::example());
        let background_texture = context.load_texture(
            "fluxa-shared-background",
            background_image,
            egui::TextureOptions::LINEAR,
        );
        let benchmark_scroll_started = std::env::var_os("FLUXA_NATIVE_BENCH_SCROLL")
            .is_some()
            .then(Instant::now);
        if benchmark_scroll_started.is_some() {
            eprintln!(
                "[fluxa-native] benchmark: automatic vertical scroll enabled (4s down / 4s up)"
            );
        }
        #[cfg(target_os = "linux")]
        let wayland_presentation = match WaylandPresentation::from_window(&window) {
            Ok(Some(presentation)) => {
                eprintln!("[fluxa-native] wp_presentation feedback enabled");
                Some(presentation)
            }
            Ok(None) => {
                eprintln!("[fluxa-native] wp_presentation is unavailable on this surface");
                None
            }
            Err(error) => {
                eprintln!("[fluxa-native] wp_presentation setup failed: {error}");
                None
            }
        };

        Ok(Self {
            context,
            state,
            instance,
            renderer,
            surface,
            device,
            queue,
            config,
            pending_resize: None,
            resize_reconfigure_until: None,
            artwork,
            icons,
            scene_renderer,
            scene: RenderScene::default(),
            home_model_cache: None,
            frame_diagnostics: FrameDiagnostics {
                target_frame_ms: benchmark_target_fps().map(|fps| 1_000.0 / f64::from(fps)),
                ..FrameDiagnostics::default()
            },
            gpu_profiler,
            benchmark_scroll_started,
            animations: UiAnimations::default(),
            background_texture,
            gamepad: Gilrs::new().map(Some).unwrap_or_else(|error| {
                eprintln!("[fluxa-native] gamepad input unavailable: {error}");
                None
            }),
            dev_reload: DevUiReload::new(),
            player: NativePlayer::default(),
            #[cfg(target_os = "linux")]
            wayland_presentation,
            #[cfg(target_os = "linux")]
            next_presentation_frame_id: 1,
            #[cfg(target_os = "linux")]
            presentation_error_logged: false,
        })
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Wayland can emit several alternating sizes while a toplevel enters
        // fullscreen or moves between workspaces. Reconfiguring wgpu for
        // each intermediate size leaves parent and Vulkan child surfaces on
        // different generations, which appears as horizontal bands.
        self.pending_resize = Some(size);
        self.resize_reconfigure_until = Some(Instant::now() + Duration::from_millis(250));
    }

    fn apply_pending_resize(&mut self) {
        let Some(deadline) = self.resize_reconfigure_until else {
            return;
        };
        if Instant::now() < deadline {
            return;
        }
        self.resize_reconfigure_until = None;
        let Some(size) = self.pending_resize.take() else {
            return;
        };
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn resize_pending(&self) -> bool {
        self.pending_resize.is_some() || self.resize_reconfigure_until.is_some()
    }

    fn event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    fn animations_active(&self) -> bool {
        self.animations.is_active() || self.artwork.has_active_animation()
    }

    #[cfg(target_os = "linux")]
    fn poll_wayland_presentation(&mut self) {
        let Some(presentation) = self.wayland_presentation.as_mut() else {
            return;
        };
        if let Err(error) = presentation.dispatch_pending()
            && !self.presentation_error_logged
        {
            eprintln!("[fluxa-native] wp_presentation dispatch failed: {error}");
            self.presentation_error_logged = true;
        }
        let events = presentation.drain_events().collect::<Vec<_>>();
        for event in events {
            self.frame_diagnostics.record_presentation(event);
        }
    }

    fn drain_gamepad_events(&mut self) -> Vec<egui::Event> {
        let Some(gamepad) = self.gamepad.as_mut() else {
            return Vec::new();
        };
        let mut events = Vec::new();
        while let Some(event) = gamepad.next_event() {
            let (button, pressed) = match event.event {
                GilrsEventType::ButtonPressed(button, _) => (button, true),
                GilrsEventType::ButtonRepeated(button, _) => (button, true),
                GilrsEventType::ButtonReleased(button, _) => (button, false),
                _ => continue,
            };
            let Some(key) = desktop_gamepad_key(button) else {
                continue;
            };
            events.push(egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        events
    }

    fn render(
        &mut self,
        window: &Window,
        focused_card: &mut usize,
        search: &mut String,
        status: &mut String,
        active_page: &mut String,
        screen_state: &mut NativeScreenState,
        runtime: &mut FluxaRuntime,
        effect_executor: &EffectExecutor,
        pending_effects: &mut Vec<Value>,
        pending_player_meta: &mut Option<Value>,
        player_started: &mut bool,
    ) {
        let frame_started = Instant::now();
        self.dev_reload.poll(&self.context);
        self.apply_pending_resize();
        self.artwork
            .poll(&self.context, &mut self.renderer, &self.device, &self.queue);
        self.artwork.begin_frame();
        self.player.sync(
            window,
            &self.instance,
            &self.device,
            runtime,
            effect_executor,
            *player_started || pending_player_meta.is_some(),
            self.config.width,
            self.config.height,
        );
        // mpv presents frames into the shared wgpu texture. Keep the parent
        // surface's normal redraw loop alive while the player is initializing
        // or playing; no child Wayland surface or second swapchain is used.
        if self.player.active_or_initializing() {
            window.request_redraw();
        }
        self.player
            .ensure_video_texture_id(&mut self.renderer, &self.device);
        if self.player.torrent_status_receiver.is_some() {
            self.context
                .request_repaint_after(Duration::from_millis(250));
        }
        let gpu_sample = self
            .gpu_profiler
            .as_mut()
            .is_some_and(GpuFrameProfiler::begin_frame);
        let gpu_timings = self
            .gpu_profiler
            .as_mut()
            .and_then(|profiler| profiler.poll(&self.device));
        let mut input = self.state.take_egui_input(window);
        if let Some(started) = self.benchmark_scroll_started {
            let phase = started.elapsed().as_secs_f64() % 8.0;
            let delta_y = if phase < 4.0 { -3.0 } else { 3.0 };
            input.events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, delta_y),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::NONE,
            });
        }
        // Feed physical desktop controllers into egui's normal focus graph.
        // The shared screens already use clickable/focusable responses, so
        // arrows and confirm/back behave identically to Android TV D-pad
        // events without adding controller-specific screen code.
        input.events.extend(self.drain_gamepad_events());
        // `egui-winit` refreshes the native monitor scale while taking input.
        // Apply the Web/CSS-pixel scale after that refresh; doing it only in
        // `NativeUi::new` is one pass too early and gets overwritten by the
        // first platform event.
        self.context
            .set_zoom_factor((1.0 / window.scale_factor() as f32).clamp(0.5, 1.0));
        self.animations.begin_frame();
        let mut ui_work = UiWorkStats::default();
        let run_ui_started = Instant::now();
        let mut output = self.context.run_ui(input, |ui| {
            draw_fluxa_ui(
                ui,
                window,
                focused_card,
                search,
                status,
                active_page,
                screen_state,
                runtime,
                pending_effects,
                pending_player_meta,
                player_started,
                &mut self.player,
                &mut self.artwork,
                self.background_texture.id(),
                &self.icons,
                &mut self.scene,
                &mut self.animations,
                &mut ui_work,
                &mut self.home_model_cache,
            )
        });
        if output.platform_output.cursor_icon == egui::CursorIcon::Default
            && self
                .context
                .interaction_snapshot(|snapshot| !snapshot.hovered.is_empty())
        {
            output.platform_output.cursor_icon = egui::CursorIcon::PointingHand;
        }
        let run_ui_ms = run_ui_started.elapsed().as_secs_f64() * 1_000.0;
        self.state
            .handle_platform_output(window, output.platform_output);
        let tessellate_started = Instant::now();
        let paint_jobs = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        let tessellate_ms = tessellate_started.elapsed().as_secs_f64() * 1_000.0;

        let (mut meshes, mut callbacks, mut vertices, mut indices) = (0, 0, 0, 0);
        let (mut draw_calls, mut texture_switches) = (0, 0);
        let mut previous_texture = None;
        for clipped in &paint_jobs {
            match &clipped.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    meshes += 1;
                    draw_calls += 1;
                    vertices += mesh.vertices.len();
                    indices += mesh.indices.len();
                    if previous_texture != Some(mesh.texture_id) {
                        texture_switches += 1;
                        previous_texture = Some(mesh.texture_id);
                    }
                }
                egui::epaint::Primitive::Callback(_) => {
                    callbacks += 1;
                    draw_calls += 1;
                    previous_texture = None;
                }
            }
        }
        let egui_upload_bytes = vertices * std::mem::size_of::<egui::epaint::Vertex>()
            + indices * std::mem::size_of::<u32>();
        let texture_upload_bytes = output
            .textures_delta
            .set
            .iter()
            .map(|(_, delta)| {
                delta.image.width() * delta.image.height() * delta.image.bytes_per_pixel()
            })
            .sum::<usize>();
        let texture_upload_started = Instant::now();
        self.renderer
            .apply_texture_deltas(&self.device, &self.queue, &output.textures_delta);
        let texture_upload_ms = texture_upload_started.elapsed().as_secs_f64() * 1_000.0;
        let descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: output.pixels_per_point,
        };
        let encoder_create_started = Instant::now();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fluxa-egui-encoder"),
            });
        let encoder_create_ms = encoder_create_started.elapsed().as_secs_f64() * 1_000.0;
        let egui_buffer_upload_started = Instant::now();
        self.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &descriptor,
        );
        let egui_buffer_upload_ms = egui_buffer_upload_started.elapsed().as_secs_f64() * 1_000.0;
        let surface_acquire_started = Instant::now();
        let surface_texture = self.surface.get_current_texture();
        let surface_acquire_ms = surface_acquire_started.elapsed().as_secs_f64() * 1_000.0;
        let frame = match surface_texture {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                // Do not immediately configure the parent with an
                // intermediate Wayland size. Keep it in the same debounced
                // resize path as normal Resized events so the parent and the
                // Vulkan subsurface switch generations together.
                self.pending_resize = Some(window.inner_size());
                self.resize_reconfigure_until = Some(Instant::now() + Duration::from_millis(250));
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Validation => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let scene_stats;
        {
            let clear_alpha = if self.player.active() {
                0.0
            } else {
                self.scene.clear_color.a as f64
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluxa-native-scene-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.scene.clear_color.r as f64,
                            g: self.scene.clear_color.g as f64,
                            b: self.scene.clear_color.b as f64,
                            a: clear_alpha,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: if gpu_sample {
                    self.gpu_profiler
                        .as_ref()
                        .map(|profiler| profiler.timestamp_writes(0, 1))
                } else {
                    None
                },
                occlusion_query_set: None,
                multiview_mask: None,
            });
            scene_stats = self.scene_renderer.render(
                &self.device,
                &self.queue,
                &mut pass,
                &self.scene,
                [self.config.width, self.config.height],
            );
        }
        let egui_encode_started = Instant::now();
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluxa-egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: if gpu_sample {
                    self.gpu_profiler
                        .as_ref()
                        .map(|profiler| profiler.timestamp_writes(2, 3))
                } else {
                    None
                },
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &descriptor);
        }
        let egui_encode_ms = egui_encode_started.elapsed().as_secs_f64() * 1_000.0;
        if gpu_sample && let Some(profiler) = self.gpu_profiler.as_ref() {
            profiler.resolve(&mut encoder);
        }
        let encoder_finish_started = Instant::now();
        let command_buffer = encoder.finish();
        let encoder_finish_ms = encoder_finish_started.elapsed().as_secs_f64() * 1_000.0;
        let submit_started = Instant::now();
        self.queue.submit(Some(command_buffer));
        let queue_submit_ms = submit_started.elapsed().as_secs_f64() * 1_000.0;
        if gpu_sample && let Some(profiler) = self.gpu_profiler.as_mut() {
            profiler.map_after_submit();
        }
        #[cfg(target_os = "linux")]
        let mut presentation_frame_id = None;
        #[cfg(target_os = "linux")]
        if let Some(presentation) = self.wayland_presentation.as_mut() {
            let frame_id = self.next_presentation_frame_id;
            self.next_presentation_frame_id = self.next_presentation_frame_id.wrapping_add(1);
            match presentation.request_feedback(frame_id) {
                Ok(()) => presentation_frame_id = Some(frame_id),
                Err(error) if !self.presentation_error_logged => {
                    eprintln!("[fluxa-native] wp_presentation request failed: {error}");
                    self.presentation_error_logged = true;
                }
                Err(_) => {}
            }
        }
        let present_started = Instant::now();
        frame.present();
        let present_ms = present_started.elapsed().as_secs_f64() * 1_000.0;
        self.renderer.free_texture_deltas(&output.textures_delta);
        self.frame_diagnostics.record(
            FrameSample {
                cpu_frame_ms: frame_started.elapsed().as_secs_f64() * 1_000.0,
                frame_started_at: frame_started,
                run_ui_ms,
                snapshot_share_ms: ui_work.snapshot_share_ms,
                home_inputs_ms: ui_work.home_inputs_ms,
                home_model_ms: ui_work.home_model_ms,
                home_draw_ms: ui_work.home_draw_ms,
                tessellate_ms,
                texture_upload_ms,
                egui_buffer_upload_ms,
                scene: scene_stats,
                egui_encode_ms,
                surface_acquire_ms,
                encoder_create_ms,
                encoder_finish_ms,
                queue_submit_ms,
                present_ms,
                meshes,
                callbacks,
                vertices,
                indices,
                draw_calls,
                texture_switches,
                egui_upload_bytes,
                texture_upload_bytes,
                player_active: self.player.active(),
                home_model_cache_hit: ui_work.home_model_cache_hit,
                #[cfg(target_os = "linux")]
                presentation_frame_id,
            },
            gpu_timings,
        );
    }
}

fn draw_native_player_overlay(
    ui: &mut egui::Ui,
    window: &Window,
    player: &mut NativePlayer,
    title: &str,
) -> bool {
    let ctx = ui.ctx().clone();
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    let now = Instant::now();

    if now.duration_since(player.overlay_last_status_poll) >= Duration::from_millis(220) {
        #[cfg(target_os = "linux")]
        if let Some(native) = player.gpu.as_ref() {
            let position = native.client.fast_position_status();
            let status = native.client.cached_static_status();
            player.overlay_position = position
                .time_pos
                .as_deref()
                .and_then(|value| value.parse().ok())
                .unwrap_or(player.overlay_position);
            player.overlay_duration = status
                .duration
                .as_deref()
                .and_then(|value| value.parse().ok())
                .unwrap_or(player.overlay_duration);
            player.overlay_paused = status.pause.as_deref() == Some("yes");
            player.overlay_muted = status.mute.as_deref() == Some("yes");
            player.overlay_volume = status
                .volume
                .as_deref()
                .and_then(|value| value.parse().ok())
                .unwrap_or(player.overlay_volume);
        }
        player.overlay_last_status_poll = now;
    }

    let pointer = ctx.input(|input| input.pointer.hover_pos());
    let pointer_moved = match (pointer, player.overlay_pointer) {
        (Some(current), Some(previous)) => current.distance(previous) > 0.5,
        (Some(_), None) => true,
        _ => false,
    };
    player.overlay_pointer = pointer;
    if pointer_moved || ctx.input(|input| input.pointer.any_pressed()) {
        player.overlay_visible = true;
        player.overlay_last_activity = now;
    } else if !player.overlay_paused
        && now.duration_since(player.overlay_last_activity) > Duration::from_secs(4)
        && !player.overlay_scrubbing
    {
        player.overlay_visible = false;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::Space)) {
        #[cfg(target_os = "linux")]
        if let Some(native) = player.gpu.as_ref() {
            let _ = native.client.command(&["cycle", "pause"]);
        }
        player.overlay_visible = true;
        player.overlay_last_activity = now;
    }
    for (key, delta) in [(egui::Key::ArrowLeft, -10.0), (egui::Key::ArrowRight, 10.0)] {
        if ctx.input(|input| input.key_pressed(key)) {
            #[cfg(target_os = "linux")]
            if let Some(native) = player.gpu.as_ref() {
                let _ = native
                    .client
                    .command_string(&format!("seek {delta} relative"));
            }
            player.overlay_position =
                (player.overlay_position + delta).clamp(0.0, player.overlay_duration.max(0.0));
            player.overlay_visible = true;
            player.overlay_last_activity = now;
        }
    }

    if let Some(texture_id) = player.video_texture_id {
        painter.image(texture_id, rect, full_uv(), Color32::WHITE);
    } else {
        painter.rect_filled(rect, 0.0, Color32::BLACK);
    }

    if !player.overlay_visible {
        return false;
    }

    // Soft top/bottom scrims mirror the web player's controls over the video
    // without putting a solid panel across the picture.
    let scrim_band = 18.0;
    for index in 0..16 {
        let top_alpha = (152.0 * (1.0 - index as f32 / 16.0).powf(1.7)).round() as u8;
        let bottom_alpha = (218.0 * (1.0 - index as f32 / 16.0).powf(1.45)).round() as u8;
        let top = rect.top() + index as f32 * scrim_band;
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), top),
                egui::pos2(rect.right(), (top + scrim_band).min(rect.bottom())),
            ),
            0.0,
            Color32::from_black_alpha(top_alpha),
        );
        let bottom = rect.bottom() - index as f32 * scrim_band;
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), (bottom - scrim_band).max(rect.top())),
                egui::pos2(rect.right(), bottom),
            ),
            0.0,
            Color32::from_black_alpha(bottom_alpha),
        );
    }

    let horizontal_margin = (rect.width() * 0.035).clamp(22.0, 64.0);
    let header_y = rect.top() + 30.0;
    let close_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + horizontal_margin + 20.0, header_y),
        Vec2::splat(42.0),
    );
    let close = player_overlay_button(ui, &painter, close_rect, "close", false);
    if close.clicked() {
        return true;
    }
    painter.text(
        egui::pos2(close_rect.right() + 10.0, header_y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(19.0),
        Color32::WHITE,
    );

    let bar_x = rect.left() + horizontal_margin;
    let bar_width = (rect.width() - horizontal_margin * 2.0).max(80.0);
    let seek_track = egui::Rect::from_min_size(
        egui::pos2(bar_x, rect.bottom() - 93.0),
        Vec2::new(bar_width, 16.0),
    );
    let seek_response = ui.interact(
        seek_track.expand2(Vec2::new(0.0, 8.0)),
        egui::Id::new("native-player-seek"),
        Sense::click_and_drag(),
    );
    let duration = player.overlay_duration.max(0.0);
    let update_seek = |player: &mut NativePlayer, position: egui::Pos2| {
        let ratio = ((position.x - seek_track.left()) / seek_track.width()).clamp(0.0, 1.0);
        let target = duration * ratio as f64;
        player.overlay_position = target;
        player.overlay_scrubbing = seek_response.dragged();
        if seek_response.clicked()
            || (seek_response.dragged()
                && now.duration_since(player.overlay_last_seek) >= Duration::from_millis(120))
        {
            #[cfg(target_os = "linux")]
            if let Some(native) = player.gpu.as_ref() {
                let _ = native
                    .client
                    .command_string(&format!("seek {target:.3} absolute"));
            }
            player.overlay_last_seek = now;
        }
    };
    if (seek_response.clicked() || seek_response.dragged())
        && let Some(position) = seek_response.interact_pointer_pos()
    {
        update_seek(player, position);
        player.overlay_last_activity = now;
    } else if !seek_response.dragged() {
        player.overlay_scrubbing = false;
    }

    painter.rect_filled(
        egui::Rect::from_center_size(seek_track.center(), Vec2::new(seek_track.width(), 3.0)),
        2.0,
        Color32::from_white_alpha(95),
    );
    if duration > 0.0 {
        let progress = (player.overlay_position / duration).clamp(0.0, 1.0) as f32;
        let played_width = seek_track.width() * progress;
        painter.rect_filled(
            egui::Rect::from_center_size(
                egui::pos2(
                    seek_track.left() + played_width * 0.5,
                    seek_track.center().y,
                ),
                Vec2::new(played_width.max(0.0), 3.5),
            ),
            2.0,
            Color32::from_rgb(232, 93, 63),
        );
        if seek_response.hovered() || seek_response.dragged() {
            painter.circle_filled(
                egui::pos2(seek_track.left() + played_width, seek_track.center().y),
                6.0,
                Color32::from_rgb(245, 245, 245),
            );
        }
    }

    let controls_y = rect.bottom() - 48.0;
    let button_size = 42.0;
    let mut x = bar_x + 1.0;
    let play_rect =
        egui::Rect::from_center_size(egui::pos2(x + 22.0, controls_y), Vec2::splat(48.0));
    if player_overlay_button(
        ui,
        &painter,
        play_rect,
        if player.overlay_paused {
            "play"
        } else {
            "pause"
        },
        true,
    )
    .clicked()
    {
        #[cfg(target_os = "linux")]
        if let Some(native) = player.gpu.as_ref() {
            let _ = native.client.command(&["cycle", "pause"]);
        }
        player.overlay_paused = !player.overlay_paused;
        player.overlay_last_activity = now;
    }
    x += 62.0;
    for (icon, seconds) in [("back", -10.0), ("forward", 10.0)] {
        let button_rect = egui::Rect::from_center_size(
            egui::pos2(x + button_size * 0.5, controls_y),
            Vec2::splat(button_size),
        );
        if player_overlay_button(ui, &painter, button_rect, icon, false).clicked() {
            #[cfg(target_os = "linux")]
            if let Some(native) = player.gpu.as_ref() {
                let _ = native
                    .client
                    .command_string(&format!("seek {seconds} relative"));
            }
            player.overlay_position =
                (player.overlay_position + seconds).clamp(0.0, duration.max(0.0));
            player.overlay_last_activity = now;
        }
        x += 48.0;
    }
    let time_text = format!(
        "{}  /  {}",
        format_player_time(player.overlay_position),
        format_player_time(duration)
    );
    painter.text(
        egui::pos2(x + 2.0, controls_y),
        Align2::LEFT_CENTER,
        time_text,
        FontId::proportional(13.0),
        Color32::from_white_alpha(218),
    );

    let mut right_x = rect.right() - horizontal_margin - 20.0;
    let fullscreen_rect =
        egui::Rect::from_center_size(egui::pos2(right_x, controls_y), Vec2::splat(42.0));
    if player_overlay_button(ui, &painter, fullscreen_rect, "fullscreen", false).clicked() {
        if window.fullscreen().is_some() {
            window.set_fullscreen(None);
        } else {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        player.overlay_last_activity = now;
    }
    right_x -= 52.0;
    let volume_rect =
        egui::Rect::from_center_size(egui::pos2(right_x, controls_y), Vec2::splat(42.0));
    if player_overlay_button(
        ui,
        &painter,
        volume_rect,
        if player.overlay_muted {
            "mute"
        } else {
            "volume"
        },
        false,
    )
    .clicked()
    {
        #[cfg(target_os = "linux")]
        if let Some(native) = player.gpu.as_ref() {
            let _ = native.client.command(&["cycle", "mute"]);
        }
        player.overlay_muted = !player.overlay_muted;
        player.overlay_last_activity = now;
    }
    let volume_label = if player.overlay_muted {
        "Muted".to_owned()
    } else {
        format!("{}%", player.overlay_volume.round() as i32)
    };
    painter.text(
        egui::pos2(right_x - 18.0, controls_y),
        Align2::RIGHT_CENTER,
        volume_label,
        FontId::proportional(12.0),
        Color32::from_white_alpha(170),
    );
    ctx.request_repaint_after(Duration::from_millis(100));
    false
}

fn player_overlay_button(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    icon: &str,
    prominent: bool,
) -> egui::Response {
    let response = ui.interact(
        rect,
        egui::Id::new(("native-player-button", icon, rect.min.x.to_bits())),
        Sense::click(),
    );
    if prominent || response.hovered() || response.is_pointer_button_down_on() {
        painter.circle_filled(
            rect.center(),
            rect.width() * 0.5,
            if prominent {
                Color32::from_white_alpha(30)
            } else {
                Color32::from_black_alpha(75)
            },
        );
    }
    let color = if response.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(235)
    };
    let c = rect.center();
    match icon {
        "play" => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + Vec2::new(-5.0, -8.0),
                    c + Vec2::new(8.0, 0.0),
                    c + Vec2::new(-5.0, 8.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
        }
        "pause" => {
            painter.rect_filled(
                egui::Rect::from_center_size(c + Vec2::new(-4.0, 0.0), Vec2::new(4.0, 17.0)),
                1.0,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(c + Vec2::new(4.0, 0.0), Vec2::new(4.0, 17.0)),
                1.0,
                color,
            );
        }
        "close" => {
            painter.line_segment(
                [c + Vec2::new(-6.0, -6.0), c + Vec2::new(6.0, 6.0)],
                egui::Stroke::new(2.0, color),
            );
            painter.line_segment(
                [c + Vec2::new(6.0, -6.0), c + Vec2::new(-6.0, 6.0)],
                egui::Stroke::new(2.0, color),
            );
        }
        "back" | "forward" => {
            let sign = if icon == "back" { -1.0 } else { 1.0 };
            let center = c + Vec2::new(-4.0 * sign, -2.0);
            painter.add(egui::Shape::line(
                (0..=20)
                    .map(|step| {
                        let angle = 0.25 + (5.2 - 0.25) * step as f32 / 20.0;
                        center + Vec2::new(angle.cos() * 8.0, angle.sin() * 8.0)
                    })
                    .collect(),
                egui::Stroke::new(1.8, color),
            ));
            painter.text(
                c + Vec2::new(0.0, 7.0),
                Align2::CENTER_CENTER,
                "10",
                FontId::proportional(9.0),
                color,
            );
        }
        "volume" | "mute" => {
            painter.rect_filled(
                egui::Rect::from_min_max(c + Vec2::new(-9.0, -4.0), c + Vec2::new(-5.0, 4.0)),
                0.5,
                color,
            );
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + Vec2::new(-5.0, -5.0),
                    c + Vec2::new(2.0, -10.0),
                    c + Vec2::new(2.0, 10.0),
                    c + Vec2::new(-5.0, 5.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
            if icon == "mute" {
                painter.line_segment(
                    [c + Vec2::new(5.0, -5.0), c + Vec2::new(11.0, 5.0)],
                    egui::Stroke::new(1.8, color),
                );
                painter.line_segment(
                    [c + Vec2::new(11.0, -5.0), c + Vec2::new(5.0, 5.0)],
                    egui::Stroke::new(1.8, color),
                );
            } else {
                painter.add(egui::Shape::line(
                    (0..=10)
                        .map(|step| {
                            let angle = -0.75 + 1.5 * step as f32 / 10.0;
                            c + Vec2::new(angle.cos() * 10.0, angle.sin() * 10.0)
                        })
                        .collect(),
                    egui::Stroke::new(1.5, color),
                ));
            }
        }
        "fullscreen" => {
            let s = egui::Stroke::new(2.0, color);
            for (a, b) in [
                ((-8.0, -3.0), (-8.0, -8.0)),
                ((-8.0, -8.0), (-3.0, -8.0)),
                ((8.0, -3.0), (8.0, -8.0)),
                ((8.0, -8.0), (3.0, -8.0)),
                ((-8.0, 3.0), (-8.0, 8.0)),
                ((-8.0, 8.0), (-3.0, 8.0)),
                ((8.0, 3.0), (8.0, 8.0)),
                ((8.0, 8.0), (3.0, 8.0)),
            ] {
                painter.line_segment([c + Vec2::new(a.0, a.1), c + Vec2::new(b.0, b.1)], s);
            }
        }
        _ => {}
    }
    response
}

fn format_player_time(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    let hours = total / 3600;
    let minutes = (total / 60) % 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn draw_fluxa_ui(
    ui: &mut egui::Ui,
    window: &Window,
    focused_card: &mut usize,
    search: &mut String,
    status: &mut String,
    active_page: &mut String,
    screen_state: &mut NativeScreenState,
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
    pending_player_meta: &mut Option<Value>,
    player_started: &mut bool,
    player: &mut NativePlayer,
    artwork: &mut ArtworkRegistry,
    background_texture: TextureId,
    icons: &SvgIconRegistry,
    scene: &mut RenderScene,
    animations: &mut UiAnimations,
    ui_work: &mut UiWorkStats,
    home_model_cache: &mut Option<CachedHomeModel>,
) {
    let ctx = ui.ctx().clone();
    let snapshot_share_started = Instant::now();
    let snapshot = runtime.snapshot_shared();
    ui_work.snapshot_share_ms = snapshot_share_started.elapsed().as_secs_f64() * 1_000.0;
    artwork.set_active_profile(
        snapshot
            .pointer("/profile/active")
            .or_else(|| snapshot.pointer("/home/activeProfile")),
    );
    artwork.set_accent_from_snapshot(&snapshot);
    // Player and loading branches return before the normal page scene setup.
    // Clear the previous page's opaque nodes so the native video subsurface
    // can show through the transparent wgpu parent surface.
    scene.clear();
    if player.active() {
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            player.stop();
            *player_started = false;
            *pending_player_meta = None;
            return;
        }
        let title = pending_player_meta
            .as_ref()
            .and_then(|meta| meta.get("name").or_else(|| meta.get("title")))
            .and_then(Value::as_str)
            .or_else(|| {
                snapshot
                    .pointer("/player/currentVideoId")
                    .and_then(Value::as_str)
            })
            .unwrap_or("Fluxa Player");
        if draw_native_player_overlay(ui, window, player, title) {
            player.stop();
            *player_started = false;
            *pending_player_meta = None;
        }
        return;
    }
    if *player_started || pending_player_meta.is_some() {
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            player.stop();
            if let Some(url) = snapshot
                .pointer("/player/resolvedUrl")
                .and_then(Value::as_str)
            {
                player.blocked_url = Some(url.to_owned());
            }
            *player_started = false;
            *pending_player_meta = None;
            return;
        }
        let rect = ui.max_rect();
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_rgb(8, 8, 10));
        let title = pending_player_meta
            .as_ref()
            .and_then(|meta| meta.get("name").or_else(|| meta.get("title")))
            .and_then(Value::as_str)
            .or_else(|| {
                snapshot
                    .pointer("/player/currentVideoId")
                    .and_then(Value::as_str)
            })
            .unwrap_or("Fluxa Player");
        let error = player.error.as_deref().or_else(|| {
            snapshot
                .pointer("/player/playerError")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
        });
        ui.painter().text(
            rect.center() - Vec2::new(0.0, 28.0),
            Align2::CENTER_CENTER,
            title,
            FontId::proportional(26.0),
            Color32::WHITE,
        );
        let torrent_status = torrent_loading_details(player.torrent_status.as_ref());
        ui.painter().text(
            rect.center() + Vec2::new(0.0, 14.0),
            Align2::CENTER_CENTER,
            error.unwrap_or_else(|| {
                if player.torrent_link.is_some() {
                    torrent_status.0.as_str()
                } else if player.active_or_initializing() {
                    "Waiting for the first video frame…"
                } else if status.is_empty() {
                    "Preparing native player…"
                } else {
                    status.as_str()
                }
            }),
            FontId::proportional(15.0),
            if error.is_some() {
                Color32::from_rgb(255, 130, 110)
            } else {
                Color32::from_white_alpha(180)
            },
        );
        ui.painter().text(
            rect.center() + Vec2::new(0.0, 42.0),
            Align2::CENTER_CENTER,
            if player.torrent_link.is_some() {
                torrent_status.1.as_str()
            } else {
                "Waiting for the first video frame"
            },
            FontId::proportional(13.0),
            Color32::from_white_alpha(145),
        );
        ui.painter().text(
            rect.center() + Vec2::new(0.0, 68.0),
            Align2::CENTER_CENTER,
            "Esc to cancel",
            FontId::proportional(13.0),
            Color32::from_white_alpha(120),
        );
        return;
    }
    ctx.set_visuals(egui::Visuals::dark());
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = Vec2::new(12.0, 12.0);
        style.visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, Color32::from_white_alpha(18));
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(28, 29, 36);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(49, 50, 60);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(232, 93, 63);
    });
    let mut navigate = None;
    let top_bar_width = 590.0;
    let top_bar_height = 60.0;
    let top_bar_x = ((ctx.content_rect().width() - top_bar_width) * 0.5).max(8.0);
    let shared_page = matches!(
        active_page.as_str(),
        "Home" | "Library" | "Discover" | "Calendar" | "Settings" | "Detail"
    );
    if active_page != "Home" && !shared_page {
        egui::Area::new("fluxa-top-navigation".into())
            .fixed_pos(egui::pos2(top_bar_x, 6.0))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(15, 15, 16, 245))
                    .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(18)))
                    .corner_radius(17)
                    .inner_margin(6)
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(top_bar_width - 12.0, top_bar_height - 12.0));
                        ui.horizontal(|ui| {
                            for (label, width) in [
                                ("Home", 104.0),
                                ("Library", 104.0),
                                ("Discover", 112.0),
                                ("Calendar", 112.0),
                            ] {
                                let selected = active_page == label;
                                let fill = if selected {
                                    Color32::from_white_alpha(36)
                                } else {
                                    Color32::TRANSPARENT
                                };
                                let text_color = if selected {
                                    Color32::WHITE
                                } else {
                                    Color32::from_white_alpha(170)
                                };
                                let (nav_rect, response) =
                                    ui.allocate_exact_size(Vec2::new(width, 48.0), Sense::click());
                                ui.painter().rect_filled(nav_rect, 24.0, fill);
                                icons.paint(
                                    ui.painter(),
                                    nav_rect.left_center() + Vec2::new(20.0, 0.0),
                                    label,
                                    text_color,
                                );
                                ui.painter().text(
                                    nav_rect.left_center() + Vec2::new(39.0, 0.0),
                                    Align2::LEFT_CENTER,
                                    label,
                                    FontId::proportional(14.0),
                                    text_color,
                                );
                                if response.clicked() {
                                    *active_page = label.to_owned();
                                    if let Ok(update) = runtime.dispatch(json!({
                                        "type": "navigationRequested",
                                        "route": label.to_lowercase(),
                                        "params": null,
                                    })) {
                                        pending_effects.extend(update.effects);
                                    }
                                    match label {
                                        "Library" => {
                                            if let Ok(update) = runtime.dispatch(json!({
                                                "type": "libraryHydrateRequested",
                                                "profileId": null,
                                            })) {
                                                pending_effects.extend(update.effects);
                                            }
                                        }
                                        "Discover" => {
                                            if let Ok(update) = runtime.dispatch(json!({
                                                "type": "discoverRequested",
                                                "loadCatalogFilters": true,
                                                "contentType": screen_state.discover_content_type,
                                                "filters": { "catalogKey": null, "extra": {} },
                                                "language": "en",
                                            })) {
                                                pending_effects.extend(update.effects);
                                            }
                                        }
                                        "Calendar" => {
                                            dispatch_calendar_month(
                                                runtime,
                                                pending_effects,
                                                screen_state.calendar_year,
                                                screen_state.calendar_month,
                                            );
                                        }
                                        _ => {}
                                    }
                                    navigate = Some(label.to_owned());
                                }
                            }
                            ui.add_space(2.0);
                            ui.separator();
                            ui.add_space(2.0);
                            let (profile_rect, profile_response) =
                                ui.allocate_exact_size(Vec2::new(64.0, 48.0), Sense::click());
                            ui.painter().circle_filled(
                                profile_rect.center(),
                                15.0,
                                Color32::from_rgb(106, 166, 54),
                            );
                            ui.painter().text(
                                profile_rect.center(),
                                Align2::CENTER_CENTER,
                                "ok",
                                FontId::proportional(12.0),
                                Color32::WHITE,
                            );
                            let profile = profile_response;
                            if profile.clicked() {
                                *active_page = "Settings".to_owned();
                                if let Ok(update) = runtime.dispatch(json!({
                                    "type": "navigationRequested",
                                    "route": "settings",
                                    "params": null,
                                })) {
                                    pending_effects.extend(update.effects);
                                }
                                navigate = Some("Settings".to_owned());
                            }
                        });
                    });
            });
    }

    let rect = ui.max_rect();
    scene.clear();
    scene.clear_color = SceneColor::rgb(0.024, 0.024, 0.028);
    scene.push(RenderNode {
        bounds: SceneRect::new(rect.left(), rect.top(), rect.width(), rect.height()),
        fill: SceneColor::rgb(0.024, 0.024, 0.028),
        accent: SceneColor::rgb(0.024, 0.024, 0.028),
        radius: 0.0,
        border_width: 0.0,
        opacity: 1.0,
        focus: 0.0,
        layer: 0,
    });
    scene.sort();
    let _ = search;
    ui.add_space(4.0);
    let page_opacity = animations.page_opacity(active_page);
    // The shared Rust renderer owns document scrolling. Keeping an egui
    // ScrollArea around it would create a second scroll model: wheel/drag
    // deltas could be consumed by the host container while the fixed-position
    // shared shelves remain at their own offset.
    ui.set_opacity(page_opacity);
    ui.add_space(8.0);
    if active_page != "Home" {
        match active_page.as_str() {
            "Library" => draw_shared_library_screen(
                ui,
                runtime,
                artwork,
                background_texture,
                active_page,
                screen_state,
                pending_effects,
                status,
            ),
            "Discover" => draw_shared_discover_screen(
                ui,
                runtime,
                artwork,
                background_texture,
                active_page,
                screen_state,
                pending_effects,
                status,
            ),
            "Calendar" => draw_shared_calendar_screen(
                ui,
                runtime,
                artwork,
                background_texture,
                active_page,
                screen_state,
                pending_effects,
                status,
            ),
            "Settings" => draw_shared_settings_screen(
                ui,
                runtime,
                artwork,
                background_texture,
                active_page,
                screen_state,
                pending_effects,
                status,
            ),
            "Detail" => draw_shared_detail_screen(
                ui,
                runtime,
                artwork,
                background_texture,
                active_page,
                pending_effects,
                status,
            ),
            _ => {
                ui.label(
                    RichText::new("This section is not available yet.")
                        .color(Color32::from_white_alpha(170)),
                );
            }
        };
    } else {
        // Home is rendered by the same responsive Rust UI component
        // as Android. The desktop host only adapts Core's JSON and
        // supplies the native artwork registry.
        let revision = runtime.revision();
        if !home_model_cache
            .as_ref()
            .is_some_and(|cached| cached.revision == revision)
        {
            let home_inputs_started = Instant::now();
            let empty_home = Value::Null;
            let empty_slides = Value::Array(Vec::new());
            let home = snapshot.get("home").unwrap_or(&empty_home);
            // Core plans are immutable until the runtime revision changes.
            // Build the shared model and its dependent hero data together so
            // a scroll redraw does not clone the full catalog payload.
            let hero_plan = runtime.home_hero_plan();
            let billboard = hero_plan
                .and_then(|plan| plan.get("billboard"))
                // A missing selected-catalog result means “no hero item”.
                // Falling back to home.billboard here can resurrect the
                // bootstrap's first shelf (including a personal collection
                // collage), which is explicitly not a hero catalog.
                .unwrap_or(&empty_home);
            let hero_slides = hero_plan
                .and_then(|plan| plan.get("slides"))
                .unwrap_or(&empty_slides);
            ui_work.home_inputs_ms = home_inputs_started.elapsed().as_secs_f64() * 1_000.0;
            let home_model_started = Instant::now();
            let model = shared_home_model(home, billboard, hero_slides);
            ui_work.home_model_ms = home_model_started.elapsed().as_secs_f64() * 1_000.0;
            *home_model_cache = Some(CachedHomeModel {
                revision,
                model,
                billboard: billboard.clone(),
            });
        } else {
            ui_work.home_model_cache_hit = true;
        }
        let cached_home = home_model_cache
            .as_ref()
            .expect("Home model cache populated for the current Core revision");
        let shared_home = &cached_home.model;
        let billboard = &cached_home.billboard;
        let mut shared_assets = DesktopHomeAssets {
            background: background_texture,
            artwork,
        };
        let home_draw_started = Instant::now();
        let shared_layout = draw_home(
            &ctx,
            Viewport::new(
                ctx.content_rect().width() as u32,
                ctx.content_rect().height() as u32,
                UiFormFactor::Desktop,
            ),
            shared_home,
            &mut shared_assets,
            // Desktop pointer navigation must not paint a permanent keyboard
            // focus ring around the first shelf card on initial render.
            None,
        );
        for request in &shared_layout.load_more {
            if let Ok(update) = runtime.dispatch(request.clone()) {
                pending_effects.extend(update.effects);
            }
        }
        ui_work.home_draw_ms = home_draw_started.elapsed().as_secs_f64() * 1_000.0;
        if let Some(activated) = shared_layout.activated {
            match activated {
                fluxa_ui::NODE_HOME
                | fluxa_ui::NODE_LIBRARY
                | fluxa_ui::NODE_DISCOVER
                | fluxa_ui::NODE_CALENDAR
                | fluxa_ui::NODE_PROFILE => {
                    let (destination, route) = match activated {
                        fluxa_ui::NODE_HOME => ("Home", "home"),
                        fluxa_ui::NODE_LIBRARY => ("Library", "library"),
                        fluxa_ui::NODE_DISCOVER => ("Discover", "discover"),
                        fluxa_ui::NODE_CALENDAR => ("Calendar", "calendar"),
                        _ => ("Settings", "settings"),
                    };
                    enter_shared_desktop_page(
                        destination,
                        route,
                        active_page,
                        runtime,
                        pending_effects,
                    );
                    navigate = Some(destination.to_owned());
                }
                fluxa_ui::NODE_PLAY | fluxa_ui::NODE_MORE_INFO => {
                    let active_hero = fluxa_ui::active_home_hero(&ctx, shared_home);
                    if let (Some(id), Some(content_type)) = (
                        active_hero
                            .and_then(|hero| hero.item_id.clone())
                            .or_else(|| value_string(&billboard, "id")),
                        active_hero
                            .and_then(|hero| hero.item_type.clone())
                            .or_else(|| value_string(&billboard, "type")),
                    ) {
                        *active_page = "Detail".to_owned();
                        if let Ok(update) = runtime.dispatch(json!({
                            "type": "detailLoadRequested",
                            "id": id,
                            "contentType": content_type,
                            "language": "en",
                            "profile": null,
                        })) {
                            pending_effects.extend(update.effects);
                        }
                    }
                }
                node if node >= fluxa_ui::NODE_CARD_BASE => {
                    *focused_card = (node - fluxa_ui::NODE_CARD_BASE) as usize;
                    if let Some(item) = shared_home.card_at(*focused_card) {
                        if let (Some(id), Some(content_type)) =
                            (item.id.clone(), item.item_type.clone())
                        {
                            if item.row_kind == fluxa_ui::HomeRowKind::Continue {
                                // Continue Watching is a resume action, not a
                                // detail navigation. Preserve the complete
                                // Core item so the player receives the last
                                // video/episode identity and resume position.
                                let meta = if item.raw.is_object() {
                                    item.raw.clone()
                                } else {
                                    json!({
                                        "id": id,
                                        "type": content_type,
                                        "name": item.title.clone(),
                                    })
                                };
                                player.blocked_url = None;
                                player.error = None;
                                *pending_player_meta = Some(meta.clone());
                                *player_started = false;
                                *status = "Preparing player…".to_owned();
                                if let Ok(update) = runtime.dispatch(json!({
                                    "type": "continueWatchingPlaybackRequested",
                                    "item": meta,
                                    "language": "en",
                                    "profile": null,
                                })) {
                                    pending_effects.extend(update.effects);
                                }
                            } else {
                                *active_page = "Detail".to_owned();
                                if let Ok(update) = runtime.dispatch(json!({
                                    "type": "detailLoadRequested",
                                    "id": id,
                                    "contentType": content_type,
                                    "language": "en",
                                    "profile": null,
                                })) {
                                    pending_effects.extend(update.effects);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let scroll_max = desktop_screen_scroll_max(
        active_page,
        runtime,
        screen_state,
        home_model_cache.as_ref().map(|cached| &cached.model),
        ctx.content_rect().size(),
    );
    draw_desktop_vertical_scrollbar(ui, active_page, scroll_max);

    if let Some(destination) = navigate {
        *status = format!("{destination} opened");
    }
}

fn desktop_screen_scroll_max(
    active_page: &str,
    runtime: &FluxaRuntime,
    screen_state: &NativeScreenState,
    home: Option<&SharedHomeModel>,
    size: Vec2,
) -> f32 {
    let viewport = Viewport::new(
        size.x.max(1.0) as u32,
        size.y.max(1.0) as u32,
        UiFormFactor::Desktop,
    );
    let snapshot = runtime.snapshot();
    match active_page {
        "Home" => home
            .map(|home| fluxa_ui::home_scroll_max(viewport, home))
            .unwrap_or_default(),
        "Library" => {
            let tab = match screen_state.library_tab.as_str() {
                "Watching" => fluxa_ui::LibraryTab::Watching,
                "Completed" => fluxa_ui::LibraryTab::Completed,
                "Dropped" => fluxa_ui::LibraryTab::Dropped,
                "Favorites" | "Liked" => fluxa_ui::LibraryTab::Liked,
                "Airing" => fluxa_ui::LibraryTab::Airing,
                "Rated" => fluxa_ui::LibraryTab::Rated,
                "History" => fluxa_ui::LibraryTab::History,
                _ => fluxa_ui::LibraryTab::Watchlist,
            };
            let mut model = screen_state
                .library_model_cache
                .as_ref()
                .map(|(_, model)| model.clone())
                .unwrap_or_else(|| fluxa_ui::library_model_from_core_snapshot(&snapshot));
            model.query.clone_from(&screen_state.library_query);
            model.sort_by.clone_from(&screen_state.library_sort);
            fluxa_ui::library_scroll_max(viewport, &model, tab)
        }
        "Discover" => {
            if let Some((_, model)) = screen_state.discover_model_cache.as_ref() {
                fluxa_ui::discover_scroll_max(viewport, model)
            } else {
                let result_count = snapshot
                    .pointer("/discover/results")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len);
                fluxa_ui::discover_scroll_max_for_result_count(viewport, result_count)
            }
        }
        "Calendar" => {
            let mut model = fluxa_ui::calendar_model_from_core_snapshot(&snapshot);
            model.selected_day = screen_state.calendar_selected_day;
            fluxa_ui::calendar_scroll_max(viewport, &model)
        }
        "Settings" => {
            let mut model = settings_model_from_core_snapshot(&snapshot);
            model.active_section = screen_state.settings_section;
            fluxa_ui::settings_scroll_max(viewport, &model)
        }
        "Detail" => {
            let model = detail_model_from_core_snapshot(&snapshot);
            fluxa_ui::detail_scroll_max(viewport, &model)
        }
        _ => 0.0,
    }
}

fn draw_desktop_vertical_scrollbar(ui: &mut egui::Ui, active_page: &str, max_offset: f32) {
    if max_offset <= 0.0 {
        return;
    }
    let Some(scroll_id) = desktop_scroll_id(active_page) else {
        return;
    };

    let context = ui.ctx();
    let screen = context.content_rect();
    let track = egui::Rect::from_min_max(
        egui::pos2(screen.right() - 10.0, screen.top() + 10.0),
        egui::pos2(screen.right() - 4.0, screen.bottom() - 10.0),
    );
    if track.height() <= 0.0 {
        return;
    }

    let id = egui::Id::new(scroll_id);
    let offset = context.data_mut(|data| {
        data.get_temp::<f32>(id)
            .unwrap_or(0.0)
            .clamp(0.0, max_offset)
    });
    let thumb_height = (track.height() * track.height() / (track.height() + max_offset))
        .clamp(34.0, track.height());
    let thumb_travel = (track.height() - thumb_height).max(0.0);
    let thumb_top = track.top() + thumb_travel * (offset / max_offset);
    let thumb = egui::Rect::from_min_size(
        egui::pos2(track.left(), thumb_top),
        Vec2::new(track.width(), thumb_height),
    );
    let hit_rect = track.expand2(Vec2::new(7.0, 0.0));
    let response = ui.interact(
        hit_rect,
        egui::Id::new(("fluxa-desktop-vertical-scrollbar", active_page)),
        Sense::click_and_drag(),
    );

    let mut next_offset = offset;
    if response.clicked() && !thumb.contains(response.interact_pointer_pos().unwrap_or_default()) {
        if let Some(pointer) = response.interact_pointer_pos() {
            let ratio = ((pointer.y - track.top() - thumb_height * 0.5) / thumb_travel.max(1.0))
                .clamp(0.0, 1.0);
            next_offset = ratio * max_offset;
        }
    } else if response.dragged() {
        let delta_y = context.input(|input| input.pointer.delta().y);
        next_offset =
            (offset + delta_y / thumb_travel.max(1.0) * max_offset).clamp(0.0, max_offset);
    }

    if (next_offset - offset).abs() > f32::EPSILON {
        context.data_mut(|data| data.insert_temp(id, next_offset));
        context.request_repaint();
    }
    if response.hovered() || response.dragged() {
        context.set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }

    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("fluxa-desktop-scrollbar-layer"),
    ));
    painter.rect_filled(track, 3.0, Color32::from_white_alpha(14));
    painter.rect_filled(
        thumb,
        3.0,
        if response.hovered() || response.dragged() {
            Color32::from_white_alpha(145)
        } else {
            Color32::from_white_alpha(82)
        },
    );
}

fn desktop_scroll_id(active_page: &str) -> Option<&'static str> {
    match active_page {
        "Home" => Some("fluxa-screen-scroll-home"),
        "Library" => Some("fluxa-screen-scroll-library"),
        "Discover" => Some("fluxa-screen-scroll-discover"),
        "Calendar" => Some("fluxa-screen-scroll-calendar"),
        "Settings" => Some("fluxa-screen-scroll-settings"),
        "Detail" => Some("fluxa-screen-scroll-detail"),
        _ => None,
    }
}

fn draw_library_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
    screen_state: &mut NativeScreenState,
    animations: &mut UiAnimations,
) {
    let theme = NativeTheme::default();
    let library = runtime
        .snapshot()
        .get("library")
        .cloned()
        .unwrap_or_default();
    let loading = library
        .get("isLoading")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    ui.add_space(34.0);
    ui.horizontal(|ui| {
        theme.header(ui, "Library", "Your saved movies and shows in one place.");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme.refresh_button(ui).clicked() {
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "libraryHydrateRequested",
                    "profileId": null,
                })) {
                    pending_effects.extend(update.effects);
                }
                *status = "Refreshing library…".to_owned();
            }
        });
    });
    ui.add_space(26.0);
    ui.horizontal_wrapped(|ui| {
        for label in ["Watchlist", "Watching", "Completed", "Dropped", "Liked"] {
            let selected = screen_state.library_tab == label;
            if theme
                .chip(
                    ui,
                    RichText::new(label).size(14.0).color(if selected {
                        Color32::WHITE
                    } else {
                        Color32::from_white_alpha(170)
                    }),
                    selected,
                )
                .clicked()
            {
                screen_state.library_tab = label.to_owned();
            }
        }
    });
    ui.add_space(28.0);
    let key = match screen_state.library_tab.as_str() {
        "Watching" => "continueWatching",
        "Completed" => "completed",
        "Dropped" => "dropped",
        "Liked" => "liked",
        _ => "watchlist",
    };
    let items = library
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    ui.horizontal(|ui| {
        ui.heading(screen_state.library_tab.clone());
        ui.label(
            RichText::new(format!(
                "  {} title{}",
                items.len(),
                if items.len() == 1 { "" } else { "s" }
            ))
            .size(13.0)
            .color(Color32::from_white_alpha(130)),
        );
    });
    ui.add_space(12.0);
    if items.is_empty() {
        ui.add_space(70.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(if loading {
                    "Loading your library…"
                } else {
                    "Nothing here yet"
                })
                .size(22.0)
                .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(match screen_state.library_tab.as_str() {
                    "Watchlist" => "Titles you save for later will appear here.",
                    "Watching" => "Start watching something and your progress will show up here.",
                    _ => "Titles from Fluxa Core will appear here when available.",
                })
                .color(Color32::from_white_alpha(145)),
            );
        });
    } else {
        draw_poster_grid(
            ui,
            &items,
            artwork,
            runtime,
            pending_effects,
            status,
            animations,
        );
    }
}

fn draw_discover_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
    screen_state: &mut NativeScreenState,
    animations: &mut UiAnimations,
) {
    let theme = NativeTheme::default();
    let snapshot = runtime.snapshot().clone();
    let discover = snapshot.get("discover").cloned().unwrap_or_default();
    let catalogs = discover
        .get("catalogs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let results = discover
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let loading = discover
        .get("isLoading")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || discover
            .get("catalogsLoading")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    ui.add_space(34.0);
    ui.heading(RichText::new("Discover").size(34.0).strong());
    ui.label(
        RichText::new("Browse catalogs from your installed add-ons.")
            .size(15.0)
            .color(Color32::from_white_alpha(165)),
    );
    ui.add_space(24.0);
    ui.horizontal_wrapped(|ui| {
        for (value, label) in [("movie", "Movies"), ("series", "Series")] {
            let selected = screen_state.discover_content_type == value;
            if theme.chip(ui, label, selected).clicked() {
                screen_state.discover_content_type = value.to_owned();
                screen_state.discover_catalog_key.clear();
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "discoverRequested",
                    "loadCatalogFilters": true,
                    "contentType": value,
                    "filters": { "catalogKey": null, "extra": {} },
                    "language": "en",
                })) {
                    pending_effects.extend(update.effects);
                }
            }
        }
    });
    ui.add_space(14.0);
    if catalogs.is_empty() {
        ui.label(
            RichText::new(if loading {
                "Loading catalogs…"
            } else {
                "No compatible catalogs found in your installed add-ons."
            })
            .color(Color32::from_white_alpha(145)),
        );
    } else {
        ui.horizontal_wrapped(|ui| {
            for catalog in &catalogs {
                let key = value_string(catalog, "key").unwrap_or_default();
                let label = value_string(catalog, "label").unwrap_or_else(|| "Catalog".to_owned());
                let selected = screen_state.discover_catalog_key == key;
                if theme.chip(ui, &label, selected).clicked() {
                    screen_state.discover_catalog_key = key;
                    let filters = json!({
                        "catalogKey": screen_state.discover_catalog_key,
                        "transportUrl": catalog.get("transportUrl").cloned().unwrap_or(Value::Null),
                        "catalogId": catalog.get("id").cloned().unwrap_or(Value::Null),
                        "extra": {}
                    });
                    if let Ok(update) = runtime.dispatch(json!({
                        "type": "discoverRequested",
                        "contentType": screen_state.discover_content_type,
                        "filters": filters,
                        "language": "en",
                    })) {
                        pending_effects.extend(update.effects);
                    }
                    *status = format!("Loading {label}…");
                }
            }
        });
    }
    ui.add_space(30.0);
    if !results.is_empty() {
        ui.heading(RichText::new("Results").size(22.0).strong());
        ui.add_space(12.0);
        draw_poster_grid(
            ui,
            &results,
            artwork,
            runtime,
            pending_effects,
            status,
            animations,
        );
    } else {
        ui.add_space(65.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(if loading {
                    "Loading results…"
                } else if catalogs.is_empty() {
                    "Install an add-on with catalogs to start discovering."
                } else {
                    "Choose a catalog to browse."
                })
                .size(21.0)
                .strong(),
            );
            if let Some(error) = discover.get("error").and_then(Value::as_str) {
                ui.add_space(8.0);
                ui.label(RichText::new(error).color(Color32::from_rgb(245, 130, 110)));
            }
        });
    }
}

fn draw_calendar_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
    screen_state: &mut NativeScreenState,
) {
    let theme = NativeTheme::default();
    let calendar = runtime
        .snapshot()
        .get("calendar")
        .cloned()
        .unwrap_or_default();
    let loading = calendar
        .get("isLoading")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut items = calendar
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    items.extend(
        calendar
            .get("localItems")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    );
    items.extend(
        calendar
            .get("externalItems")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    );
    ui.add_space(34.0);
    ui.horizontal(|ui| {
        theme.header(
            ui,
            "Calendar",
            &format!("{} releases scheduled", items.len()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme.refresh_button(ui).clicked() {
                dispatch_calendar_month(
                    runtime,
                    pending_effects,
                    screen_state.calendar_year,
                    screen_state.calendar_month,
                );
                *status = "Refreshing calendar…".to_owned();
            }
        });
    });
    ui.add_space(22.0);
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("‹").corner_radius(9)).clicked() {
            shift_month(screen_state, -1);
            dispatch_calendar_month(
                runtime,
                pending_effects,
                screen_state.calendar_year,
                screen_state.calendar_month,
            );
        }
        ui.label(
            RichText::new(format!(
                "{} {}",
                month_name(screen_state.calendar_month),
                screen_state.calendar_year
            ))
            .size(20.0)
            .strong(),
        );
        if ui.add(egui::Button::new("›").corner_radius(9)).clicked() {
            shift_month(screen_state, 1);
            dispatch_calendar_month(
                runtime,
                pending_effects,
                screen_state.calendar_year,
                screen_state.calendar_month,
            );
        }
        ui.add_space(20.0);
        ui.label(
            RichText::new(if loading { "Loading…" } else { "" })
                .color(Color32::from_white_alpha(140)),
        );
    });
    ui.add_space(18.0);
    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    egui::Grid::new("fluxa-calendar-grid")
        .num_columns(7)
        .spacing(Vec2::new(8.0, 8.0))
        .show(ui, |ui| {
            for day in weekdays {
                ui.label(
                    RichText::new(day)
                        .size(12.0)
                        .color(Color32::from_white_alpha(130))
                        .strong(),
                );
            }
            ui.end_row();
            let leading =
                weekday_sunday_zero(screen_state.calendar_year, screen_state.calendar_month, 1)
                    as usize;
            let total =
                days_in_month(screen_state.calendar_year, screen_state.calendar_month) as usize;
            let mut cell = 0usize;
            while cell < leading + total {
                for column in 0..7 {
                    let day = cell
                        .checked_sub(leading)
                        .and_then(|value| (value < total).then_some(value + 1));
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new((ui.available_width() / 7.0 - 8.0).max(92.0), 108.0),
                        Sense::click(),
                    );
                    ui.painter().rect_filled(
                        rect,
                        10.0,
                        if day.is_some() {
                            Color32::from_rgb(19, 20, 25)
                        } else {
                            Color32::from_rgb(10, 10, 12)
                        },
                    );
                    if let Some(day) = day {
                        ui.painter().text(
                            rect.left_top() + Vec2::new(10.0, 8.0),
                            Align2::LEFT_TOP,
                            day.to_string(),
                            FontId::proportional(14.0),
                            Color32::from_white_alpha(190),
                        );
                        let date = format!(
                            "{:04}-{:02}-{:02}",
                            screen_state.calendar_year, screen_state.calendar_month, day
                        );
                        let day_items = items
                            .iter()
                            .filter(|item| {
                                calendar_item_date(item).as_deref() == Some(date.as_str())
                            })
                            .take(2)
                            .collect::<Vec<_>>();
                        for (index, item) in day_items.iter().enumerate() {
                            let title = first_value_string(item, &["title", "name"])
                                .unwrap_or_else(|| "Release".to_owned());
                            let line = format!("• {}", truncate_text(&title, 14));
                            ui.painter().text(
                                rect.left_top() + Vec2::new(10.0, 34.0 + index as f32 * 22.0),
                                Align2::LEFT_TOP,
                                line,
                                FontId::proportional(12.0),
                                Color32::WHITE,
                            );
                            if let Some(url) = first_value_string(
                                item,
                                &["resolvedArtworkUrl", "artworkUrl", "poster", "seriesPoster"],
                            ) {
                                if let Some(texture) = artwork.texture(Some(url.as_str())) {
                                    ui.painter().image(
                                        texture,
                                        egui::Rect::from_min_size(
                                            rect.right_top() + Vec2::new(-38.0, 8.0),
                                            Vec2::new(28.0, 40.0),
                                        ),
                                        full_uv(),
                                        Color32::WHITE,
                                    );
                                }
                            }
                        }
                        if response.clicked() && !day_items.is_empty() {
                            *status = format!(
                                "{} release{} on {}",
                                day_items.len(),
                                if day_items.len() == 1 { "" } else { "s" },
                                date
                            );
                        }
                    }
                    if column == 6 {
                        ui.end_row();
                    }
                    cell += 1;
                }
            }
        });
    if items.is_empty() && !loading {
        ui.add_space(18.0);
        ui.label(
            RichText::new("No releases found for this month.")
                .color(Color32::from_white_alpha(145)),
        );
    }
}

fn draw_settings_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
) {
    let values = runtime
        .snapshot()
        .pointer("/settings/values")
        .cloned()
        .unwrap_or_else(|| json!({}));
    ui.add_space(34.0);
    ui.heading(RichText::new("Settings").size(34.0).strong());
    ui.label(
        RichText::new("Playback, appearance, accounts, and storage.")
            .size(15.0)
            .color(Color32::from_white_alpha(165)),
    );
    ui.add_space(28.0);
    egui::Grid::new("fluxa-settings-grid")
        .num_columns(2)
        .spacing(Vec2::new(18.0, 18.0))
        .show(ui, |ui| {
            for (index, (section, description, rows)) in [
                (
                    "General",
                    "How Fluxa starts and behaves",
                    [
                        ("Animations", "animationsEnabled"),
                        ("Notifications", "notificationsEnabled"),
                    ],
                ),
                (
                    "Playback",
                    "Defaults for watching content",
                    [
                        ("Autoplay next episode", "autoplayNext"),
                        ("Remember playback position", "rememberPlayback"),
                    ],
                ),
                (
                    "Appearance",
                    "Visual preferences",
                    [
                        ("Use compact cards", "compactCards"),
                        ("Show episode badges", "showEpisodeBadges"),
                    ],
                ),
                (
                    "Storage",
                    "Local data and cache",
                    [
                        ("Cache artwork", "cacheArtwork"),
                        ("Use hardware decoding", "hardwareDecoding"),
                    ],
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(330.0, 146.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, 12.0, Color32::from_rgb(18, 19, 24));
                ui.painter().text(
                    rect.left_top() + Vec2::new(16.0, 14.0),
                    Align2::LEFT_TOP,
                    section,
                    FontId::proportional(18.0),
                    Color32::WHITE,
                );
                ui.painter().text(
                    rect.left_top() + Vec2::new(16.0, 38.0),
                    Align2::LEFT_TOP,
                    description,
                    FontId::proportional(12.0),
                    Color32::from_white_alpha(130),
                );
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(
                        rect.shrink2(Vec2::new(16.0, 58.0))
                            .translate(Vec2::new(0.0, 42.0)),
                    ),
                    |ui| {
                        for (label, key) in rows {
                            let mut value =
                                values.get(key).and_then(Value::as_bool).unwrap_or(true);
                            if ui.checkbox(&mut value, label).changed() {
                                if let Ok(update) = runtime.dispatch(
                                    json!({"type":"settingsChanged", "key":key, "value":value}),
                                ) {
                                    pending_effects.extend(update.effects);
                                }
                                *status = format!("Updated {label}");
                            }
                        }
                    },
                );
                if index % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    ui.add_space(18.0);
    ui.label(
        RichText::new(if status.is_empty() {
            "Changes are saved through Fluxa Core."
        } else {
            status.as_str()
        })
        .color(Color32::from_white_alpha(135)),
    );
}

fn draw_poster_grid(
    ui: &mut egui::Ui,
    items: &[Value],
    artwork: &mut ArtworkRegistry,
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
    animations: &mut UiAnimations,
) {
    let columns = ((ui.available_width() / 174.0).floor() as usize).max(1);
    let row_height = 280.0;
    let row_gap = 18.0;
    let row_step = row_height + row_gap;
    let row_count = items.len().div_ceil(columns);
    let visible_rect = ui.clip_rect();
    let content_top = ui.cursor().top();
    let first_row = ((visible_rect.top() - content_top) / row_step)
        .floor()
        .max(0.0) as usize;
    let last_row = ((visible_rect.bottom() - content_top) / row_step)
        .ceil()
        .max(0.0) as usize;
    let first_row = first_row.min(row_count);
    let last_row = last_row.min(row_count);
    if first_row > 0 {
        ui.add_space(first_row as f32 * row_step);
    }
    for row_index in first_row..last_row {
        let row = &items[row_index * columns..((row_index + 1) * columns).min(items.len())];
        ui.horizontal(|ui| {
            for item in row {
                draw_poster_card(
                    ui,
                    item,
                    artwork,
                    runtime,
                    pending_effects,
                    status,
                    animations,
                );
            }
        });
        if row_index + 1 < row_count {
            ui.add_space(row_gap);
        }
    }
    let reserved_height = row_count.saturating_sub(last_row) as f32 * row_step;
    if reserved_height > 0.0 {
        ui.add_space(reserved_height);
    }
}

fn dispatch_calendar_month(
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
    year: i32,
    month: i32,
) {
    if let Ok(update) =
        runtime.dispatch(json!({"type":"calendarMonthRequested", "year":year, "month":month}))
    {
        pending_effects.extend(update.effects);
    }
}

fn shift_month(state: &mut NativeScreenState, delta: i32) {
    let index = state.calendar_year.saturating_mul(12) + state.calendar_month - 1 + delta;
    state.calendar_year = index.div_euclid(12);
    state.calendar_month = index.rem_euclid(12) + 1;
}

fn month_name(month: i32) -> &'static str {
    [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ]
    .get(month.saturating_sub(1) as usize)
    .copied()
    .unwrap_or("Month")
}

fn calendar_item_date(item: &Value) -> Option<String> {
    first_value_string(
        item,
        &["dateIso", "airDate", "released", "releaseDate", "date"],
    )
    .map(|date| date.chars().take(10).collect())
}

fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn weekday_sunday_zero(year: i32, month: i32, day: i32) -> i32 {
    let (mut year, mut month) = (year, month);
    if month < 3 {
        year -= 1;
        month += 12;
    }
    (day + (13 * (month + 1)) / 5 + year + year / 4 - year / 100 + year / 400 + 6).rem_euclid(7)
}

fn current_year_month() -> (i32, i32) {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    let mut year = 1970;
    let mut remaining = days as i64;
    while remaining >= i64::from(if is_leap_year(year) { 366 } else { 365 }) {
        remaining -= i64::from(if is_leap_year(year) { 366 } else { 365 });
        year += 1;
    }
    let mut month = 1;
    while remaining >= i64::from(days_in_month(year, month)) {
        remaining -= i64::from(days_in_month(year, month));
        month += 1;
    }
    (year, month)
}

struct NativeScreenState {
    library_tab: String,
    library_query: String,
    library_sort: String,
    library_model_cache: Option<((u64, String, String, String), fluxa_ui::LibraryModel)>,
    discover_model_cache: Option<((u64, String, String, String), fluxa_ui::DiscoverModel)>,
    discover_projection_sender: Sender<(u64, usize, Vec<fluxa_ui::HomeCard>)>,
    discover_projection_receiver: Receiver<(u64, usize, Vec<fluxa_ui::HomeCard>)>,
    discover_projection_pending: Option<(u64, usize)>,
    calendar_selected_day: Option<u32>,
    settings_section: usize,
    settings_addon_url: String,
    settings_plugin_url: String,
    discover_content_type: String,
    discover_catalog_key: String,
    discover_extra_value: String,
    calendar_year: i32,
    calendar_month: i32,
}

impl Default for NativeScreenState {
    fn default() -> Self {
        let (year, month) = current_year_month();
        let (discover_projection_sender, discover_projection_receiver) = mpsc::channel();
        Self {
            library_tab: "Watchlist".to_owned(),
            library_query: String::new(),
            library_sort: "recent".to_owned(),
            library_model_cache: None,
            discover_model_cache: None,
            discover_projection_sender,
            discover_projection_receiver,
            discover_projection_pending: None,
            calendar_selected_day: None,
            settings_section: 0,
            settings_addon_url: String::new(),
            settings_plugin_url: String::new(),
            discover_content_type: "movie".to_owned(),
            discover_catalog_key: String::new(),
            discover_extra_value: String::new(),
            calendar_year: year,
            calendar_month: month,
        }
    }
}

struct FluxaDesktopApp {
    window: Option<Arc<Window>>,
    ui: Option<NativeUi>,
    runtime: Option<FluxaRuntime>,
    focused_card: usize,
    search: String,
    status: String,
    active_page: String,
    screen_state: NativeScreenState,
    effect_executor: Option<EffectExecutor>,
    effect_sender: Option<Sender<EffectCompletion>>,
    effect_receiver: Option<Receiver<EffectCompletion>>,
    pending_effects: Vec<Value>,
    pending_player_meta: Option<Value>,
    player_started: bool,
    force_redraw: bool,
    benchmark_target_fps: Option<u32>,
    next_benchmark_redraw: Option<Instant>,
    next_scheduled_redraw: Option<Instant>,
}

impl Default for FluxaDesktopApp {
    fn default() -> Self {
        Self {
            window: None,
            ui: None,
            runtime: None,
            focused_card: 0,
            search: String::new(),
            status: String::new(),
            active_page: std::env::var("FLUXA_NATIVE_INITIAL_PAGE")
                .ok()
                .filter(|page| {
                    matches!(
                        page.as_str(),
                        "Home" | "Library" | "Discover" | "Calendar" | "Settings" | "Detail"
                    )
                })
                .unwrap_or_else(|| "Home".to_owned()),
            screen_state: NativeScreenState::default(),
            effect_executor: None,
            effect_sender: None,
            effect_receiver: None,
            pending_effects: Vec::new(),
            pending_player_meta: None,
            player_started: false,
            force_redraw: std::env::var_os("FLUXA_NATIVE_FORCE_REDRAW").is_some()
                || benchmark_target_fps().is_some()
                || std::env::var_os("FLUXA_NATIVE_BENCH_UNCAPPED").is_some(),
            benchmark_target_fps: benchmark_target_fps(),
            next_benchmark_redraw: None,
            next_scheduled_redraw: None,
        }
    }
}

impl FluxaDesktopApp {
    fn schedule_effects(&self, effects: Vec<Value>) {
        let (Some(executor), Some(sender)) = (&self.effect_executor, &self.effect_sender) else {
            if !effects.is_empty() {
                eprintln!(
                    "[fluxa-native] cannot schedule {} effect(s): executor is not initialized",
                    effects.len()
                );
            }
            return;
        };
        for effect in effects {
            eprintln!(
                "[fluxa-native] scheduling effect type={} id={}",
                effect
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("<missing>"),
                effect
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("<missing>")
            );
            executor.spawn(effect, sender.clone());
        }
    }

    fn drain_effect_completions(&mut self) -> bool {
        let Some(receiver) = self.effect_receiver.as_ref() else {
            return false;
        };
        let mut follow_up_effects = Vec::new();
        let mut changed = false;
        while let Ok(completion) = receiver.try_recv() {
            changed = true;
            let is_discover_catalogs = completion.value.get("addons").is_some();
            eprintln!(
                "[fluxa-native] effect completed id={} status={}",
                completion.effect_id, completion.status
            );
            let Some(runtime) = self.runtime.as_mut() else {
                continue;
            };
            let completion_started = Instant::now();
            let completion_result = if completion.effect_type == "fetchDiscoverPage" {
                runtime.complete_discover_page_result(
                    &completion.effect_id,
                    completion.status,
                    completion.value,
                    completion.error,
                )
            } else {
                runtime.complete_effect_result(
                    &completion.effect_id,
                    completion.status,
                    completion.value,
                    completion.error,
                )
            };
            let completion_ms = completion_started.elapsed().as_secs_f64() * 1_000.0;
            if completion_ms >= 8.0 {
                eprintln!(
                    "[fluxa-native] effect completion type={} applied in {:.1}ms",
                    completion.effect_type, completion_ms,
                );
            }
            match completion_result {
                Ok(update) => {
                    if is_discover_catalogs {
                        let snapshot = runtime.snapshot();
                        eprintln!(
                            "[fluxa-native] Discover snapshot after completion: catalogs={} content_type={} follow_up_effects={}",
                            snapshot
                                .pointer("/discover/catalogs")
                                .and_then(Value::as_array)
                                .map_or(0, Vec::len),
                            snapshot
                                .pointer("/discover/contentType")
                                .and_then(Value::as_str)
                                .unwrap_or("<missing>"),
                            update.effects.len(),
                        );
                    }
                    follow_up_effects.extend(update.effects);
                }
                Err(error) => {
                    eprintln!(
                        "[fluxa-native] effect {} completion was rejected: {error}",
                        completion.effect_id
                    );
                    self.status = format!("Native effect failed: {error}");
                }
            }
        }
        if !follow_up_effects.is_empty() {
            self.schedule_effects(follow_up_effects);
        }
        changed || self.pump_player_resolution()
    }

    fn pump_player_resolution(&mut self) -> bool {
        if self.player_started {
            return false;
        }
        let Some(meta) = self.pending_player_meta.clone() else {
            return false;
        };
        let Some(runtime) = self.runtime.as_mut() else {
            return false;
        };

        let snapshot = runtime.snapshot().clone();
        if let Some(streams) = snapshot
            .pointer("/player/currentStreams")
            .and_then(Value::as_array)
        {
            let selected_index = snapshot
                .pointer("/player/currentStreamIndex")
                .and_then(Value::as_i64)
                .unwrap_or(0)
                .max(0) as usize;
            let stream = streams.get(selected_index).cloned().unwrap_or(Value::Null);
            let url = snapshot
                .pointer("/player/currentUrl")
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty())
                .map(ToOwned::to_owned)
                .or_else(|| native_playback_url(&stream, &meta));
            if let Some(url) = url {
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "playerResolvePlaybackRequested",
                    "url": url,
                    "stream": stream,
                    "currentVideoId": snapshot
                        .pointer("/player/currentVideoId")
                        .cloned()
                        .unwrap_or(Value::Null),
                    "title": meta
                        .get("name")
                        .or_else(|| meta.get("title"))
                        .and_then(Value::as_str)
                        .unwrap_or("Fluxa"),
                })) {
                    self.pending_effects.extend(update.effects);
                    self.player_started = true;
                    self.status = "Player source ready".to_owned();
                    return true;
                }
            }
        }

        if snapshot
            .pointer("/player/pendingStreamLoad")
            .is_some_and(|value| value.is_object())
        {
            return false;
        }
        let Some(target) = snapshot
            .pointer("/player/directPlaybackTarget")
            .filter(|value| value.is_object())
        else {
            return false;
        };
        let Some(streams) = target.get("streams").and_then(Value::as_array) else {
            return false;
        };
        if streams.is_empty() {
            self.status = "No streams were returned by Fluxa Core".to_owned();
            return true;
        }
        let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("movie");
        let Some(content_id) = meta.get("id").and_then(Value::as_str) else {
            self.status = "Player source is missing a content id".to_owned();
            self.pending_player_meta = None;
            return true;
        };
        let current_video_id = meta
            .get("lastVideoId")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .unwrap_or(content_id);
        let profile = snapshot
            .pointer("/profile/active")
            .cloned()
            .unwrap_or(Value::Null);
        let update = runtime.dispatch(json!({
            "type": "playerLoadStreamsRequested",
            "contentType": content_type,
            "id": current_video_id,
            "currentVideoId": current_video_id,
            "initialVideoId": current_video_id,
            "initialStreams": streams,
            "initialStreamIndex": meta.get("lastStreamIndex").cloned().unwrap_or(json!(0)),
            "savedUrl": meta.get("lastStreamUrl").cloned().unwrap_or(Value::Null),
            "savedTitle": meta.get("lastStreamTitle").cloned().unwrap_or(Value::Null),
            "sourceSelectionMode": profile.get("streamSourceSelectionMode").and_then(Value::as_str).unwrap_or("manual"),
            "regexPattern": profile.get("streamSourceRegexPattern"),
            "title": meta.get("name").or_else(|| meta.get("title")),
            "originalName": meta.get("originalName"),
            "year": meta.get("year"),
            "language": "en",
            "profile": profile,
        }));
        match update {
            Ok(update) => {
                self.pending_effects.extend(update.effects);
                true
            }
            Err(error) => {
                self.status = format!("Fluxa Core could not select a player source: {error}");
                true
            }
        }
    }
}

impl ApplicationHandler for FluxaDesktopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Fluxa")
            .with_inner_size(native_window_size());
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("create Fluxa window"),
        );
        let ui = pollster::block_on(NativeUi::new(window.clone()))
            .expect("initialize Fluxa egui renderer");
        let storage = platform::Storage::open_default().expect("initialize Fluxa storage");
        let initial_prefs = storage
            .read_json("prefs")
            .ok()
            .flatten()
            .unwrap_or_else(|| json!({}));
        let (initial_state, fixture_mode) = native_initial_runtime_state(initial_prefs, &storage);
        let mut runtime = FluxaRuntime::new(initial_state).expect("initialize Fluxa core runtime");
        let initial_effects = if fixture_mode {
            eprintln!("[fluxa-native] fixture mode enabled; skipping initial network load");
            Vec::new()
        } else {
            match self.active_page.as_str() {
            "Library" => runtime
                .dispatch(json!({"type":"libraryHydrateRequested", "profileId":null}))
                .expect("request Fluxa library data")
                .effects,
            "Discover" => runtime
                .dispatch(json!({"type":"discoverRequested", "loadCatalogFilters":true, "contentType":"movie", "filters":{"catalogKey":null,"extra":{}}, "language":"en"}))
                .expect("request Fluxa discover catalogs")
                .effects,
            "Calendar" => runtime
                .dispatch(json!({"type":"calendarMonthRequested", "year":current_year_month().0, "month":current_year_month().1}))
                .expect("request Fluxa calendar data")
                .effects,
            "Settings" => runtime
                .dispatch(json!({"type":"navigationRequested", "route":"settings", "params":null}))
                .expect("open Fluxa settings")
                .effects,
            "Detail" => runtime
                .dispatch(json!({"type":"navigationRequested", "route":"detail", "params":null}))
                .expect("open Fluxa detail")
                .effects,
            _ => runtime
                .dispatch(json!({"type":"homeLoadRequested", "language":"en", "force":false}))
                .expect("request Fluxa home data")
                .effects,
            }
        };
        eprintln!(
            "[fluxa-native] initial {} load dispatched with {} effect(s)",
            self.active_page,
            initial_effects.len()
        );
        self.window = Some(window.clone());
        self.ui = Some(ui);
        self.runtime = Some(runtime);
        let (sender, receiver) = mpsc::channel();
        self.effect_executor = Some(EffectExecutor::new(storage));
        if let Some(executor) = self.effect_executor.as_ref() {
            executor.warm_torrent_engine();
        }
        self.effect_sender = Some(sender);
        self.effect_receiver = Some(receiver);
        self.status = match self.active_page.as_str() {
            "Home" => "Loading home data from Fluxa Core…",
            "Library" => "Loading library from Fluxa Core…",
            "Discover" => "Loading catalogs from Fluxa Core…",
            "Calendar" => "Loading calendar from Fluxa Core…",
            _ => "Settings loaded from Fluxa Core",
        }
        .to_owned();
        self.schedule_effects(initial_effects);
        window.request_redraw();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        if let Some(ui) = self.ui.as_mut() {
            ui.poll_wayland_presentation();
        }
        let effects_changed = self.drain_effect_completions();
        let artwork_pending = self.ui.as_ref().is_some_and(|ui| ui.artwork.has_pending());
        let animations_active = self.ui.as_ref().is_some_and(NativeUi::animations_active);
        let player_active = self
            .ui
            .as_ref()
            .is_some_and(|ui| ui.player.active_or_initializing());
        let resize_pending = self.ui.as_ref().is_some_and(NativeUi::resize_pending);
        if self.force_redraw
            && let Some(window) = self.window.as_ref()
        {
            if let Some(fps) = self.benchmark_target_fps {
                let period = Duration::from_secs_f64(1.0 / f64::from(fps));
                let now = Instant::now();
                let deadline = self.next_benchmark_redraw.get_or_insert(now);
                if now >= *deadline {
                    window.request_redraw();
                    self.next_benchmark_redraw = Some(now + period);
                } else {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(*deadline));
                }
            } else {
                window.request_redraw();
            }
        } else {
            if (effects_changed || player_active)
                && let Some(window) = self.window.as_ref()
            {
                window.request_redraw();
            }
            // Artwork decoding, GIF/WebP playback, short UI transitions and
            // resize settling do not need to redraw at the monitor's maximum
            // refresh rate. The old loop requested a new frame on every
            // AboutToWait callback (up to 180+ fps), keeping the CPU busy
            // while the UI was otherwise idle. Cap active non-video work at
            // 120 Hz, or at the monitor's refresh rate on slower displays;
            // the artwork scheduler still advances using source frame
            // durations, while input and video redraws remain immediate.
            if (artwork_pending || animations_active || resize_pending) && !player_active {
                let now = Instant::now();
                let deadline = self.next_scheduled_redraw.get_or_insert(now);
                if now >= *deadline {
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                    let interval = self
                        .window
                        .as_ref()
                        .map(|window| active_redraw_interval(window))
                        .unwrap_or(Duration::from_millis(16));
                    *deadline = now + interval;
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(*deadline));
            } else {
                self.next_scheduled_redraw = None;
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let (Some(window), Some(ui)) = (self.window.as_ref(), self.ui.as_mut()) {
            ui.event(window, &event);
            if !matches!(&event, WindowEvent::RedrawRequested) {
                window.request_redraw();
            }
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.resize(size);
                    ui.player.invalidate_video_surface("window resize");
                }
            }
            WindowEvent::Focused(true) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.player
                        .invalidate_video_surface("workspace/focus restore");
                }
            }
            WindowEvent::Occluded(false) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.player.invalidate_video_surface("window unoccluded");
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(ui), Some(runtime)) = (
                    self.window.as_ref(),
                    self.ui.as_mut(),
                    self.runtime.as_mut(),
                ) {
                    ui.render(
                        window,
                        &mut self.focused_card,
                        &mut self.search,
                        &mut self.status,
                        &mut self.active_page,
                        &mut self.screen_state,
                        runtime,
                        self.effect_executor
                            .as_ref()
                            .expect("torrent/player effect executor initialized"),
                        &mut self.pending_effects,
                        &mut self.pending_player_meta,
                        &mut self.player_started,
                    );
                    let effects = std::mem::take(&mut self.pending_effects);
                    self.schedule_effects(effects);
                }
            }
            _ => {}
        }
    }
}

fn native_initial_runtime_state(initial_prefs: Value, storage: &Storage) -> (Value, bool) {
    let Some(path) = std::env::var_os("FLUXA_NATIVE_FIXTURE") else {
        return (
            fluxa_effects::persisted_runtime_state(storage, initial_prefs),
            false,
        );
    };
    let path = std::path::PathBuf::from(path);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        eprintln!("[fluxa-native] fixture not readable: {}", path.display());
        return (json!({"settings": {"values": initial_prefs}}), false);
    };
    let Ok(mut state) = serde_json::from_str::<Value>(&raw) else {
        eprintln!(
            "[fluxa-native] fixture is not valid JSON: {}",
            path.display()
        );
        return (json!({"settings": {"values": initial_prefs}}), false);
    };
    if let Value::Object(object) = &mut state {
        object
            .entry("settings")
            .or_insert_with(|| json!({"values": initial_prefs.clone()}));
        (state, true)
    } else {
        eprintln!(
            "[fluxa-native] fixture root must be a JSON object: {}",
            path.display()
        );
        (json!({"settings": {"values": initial_prefs}}), false)
    }
}

fn native_core_value(method: &str, args: Value) -> Option<Value> {
    let response = fluxa_core::ffi::core_invoke(method, &args.to_string());
    let envelope: Value = serde_json::from_str(&response).ok()?;
    (envelope.get("ok").and_then(Value::as_bool) == Some(true))
        .then(|| envelope.get("value").cloned().unwrap_or(Value::Null))
}

fn native_playback_url(stream: &Value, meta: &Value) -> Option<String> {
    let raw = fluxa_core::ffi::core_invoke(
        "playbackPreparePlan",
        &json!({"stream": stream, "meta": meta}).to_string(),
    );
    let envelope: Value = serde_json::from_str(&raw).ok()?;
    let plan = envelope
        .get("ok")
        .and_then(Value::as_bool)
        .filter(|ok| *ok)
        .and_then(|_| envelope.get("value"))?;
    let mode = plan.get("mode").and_then(Value::as_str)?;
    if matches!(mode, "direct" | "torrent" | "external") {
        plan.get("url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned)
    } else {
        None
    }
}

fn torrent_loading_details(status: Option<&Value>) -> (String, String) {
    let Some(status) = status else {
        return (
            "Resolving torrent metadata…".to_owned(),
            "Connecting to trackers and discovering peers".to_owned(),
        );
    };
    if status.get("stat").and_then(Value::as_i64) == Some(-1) {
        return (
            "Torrent stream reported an error".to_owned(),
            status
                .get("error")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .unwrap_or("Still waiting for the selected torrent")
                .to_owned(),
        );
    }

    let phase = status
        .get("phase")
        .and_then(Value::as_str)
        .or_else(|| status.get("stat_string").and_then(Value::as_str))
        .unwrap_or("resolving_metadata");
    if phase == "initializing"
        && status.get("resolving").and_then(Value::as_bool) == Some(false)
        && status
            .get("hash")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    {
        return (
            "Torrent metadata has not resolved yet".to_owned(),
            "The selected source is still waiting for a metadata response".to_owned(),
        );
    }
    let peers = status
        .get("active_peers")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let peer_total = status
        .get("total_peers")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let percent = status
        .get("preload")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .round()
        .clamp(0.0, 100.0) as u64;
    let downloaded = status
        .get("loaded_size")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let speed = status
        .get("download_speed")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let phase_label = match phase {
        "resolving_metadata" | "initializing" | "resolving" => "Fetching torrent metadata…",
        "connecting_peers" => "Discovering peers…",
        "buffering_startup" => "Downloading startup buffer…",
        "rebuffering" => "Refilling playback buffer…",
        "seeking" => "Preparing the requested playback position…",
        "streaming" => "Torrent buffer ready · starting video…",
        "stalled" => "Torrent transfer is paused",
        "error" => "Torrent stream reported an error",
        "status_unavailable" => "Checking torrent engine status…",
        _ if peers == 0 => "Discovering peers…",
        _ => "Downloading torrent data…",
    };
    if phase == "status_unavailable" {
        return (
            phase_label.to_owned(),
            status
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("No response from the torrent status endpoint")
                .to_owned(),
        );
    }
    let peer_label = if peer_total > 0 {
        format!("{peers}/{peer_total} peers")
    } else {
        format!("{peers} peers")
    };
    let downloaded_label = if downloaded >= 1024 * 1024 {
        format!(
            "{:.1} MiB downloaded",
            downloaded as f64 / (1024.0 * 1024.0)
        )
    } else {
        format!("{} KiB downloaded", downloaded / 1024)
    };
    let speed_label = if speed >= 1024.0 * 1024.0 {
        format!("{:.1} MiB/s", speed / (1024.0 * 1024.0))
    } else if speed >= 1024.0 {
        format!("{:.0} KiB/s", speed / 1024.0)
    } else {
        format!("{:.0} B/s", speed)
    };
    (
        phase_label.to_owned(),
        format!("{peer_label} · {percent}% buffer · {downloaded_label} · {speed_label}"),
    )
}

fn value_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn native_window_size() -> winit::dpi::LogicalSize<f64> {
    let Some(raw) = std::env::var_os("FLUXA_NATIVE_WINDOW_SIZE") else {
        return winit::dpi::LogicalSize::new(1280.0, 800.0);
    };
    let raw = raw.to_string_lossy();
    let Some((width, height)) = raw.split_once('x') else {
        return winit::dpi::LogicalSize::new(1280.0, 800.0);
    };
    match (width.parse::<f64>(), height.parse::<f64>()) {
        (Ok(width), Ok(height))
            if width.is_finite() && height.is_finite() && width >= 640.0 && height >= 480.0 =>
        {
            winit::dpi::LogicalSize::new(width, height)
        }
        _ => winit::dpi::LogicalSize::new(1280.0, 800.0),
    }
}

fn draw_horizontal_gradient(
    painter: &egui::Painter,
    rect: egui::Rect,
    start: Color32,
    end: Color32,
) {
    let mut mesh = egui::epaint::Mesh::default();
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend([
        egui::epaint::Vertex {
            pos: rect.left_top(),
            uv: egui::pos2(0.0, 0.0),
            color: start,
        },
        egui::epaint::Vertex {
            pos: rect.right_top(),
            uv: egui::pos2(0.0, 0.0),
            color: end,
        },
        egui::epaint::Vertex {
            pos: rect.right_bottom(),
            uv: egui::pos2(0.0, 0.0),
            color: end,
        },
        egui::epaint::Vertex {
            pos: rect.left_bottom(),
            uv: egui::pos2(0.0, 0.0),
            color: start,
        },
    ]);
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_vertical_gradient(painter: &egui::Painter, rect: egui::Rect, start: Color32, end: Color32) {
    let mut mesh = egui::epaint::Mesh::default();
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend([
        egui::epaint::Vertex {
            pos: rect.left_top(),
            uv: egui::pos2(0.0, 0.0),
            color: start,
        },
        egui::epaint::Vertex {
            pos: rect.right_top(),
            uv: egui::pos2(0.0, 0.0),
            color: start,
        },
        egui::epaint::Vertex {
            pos: rect.right_bottom(),
            uv: egui::pos2(0.0, 0.0),
            color: end,
        },
        egui::epaint::Vertex {
            pos: rect.left_bottom(),
            uv: egui::pos2(0.0, 0.0),
            color: end,
        },
    ]);
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    painter.add(egui::Shape::mesh(mesh));
}

fn full_uv() -> egui::Rect {
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
}

fn cover_uv_with_position(
    size: [u32; 2],
    destination: egui::Rect,
    vertical_position: f32,
) -> egui::Rect {
    let image_aspect = size[0] as f32 / size[1].max(1) as f32;
    let destination_aspect = destination.width() / destination.height().max(1.0);
    if image_aspect > destination_aspect {
        let visible_width = destination_aspect / image_aspect;
        let crop = (1.0 - visible_width) * 0.5;
        egui::Rect::from_min_max(egui::pos2(crop, 0.0), egui::pos2(1.0 - crop, 1.0))
    } else {
        let visible_height = image_aspect / destination_aspect.max(f32::EPSILON);
        let position = vertical_position.clamp(0.0, 1.0);
        let top = (1.0 - visible_height) * position;
        egui::Rect::from_min_max(egui::pos2(0.0, top), egui::pos2(1.0, top + visible_height))
    }
}

fn contain_rect(container: egui::Rect, size: [u32; 2]) -> egui::Rect {
    let image_size = Vec2::new(size[0] as f32, size[1].max(1) as f32);
    let scale = (container.width() / image_size.x).min(container.height() / image_size.y);
    let drawn_size = image_size * scale;
    egui::Rect::from_center_size(container.center(), drawn_size)
}

fn category_title(category: &Value, index: usize) -> String {
    first_value_string(category, &["homeTitle", "name", "label", "title", "id"])
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| format!("Catalog {}", index + 1))
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn continue_episode_code(item: &Value) -> Option<String> {
    let season = item
        .get("lastEpisodeSeason")
        .or_else(|| item.get("season"))
        .and_then(Value::as_i64)?;
    let number = item
        .get("lastEpisodeNumber")
        .or_else(|| item.get("episode"))
        .and_then(Value::as_i64)?;
    Some(format!("S{season}:E{number}"))
}

fn continue_remaining_label(item: &Value) -> Option<String> {
    if item.get("continueWatchingBadge").and_then(Value::as_str) == Some("newEpisode") {
        return Some("New episode".to_owned());
    }
    if item.get("continueWatchingBadge").and_then(Value::as_str) == Some("upNext") {
        return Some("Up next".to_owned());
    }
    let offset = item.get("timeOffset").and_then(Value::as_f64)?;
    let duration = item.get("duration").and_then(Value::as_f64)?;
    if duration <= 0.0 || !offset.is_finite() || !duration.is_finite() {
        return None;
    }
    let minutes = ((duration - offset).max(0.0) / 60.0).ceil() as i64;
    Some(format!("{minutes} min left"))
}

fn draw_badge(painter: &egui::Painter, anchor: egui::Pos2, text: &str, align: Align2) {
    let width = (text.chars().count() as f32 * 6.5 + 14.0).clamp(44.0, 112.0);
    let size = Vec2::new(width, 22.0);
    let rect = match align {
        Align2::LEFT_TOP => egui::Rect::from_min_size(anchor, size),
        Align2::RIGHT_TOP => egui::Rect::from_min_size(anchor - Vec2::new(width, 0.0), size),
        _ => egui::Rect::from_center_size(anchor, size),
    };
    painter.rect_filled(rect, 6.0, Color32::from_rgba_unmultiplied(18, 18, 20, 220));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.0),
        Color32::WHITE,
    );
}

fn item_year(item: &Value) -> String {
    if let Some(year) = item.get("year").and_then(Value::as_i64) {
        return year.to_string();
    }
    if let Some(year) = value_string(item, "year") {
        return year;
    }
    value_string(item, "releaseInfo")
        .or_else(|| value_string(item, "releaseDate"))
        .map(|value| value.chars().take(4).collect())
        .unwrap_or_default()
}

fn draw_poster_card(
    ui: &mut egui::Ui,
    item: &Value,
    artwork: &mut ArtworkRegistry,
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
    animations: &mut UiAnimations,
) {
    let title = value_string(item, "name").unwrap_or_else(|| "Untitled".to_owned());
    let year = item_year(item);
    let card_width = 156.0;
    let tile_height = 234.0;
    ui.vertical(|ui| {
        ui.set_width(card_width);
        let (tile, response) =
            ui.allocate_exact_size(Vec2::new(card_width, tile_height), Sense::click());
        let item_artwork = first_value_string(item, &["poster", "posterUrl", "background"]);
        // ScrollArea still lays out the complete row, but only cards near the
        // viewport should start network/decode work. This is the same lazy
        // loading contract the Web renderer gets from loading="lazy".
        let item_texture = ui
            .is_rect_visible(tile.expand(32.0))
            .then(|| artwork.texture(item_artwork.as_deref()))
            .flatten();
        let animation_key = value_string(item, "id").unwrap_or_else(|| title.clone());
        let focus_progress = animations.value(
            format!("poster-focus:{animation_key}"),
            if response.hovered() { 1.0 } else { 0.0 },
            150,
            Easing::EaseOutCubic,
        );
        let artwork_progress = animations.value(
            format!("poster-artwork:{animation_key}"),
            if item_texture.is_some() { 1.0 } else { 0.0 },
            180,
            Easing::EaseOutCubic,
        );
        let visual_tile = egui::Rect::from_center_size(
            tile.center(),
            tile.size() * (1.0 + focus_progress * 0.025),
        );
        ui.painter()
            .rect_filled(visual_tile, 11.0, Color32::from_rgb(26, 27, 32));
        if let Some(texture) = item_texture {
            ui.painter().image(
                texture,
                visual_tile,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::from_white_alpha((artwork_progress * 255.0) as u8),
            );
        }
        if item_texture.is_none() || artwork_progress < 0.99 {
            let initials = title
                .split_whitespace()
                .take(2)
                .filter_map(|part| part.chars().next())
                .collect::<String>()
                .to_uppercase();
            ui.painter().text(
                visual_tile.center(),
                Align2::CENTER_CENTER,
                initials,
                FontId::proportional(30.0),
                Color32::from_white_alpha((70.0 * (1.0 - artwork_progress)) as u8),
            );
        }
        if focus_progress > 0.001 {
            ui.painter().rect_stroke(
                visual_tile.expand(3.0),
                14.0,
                egui::Stroke::new(
                    2.0,
                    Color32::from_white_alpha((220.0 * focus_progress) as u8),
                ),
                egui::StrokeKind::Outside,
            );
        }
        if response.clicked() {
            if let Some(id) = value_string(item, "id") {
                let content_type = value_string(item, "type")
                    .or_else(|| value_string(item, "contentType"))
                    .unwrap_or_else(|| "movie".to_owned());
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "navigationRequested",
                    "route": "detail",
                    "params": { "id": id, "contentType": content_type },
                })) {
                    pending_effects.extend(update.effects);
                }
            }
            *status = format!("Selected {title}");
        }
        ui.add_space(7.0);
        ui.label(
            RichText::new(truncate_text(&title, 24))
                .size(14.0)
                .strong()
                .color(Color32::WHITE),
        );
        if !year.is_empty() {
            ui.label(
                RichText::new(year)
                    .size(12.0)
                    .color(Color32::from_white_alpha(125)),
            );
        }
    });
}

fn hero_meta_line(value: &Value) -> String {
    let kind = match value_string(value, "type").as_deref() {
        Some("series") | Some("tv") | Some("show") => "TV",
        Some("movie") => "Movie",
        Some("tv_movie") | Some("tvMovie") => "TV Movie",
        _ => "",
    };
    let year = value
        .get("year")
        .and_then(Value::as_i64)
        .map(|year| year.to_string())
        .or_else(|| value_string(value, "year"));
    let runtime = value_string(value, "runtime").or_else(|| {
        value
            .get("duration")
            .and_then(Value::as_i64)
            .filter(|duration| *duration > 0)
            .map(|duration| format!("{}min", duration / 60))
    });
    let genres = value
        .get("genres")
        .and_then(Value::as_array)
        .map(|genres| {
            genres
                .iter()
                .filter_map(Value::as_str)
                .take(2)
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .filter(|genres| !genres.is_empty());
    [Some(kind.to_owned()), genres, year, runtime]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn first_value_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| value_string(value, key))
}

fn native_wgpu_backends() -> wgpu::Backends {
    if let Some(backends) = fluxa_renderer::platform::backend_override(
        std::env::var("FLUXA_NATIVE_WGPU_BACKEND").ok().as_deref(),
    ) {
        return backends;
    }
    #[cfg(target_os = "linux")]
    {
        wgpu::Backends::VULKAN
    }
    #[cfg(not(target_os = "linux"))]
    {
        wgpu::Backends::all()
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    // The bundled KhooLy/mpv fork exposes Vulkan render-API import through
    // its gpu-next backend. The previous Tauri host set this before creating
    // any mpv handle; the native Rust shell must do the same.
    // SAFETY: this is the process entry point, before the event loop or any
    // worker thread has started reading the environment.
    unsafe { std::env::set_var("MPV_LIBMPV_RENDER_BACKEND", "gpu-next") };
    fluxa_mpv::set_error_reporter(|error| {
        log::error!("native mpv: {}", error.message);
        sentry::with_scope(
            |scope| {
                scope.set_tag("mpv.error_code", error.error_code);
                if let Some(url) = &error.url {
                    scope.set_extra("mpv.url", url.clone().into());
                }
                if !error.log_tail.is_empty() {
                    scope.set_extra("mpv.log_tail", error.log_tail.clone().into());
                }
            },
            || sentry::capture_message(&error.message, sentry::Level::Error),
        );
    });
    EventLoop::new()?.run_app(&mut FluxaDesktopApp::default())
}
