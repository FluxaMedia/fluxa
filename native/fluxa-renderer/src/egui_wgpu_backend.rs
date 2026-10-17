//! egui paint backend implemented with WGPU.
//!
//! Shared screens hand egui paint output to this adapter. Platform hosts keep
//! ownership of their native surface and frame lifecycle, but do not own or
//! configure egui-wgpu's renderer/pipelines directly.

use std::hash::{BuildHasher, Hasher};
use std::ops::Range;

use egui::epaint::{ImageDelta, PaintCallback, Primitive};
use egui::{ClippedPrimitive, Rect, TextureId, TexturesDelta};
use egui_wgpu::{Renderer, RendererOptions};

use crate::glass::{GlassPane, GlassPass, GlassStyle};

pub use egui_wgpu::ScreenDescriptor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    pub geometry: u64,
    pub textures: u64,
}

pub fn fingerprint(
    paint_jobs: &[ClippedPrimitive],
    glass_of: impl Fn(&PaintCallback) -> Option<GlassStyle>,
) -> Option<Fingerprint> {
    let state = ahash::RandomState::with_seeds(0x51f0, 0x7c15, 0x9e37, 0x85eb);
    let mut geometry = state.build_hasher();
    let mut textures = state.build_hasher();
    let rect = |hasher: &mut ahash::AHasher, rect: Rect| {
        hasher.write(bytemuck::bytes_of(&[
            rect.min.x, rect.min.y, rect.max.x, rect.max.y,
        ]));
    };
    for job in paint_jobs {
        rect(&mut geometry, job.clip_rect);
        match &job.primitive {
            Primitive::Mesh(mesh) => {
                geometry.write_usize(mesh.vertices.len());
                geometry.write(bytemuck::cast_slice(&mesh.vertices));
                geometry.write(bytemuck::cast_slice(&mesh.indices));
                match mesh.texture_id {
                    TextureId::Managed(id) => textures.write_u64(id << 1),
                    TextureId::User(id) => textures.write_u64(id << 1 | 1),
                }
            }
            Primitive::Callback(callback) if crate::hole::is_hole(callback) => {
                rect(&mut geometry, callback.rect);
            }
            Primitive::Callback(callback) => {
                let style = glass_of(callback)?;
                rect(&mut geometry, callback.rect);
                geometry.write(bytemuck::bytes_of(&[
                    style.radius,
                    style.refraction,
                    style.bevel,
                    style.rim,
                ]));
                geometry.write(&style.tint.to_array());
            }
        }
    }
    Some(Fingerprint {
        geometry: geometry.finish(),
        textures: textures.finish(),
    })
}

pub struct EguiWgpuBackend {
    renderer: Renderer,
    glass: GlassPass,
}

impl EguiWgpuBackend {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let mut renderer = Renderer::new(device, format, RendererOptions::default());
        crate::hole::install(device, format, &mut renderer.callback_resources);
        Self {
            renderer,
            glass: GlassPass::new(device, format),
        }
    }

    pub fn update_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: TextureId,
        delta: &ImageDelta,
    ) {
        self.renderer.update_texture(device, queue, id, delta);
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        paint_jobs: &[ClippedPrimitive],
        descriptor: &ScreenDescriptor,
    ) {
        let _ = self
            .renderer
            .update_buffers(device, queue, encoder, paint_jobs, descriptor);
    }

    pub fn render(
        &mut self,
        pass: &mut wgpu::RenderPass<'static>,
        paint_jobs: &[ClippedPrimitive],
        descriptor: &ScreenDescriptor,
    ) {
        self.renderer.render(pass, paint_jobs, descriptor);
    }

    pub fn render_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        clear: wgpu::Color,
        paint_jobs: &[ClippedPrimitive],
        descriptor: &ScreenDescriptor,
        glass_of: impl Fn(&PaintCallback) -> Option<GlassStyle>,
    ) -> wgpu::CommandBuffer {
        let mut panes = Vec::new();
        let mut segments: Vec<(Range<usize>, Range<usize>)> = Vec::new();
        let mut start = 0;
        let mut first_pane = 0;
        for (index, job) in paint_jobs.iter().enumerate() {
            let Primitive::Callback(callback) = &job.primitive else {
                continue;
            };
            let Some(style) = glass_of(callback) else {
                continue;
            };
            if index > start {
                segments.push((start..index, first_pane..panes.len()));
                first_pane = panes.len();
            }
            start = index + 1;
            panes.push(GlassPane {
                rect: callback.rect,
                clip: job.clip_rect,
                style,
            });
        }
        segments.push((start..paint_jobs.len(), first_pane..panes.len()));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluxa-frame"),
        });
        if panes.is_empty() {
            self.update_buffers(device, queue, &mut encoder, paint_jobs, descriptor);
            let mut pass = begin(&mut encoder, target, wgpu::LoadOp::Clear(clear));
            self.renderer.render(&mut pass, paint_jobs, descriptor);
            drop(pass);
            return encoder.finish();
        }

        self.glass.prepare(
            device,
            queue,
            descriptor.size_in_pixels,
            descriptor.pixels_per_point,
            &panes,
        );
        drop(begin(
            &mut encoder,
            self.glass.scene(),
            wgpu::LoadOp::Clear(clear),
        ));
        let reach = BLUR_REACH_PX / descriptor.pixels_per_point + REFRACTION_REACH_PT;
        let backdrop = |pane: &GlassPane| pane.rect.intersect(pane.clip).expand(reach);
        let region = panes
            .iter()
            .map(backdrop)
            .reduce(Rect::union)
            .and_then(|rect| scissor(rect, descriptor));
        let mut painted: Vec<Rect> = Vec::new();
        let mut blurred = false;
        for (jobs, pane_range) in segments {
            let stale = !blurred
                || panes[pane_range.clone()].iter().any(|pane| {
                    let backdrop = backdrop(pane);
                    painted.iter().any(|rect| rect.intersects(backdrop))
                });
            if let Some(region) = region.filter(|_| stale && !pane_range.is_empty()) {
                self.glass.blur(&mut encoder, region);
                blurred = true;
                painted.clear();
            }
            painted.extend(panes[pane_range.clone()].iter().map(|pane| pane.rect));
            let jobs = &paint_jobs[jobs];
            painted.extend(jobs.iter().filter_map(bounds));
            self.update_buffers(device, queue, &mut encoder, jobs, descriptor);
            let mut pass = begin(&mut encoder, self.glass.scene(), wgpu::LoadOp::Load);
            for index in pane_range {
                if let Some(clip) = scissor(panes[index].clip, descriptor) {
                    self.glass.draw(&mut pass, index, clip);
                }
            }
            self.renderer.render(&mut pass, jobs, descriptor);
            drop(pass);
            let _queue = crate::queue_gate::guard();
            queue.submit([std::mem::replace(
                &mut encoder,
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("fluxa-frame"),
                }),
            )
            .finish()]);
        }
        self.glass.present(&mut encoder, target);
        encoder.finish()
    }

    pub fn free_texture(&mut self, id: &TextureId) {
        self.renderer.free_texture(id);
    }

    pub fn register_native_texture(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        filter: wgpu::FilterMode,
    ) -> TextureId {
        self.renderer.register_native_texture(device, view, filter)
    }

    pub fn register_mipmapped_texture(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> TextureId {
        self.renderer.register_native_texture_with_sampler_options(
            device,
            view,
            wgpu::SamplerDescriptor {
                label: Some("fluxa-mipmapped"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            },
        )
    }

    pub fn apply_texture_deltas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        deltas: &TexturesDelta,
    ) {
        for (id, delta) in &deltas.set {
            self.update_texture(device, queue, *id, delta);
        }
    }

    pub fn free_texture_deltas(&mut self, deltas: &TexturesDelta) {
        for id in &deltas.free {
            self.free_texture(id);
        }
    }
}

fn begin(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
) -> wgpu::RenderPass<'static> {
    encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fluxa-egui-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        })
        .forget_lifetime()
}

const BLUR_REACH_PX: f32 = 48.0;
const REFRACTION_REACH_PT: f32 = 24.0;

fn bounds(job: &ClippedPrimitive) -> Option<Rect> {
    let rect = match &job.primitive {
        Primitive::Mesh(mesh) => {
            let mut rect = Rect::NOTHING;
            for vertex in &mesh.vertices {
                rect.extend_with(vertex.pos);
            }
            rect
        }
        Primitive::Callback(callback) => callback.rect,
    }
    .intersect(job.clip_rect);
    rect.is_positive().then_some(rect)
}

fn scissor(clip: Rect, descriptor: &ScreenDescriptor) -> Option<[u32; 4]> {
    let [width, height] = descriptor.size_in_pixels;
    let ppp = descriptor.pixels_per_point;
    let x0 = ((clip.min.x * ppp).round().max(0.0) as u32).min(width);
    let y0 = ((clip.min.y * ppp).round().max(0.0) as u32).min(height);
    let x1 = ((clip.max.x * ppp).round().max(0.0) as u32).clamp(x0, width);
    let y1 = ((clip.max.y * ppp).round().max(0.0) as u32).clamp(y0, height);
    (x1 > x0 && y1 > y0).then(|| [x0, y0, x1 - x0, y1 - y0])
}
