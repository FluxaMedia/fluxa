//! egui paint backend implemented with WGPU.
//!
//! Shared screens hand egui paint output to this adapter. Platform hosts keep
//! ownership of their native surface and frame lifecycle, but do not own or
//! configure egui-wgpu's renderer/pipelines directly.

use egui::epaint::ImageDelta;
use egui::{ClippedPrimitive, TextureId, TexturesDelta};
use egui_wgpu::{Renderer, RendererOptions};

pub use egui_wgpu::ScreenDescriptor;

pub struct EguiWgpuBackend {
    renderer: Renderer,
}

impl EguiWgpuBackend {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            renderer: Renderer::new(device, format, RendererOptions::default()),
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
