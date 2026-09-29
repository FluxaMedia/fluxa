use bytemuck::{Pod, Zeroable};
use egui::{Color32, Rect, Rgba};

const PANE_STRIDE: u64 = 256;
const LEVELS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlassStyle {
    pub radius: f32,
    pub tint: Color32,
    pub refraction: f32,
    pub bevel: f32,
    pub rim: f32,
}

pub(crate) struct GlassPane {
    pub rect: Rect,
    pub clip: Rect,
    pub style: GlassStyle,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PaneUniform {
    rect: [f32; 4],
    tint: [f32; 4],
    params: [f32; 4],
    screen: [f32; 4],
}

struct Level {
    view: wgpu::TextureView,
    bind: wgpu::BindGroup,
}

struct Targets {
    size: [u32; 2],
    scene: Level,
    down: Vec<Level>,
    up: Vec<Level>,
}

pub(crate) struct GlassPass {
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    pane_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    blit: wgpu::RenderPipeline,
    pane: wgpu::RenderPipeline,
    panes: Option<(wgpu::Buffer, wgpu::BindGroup, u64)>,
    targets: Option<Targets>,
}

impl GlassPass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("glass.wgsl"));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluxa-glass-source"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pane_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluxa-glass-pane"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(size_of::<PaneUniform>() as u64),
                },
                count: None,
            }],
        });
        let full_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluxa-glass-full"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pane_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluxa-glass-pane"),
            bind_group_layouts: &[Some(&layout), Some(&pane_layout)],
            immediate_size: 0,
        });
        let pipeline = |label, layout, vs, fs, topology, blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let list = wgpu::PrimitiveTopology::TriangleList;
        Self {
            format,
            down: pipeline("fluxa-glass-down", &full_layout, "vs_full", "fs_down", list, None),
            up: pipeline("fluxa-glass-up", &full_layout, "vs_full", "fs_up", list, None),
            blit: pipeline("fluxa-glass-blit", &full_layout, "vs_full", "fs_blit", list, None),
            pane: pipeline(
                "fluxa-glass-pane",
                &pane_pipeline_layout,
                "vs_pane",
                "fs_pane",
                wgpu::PrimitiveTopology::TriangleStrip,
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            ),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("fluxa-glass"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            layout,
            pane_layout,
            panes: None,
            targets: None,
        }
    }

    fn level(&self, device: &wgpu::Device, size: [u32; 2]) -> Level {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fluxa-glass-level"),
            size: wgpu::Extent3d {
                width: size[0].max(1),
                height: size[1].max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluxa-glass-level"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        Level { view, bind }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        pixels_per_point: f32,
        panes: &[GlassPane],
    ) {
        if self.targets.as_ref().is_none_or(|targets| targets.size != size) {
            let scaled = |shift: u32| [size[0] >> shift, size[1] >> shift];
            let scene = self.level(device, size);
            let down = (1..=LEVELS as u32)
                .map(|shift| self.level(device, scaled(shift)))
                .collect();
            let up = (1..LEVELS as u32)
                .map(|shift| self.level(device, scaled(shift)))
                .collect();
            self.targets = Some(Targets {
                size,
                scene,
                down,
                up,
            });
        }
        let needed = PANE_STRIDE * panes.len().max(1) as u64;
        if self.panes.as_ref().is_none_or(|(_, _, capacity)| *capacity < needed) {
            let capacity = needed.next_power_of_two();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fluxa-glass-panes"),
                size: capacity,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fluxa-glass-panes"),
                layout: &self.pane_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(size_of::<PaneUniform>() as u64),
                    }),
                }],
            });
            self.panes = Some((buffer, bind, capacity));
        }
        let mut bytes = vec![0u8; needed as usize];
        let srgb = self.format.is_srgb();
        for (index, pane) in panes.iter().enumerate() {
            let tint = if srgb {
                Rgba::from(pane.style.tint).to_array()
            } else {
                pane.style.tint.to_normalized_gamma_f32()
            };
            let px = |value: f32| value * pixels_per_point;
            let uniform = PaneUniform {
                rect: [
                    px(pane.rect.min.x),
                    px(pane.rect.min.y),
                    px(pane.rect.max.x),
                    px(pane.rect.max.y),
                ],
                tint,
                params: [
                    px(pane.style.radius),
                    px(pane.style.refraction),
                    px(pane.style.bevel),
                    pane.style.rim,
                ],
                screen: [size[0] as f32, size[1] as f32, pixels_per_point, 0.0],
            };
            let start = index * PANE_STRIDE as usize;
            bytes[start..start + size_of::<PaneUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&uniform));
        }
        if let Some((buffer, _, _)) = &self.panes {
            queue.write_buffer(buffer, 0, &bytes);
        }
    }

    pub fn scene(&self) -> &wgpu::TextureView {
        &self.targets.as_ref().expect("prepare before scene").scene.view
    }

    fn fullscreen(
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        source: &wgpu::BindGroup,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fluxa-glass-fullscreen"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, source, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn blur(&self, encoder: &mut wgpu::CommandEncoder) {
        let targets = self.targets.as_ref().expect("prepare before blur");
        let mut source = &targets.scene.bind;
        for level in &targets.down {
            Self::fullscreen(encoder, &self.down, source, &level.view);
            source = &level.bind;
        }
        for level in targets.up.iter().rev() {
            Self::fullscreen(encoder, &self.up, source, &level.view);
            source = &level.bind;
        }
    }

    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'static>,
        index: usize,
        clip: [u32; 4],
    ) {
        let (Some(targets), Some((_, panes, _))) = (&self.targets, &self.panes) else {
            return;
        };
        let blurred = targets.up.first().unwrap_or(&targets.down[0]);
        pass.set_viewport(0.0, 0.0, targets.size[0] as f32, targets.size[1] as f32, 0.0, 1.0);
        pass.set_scissor_rect(clip[0], clip[1], clip[2], clip[3]);
        pass.set_pipeline(&self.pane);
        pass.set_bind_group(0, &blurred.bind, &[]);
        pass.set_bind_group(1, panes, &[(index as u64 * PANE_STRIDE) as u32]);
        pass.draw(0..4, 0..1);
    }

    pub fn present(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let targets = self.targets.as_ref().expect("prepare before present");
        Self::fullscreen(encoder, &self.blit, &targets.scene.bind, target);
    }
}
