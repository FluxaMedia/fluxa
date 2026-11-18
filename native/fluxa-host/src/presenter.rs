use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use fluxa_renderer::egui_wgpu_backend::{EguiWgpuBackend, ScreenDescriptor};
use fluxa_renderer::glass::GlassStyle;

#[derive(Clone)]
pub(super) struct SharedBackend(Arc<Mutex<EguiWgpuBackend>>);

impl SharedBackend {
    pub(super) fn new(renderer: EguiWgpuBackend) -> Self {
        Self(Arc::new(Mutex::new(renderer)))
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, EguiWgpuBackend> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub(super) struct Frame {
    pub(super) deltas: egui::TexturesDelta,
    pub(super) jobs: Vec<egui::ClippedPrimitive>,
    pub(super) descriptor: ScreenDescriptor,
    pub(super) clear: wgpu::Color,
    pub(super) surface: Option<Arc<wgpu::Surface<'static>>>,
}

#[derive(Default)]
struct Pending {
    count: Mutex<usize>,
    idle: Condvar,
}

pub(super) struct Presenter {
    sender: SyncSender<Frame>,
    pending: Arc<Pending>,
    failure: Arc<Mutex<Option<String>>>,
}

impl Presenter {
    pub(super) fn spawn(
        device: wgpu::Device,
        queue: wgpu::Queue,
        renderer: SharedBackend,
        glass: fn(&egui::epaint::PaintCallback) -> Option<GlassStyle>,
    ) -> Self {
        let (sender, receiver) = sync_channel::<Frame>(0);
        let pending = Arc::new(Pending::default());
        let failure = Arc::new(Mutex::new(None));
        let worker_pending = pending.clone();
        let worker_failure = failure.clone();
        let _ = std::thread::Builder::new()
            .name("fluxa-present".to_owned())
            .spawn(move || {
                while let Ok(frame) = receiver.recv() {
                    if let Err(error) = present(&device, &queue, &renderer, glass, frame) {
                        *worker_failure
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner) = Some(error);
                    }
                    let mut count = worker_pending
                        .count
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner);
                    *count -= 1;
                    worker_pending.idle.notify_all();
                }
            });
        Self {
            sender,
            pending,
            failure,
        }
    }

    pub(super) fn submit(&self, frame: Frame) {
        *self
            .pending
            .count
            .lock()
            .unwrap_or_else(PoisonError::into_inner) += 1;
        if self.sender.send(frame).is_err() {
            *self
                .pending
                .count
                .lock()
                .unwrap_or_else(PoisonError::into_inner) -= 1;
        }
    }

    pub(super) fn flush(&self) {
        let mut count = self
            .pending
            .count
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while *count > 0 {
            count = self
                .pending
                .idle
                .wait(count)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    pub(super) fn take_failure(&self) -> Option<String> {
        self.failure
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }
}

fn present(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &SharedBackend,
    glass: fn(&egui::epaint::PaintCallback) -> Option<GlassStyle>,
    mut frame: Frame,
) -> Result<(), String> {
    let acquired = frame.surface.as_deref().map(acquire);
    let (texture, commands, failure) = {
        let mut renderer = renderer.lock();
        renderer.apply_texture_deltas(device, queue, &frame.deltas);
        match acquired {
            Some(Ok(texture)) => {
                let view = texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let commands = renderer.render_frame(
                    device,
                    queue,
                    &view,
                    frame.clear,
                    &frame.jobs,
                    &frame.descriptor,
                    glass,
                );
                (Some(texture), Some(commands), None)
            }
            Some(Err(error)) => (None, None, Some(error)),
            None => (None, None, None),
        }
    };
    if let Some(commands) = commands {
        let _queue = fluxa_renderer::queue_gate::guard();
        queue.submit(std::iter::once(commands));
    }
    if let Some(texture) = texture {
        let _queue = fluxa_renderer::queue_gate::guard();
        queue.present(texture);
    }
    renderer.lock().free_textures(&frame.deltas.free);
    frame.deltas.clear();
    failure.map_or(Ok(()), Err)
}

fn acquire(surface: &wgpu::Surface<'static>) -> Result<wgpu::SurfaceTexture, String> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(texture)
        | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(texture),
        wgpu::CurrentSurfaceTexture::Timeout => Err("surface timeout".to_owned()),
        wgpu::CurrentSurfaceTexture::Occluded => Err("surface occluded".to_owned()),
        wgpu::CurrentSurfaceTexture::Outdated => Err("surface outdated".to_owned()),
        wgpu::CurrentSurfaceTexture::Lost => Err("surface lost".to_owned()),
        wgpu::CurrentSurfaceTexture::Validation => Err("surface validation error".to_owned()),
    }
}
