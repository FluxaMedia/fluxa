use crate::DesktopState;
use crate::mpv_render::VulkanTargetImage;
use crate::vulkan::VulkanContext;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub(crate) struct VulkanShared {
    pub(crate) width: AtomicI32,
    pub(crate) height: AtomicI32,
    pub(crate) hdr: AtomicBool,
    pub(crate) mpv_context_ready: AtomicBool,
    pub(crate) load_in_progress: AtomicBool,
}

const PRESENT_WATCHDOG_INTERVAL: Duration = Duration::from_millis(200);

pub(crate) fn spawn_vulkan_render_thread(
    app: AppHandle,
    mut ctx: VulkanContext,
    shared: Arc<VulkanShared>,
) {
    std::thread::Builder::new()
        .name("vulkan-player-render".into())
        .spawn(move || {
            let mut last_present = Instant::now();
            let mut last_context_error: Option<String> = None;
            let mut last_render_error: Option<String> = None;
            let mut last_presented_extents: Option<(u32, u32, u32, u32)> = None;
            loop {
                let width = shared.width.load(Ordering::Acquire);
                let height = shared.height.load(Ordering::Acquire);
                if width > 1
                    && height > 1
                    && std::env::var_os("FLUXA_NATIVE_AUTOTEST_SKIP_RESIZE").is_none()
                {
                    if let Err(error) = ctx.resize(width, height) {
                        log::warn!("linux_native_render: Vulkan resize failed: {error}");
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
                if shared.load_in_progress.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(2));
                    continue;
                }
                let forced = last_present.elapsed() >= PRESENT_WATCHDOG_INTERVAL;
                shared.hdr.store(ctx.is_hdr(), Ordering::Release);
                let state = app.state::<DesktopState>();
                let mut wire_error = false;
                let ready = {
                    let Ok(mut guard) = state.player_render_state.try_lock() else {
                        std::thread::sleep(Duration::from_millis(2));
                        continue;
                    };
                    match guard.as_mut() {
                        None => {
                            shared.mpv_context_ready.store(false, Ordering::Release);
                            false
                        }
                        Some(renderer) => {
                            if renderer.needs_vulkan_context() {
                                shared.mpv_context_ready.store(false, Ordering::Release);
                                let (instance, phys_device, device, queue_index, queue_count, get_proc_addr) = ctx.device_handles();
                                let extensions = ctx.enabled_device_extension_ptrs();
                                match renderer.create_vulkan_context(instance, phys_device, device, queue_index, queue_count, get_proc_addr, &extensions) {
                                    Ok(()) => {
                                        last_context_error = None;
                                        shared.mpv_context_ready.store(true, Ordering::Release);
                                    }
                                    Err(error) => {
                                        if last_context_error.as_deref() != Some(error.as_str()) {
                                            log::error!("linux_native_render: mpv Vulkan context failed: {error}");
                                            crate::diagnostics::report(&app, format!("linux_native_render: mpv Vulkan context failed: {error}"), sentry::Level::Error);
                                            last_context_error = Some(error);
                                        }
                                        wire_error = true;
                                    }
                                }
                            } else {
                                shared.mpv_context_ready.store(true, Ordering::Release);
                            }
                            !wire_error && (forced || renderer.vulkan_frame_ready())
                        }
                    }
                };
                if wire_error || !ready {
                    std::thread::sleep(Duration::from_millis(if wire_error { 250 } else { 4 }));
                    continue;
                }
                let image_usage = ctx.image_usage();
                let result = ctx.render_and_present(
                    |image, format, width, height, layout, wait, signal| {
                    let mut guard = state.player_render_state.lock().map_err(|_| "player renderer lock poisoned".to_string())?;
                    let renderer = guard.as_mut().ok_or_else(|| "player renderer destroyed".to_string())?;
                    let mut target = VulkanTargetImage {
                        image,
                        format,
                        w: width as i32,
                        h: height as i32,
                        usage: image_usage,
                        layout,
                        wait_semaphore: wait,
                        signal_semaphore: signal,
                    };
                    renderer.render_vulkan_frame(&mut target).map(|_| target.layout)
                },
                );
                match result {
                    Ok(()) => {
                        last_present = Instant::now();
                        last_render_error = None;
                        let extents = ctx.debug_extents();
                        if last_presented_extents != Some(extents) {
                            log::info!(
                                "linux_native_render: frame presented requested={}x{} surface={}x{}",
                                extents.0,
                                extents.1,
                                extents.2,
                                extents.3
                            );
                            last_presented_extents = Some(extents);
                        }
                        if let Ok(mut guard) = state.player_render_state.lock() {
                            if let Some(renderer) = guard.as_mut() {
                                renderer.report_swap();
                            }
                        }
                    }
                    Err(error) => {
                        if last_render_error.as_deref() != Some(error.as_str()) {
                            log::warn!("linux_native_render: Vulkan render failed: {error}");
                            crate::diagnostics::report(&app, format!("linux_native_render: Vulkan render failed: {error}"), sentry::Level::Error);
                            last_render_error = Some(error);
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        })
        .expect("failed to spawn Vulkan render thread");
}
