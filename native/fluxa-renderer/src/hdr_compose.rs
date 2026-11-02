use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdrEncoding {
    Pq,
    Scrgb,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    video_rect: [f32; 4],
    values: [f32; 4],
}

pub struct HdrCompose {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params: wgpu::Buffer,
    blank: wgpu::TextureView,
    encoding: HdrEncoding,
}

pub struct HdrFrame<'a> {
    pub ui: &'a wgpu::TextureView,
    pub ui_encoded: bool,
    pub video: Option<(&'a wgpu::TextureView, [f32; 4])>,
    pub sdr_white_nits: f32,
    pub target: &'a wgpu::TextureView,
}

impl HdrCompose {
    pub fn new(device: &wgpu::Device, output: wgpu::TextureFormat, encoding: HdrEncoding) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("hdr_compose.wgsl"));
        let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluxa-hdr-compose"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(size_of::<Params>() as u64),
                    },
                    count: None,
                },
                texture_entry(1),
                texture_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluxa-hdr-compose"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fluxa-hdr-compose"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fluxa-hdr-compose"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluxa-hdr-compose"),
            size: size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let blank = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("fluxa-hdr-compose-blank"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            pipeline,
            layout,
            sampler,
            params,
            blank,
            encoding,
        }
    }

    pub fn compose(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &HdrFrame,
    ) -> wgpu::CommandBuffer {
        let (video, video_rect) = match frame.video {
            Some((view, rect)) => (view, rect),
            None => (&self.blank, [0.0; 4]),
        };
        let params = Params {
            video_rect,
            values: [
                frame.sdr_white_nits,
                f32::from(self.encoding == HdrEncoding::Scrgb),
                f32::from(frame.video.is_some()),
                f32::from(frame.ui_encoded),
            ],
        };
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluxa-hdr-compose"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(frame.ui),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(video),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluxa-hdr-compose"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluxa-hdr-compose"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame.target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.finish()
    }
}

#[cfg(all(test, feature = "desktop-demo"))]
mod tests {
    use super::*;

    const PQ_203_NITS: f32 = 0.5807;

    fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .ok()?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
    }

    fn texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        pixels: &[u8],
    ) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 2,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: None,
            },
            texture.size(),
        );
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn run(encoding: HdrEncoding, ui: [u8; 8], video_code: u32) -> [[f32; 4]; 2] {
        let Some((device, queue)) = device() else {
            return [[f32::NAN; 4]; 2];
        };
        let ui = texture(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb, &ui);
        let code = video_code.to_le_bytes();
        let video = texture(
            &device,
            &queue,
            wgpu::TextureFormat::Rgb10a2Unorm,
            &[code, code].concat(),
        );
        let output = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 2,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target = output.create_view(&wgpu::TextureViewDescriptor::default());
        let compose = HdrCompose::new(&device, wgpu::TextureFormat::Rgba32Float, encoding);
        let commands = compose.compose(
            &device,
            &queue,
            &HdrFrame {
                ui: &ui,
                ui_encoded: false,
                video: Some((&video, [0.0, 0.0, 2.0, 1.0])),
                sdr_white_nits: 203.0,
                target: &target,
            },
        );
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            output.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: None,
                },
            },
            output.size(),
        );
        queue.submit([commands, encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = readback.slice(..).get_mapped_range().unwrap();
        let floats: &[f32] = bytemuck::cast_slice(&data[..32]);
        [
            [floats[0], floats[1], floats[2], floats[3]],
            [floats[4], floats[5], floats[6], floats[7]],
        ]
    }

    fn close(actual: f32, expected: f32, tolerance: f32) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn opaque_ui_white_lands_on_sdr_white_in_scrgb() {
        let pixels = run(HdrEncoding::Scrgb, [255; 8], 0);
        if pixels[0][0].is_nan() {
            return;
        }
        close(pixels[0][0], 203.0 / 80.0, 0.02);
        close(pixels[0][2], 203.0 / 80.0, 0.02);
    }

    #[test]
    fn transparent_ui_shows_pq_video_in_pq() {
        let code = (PQ_203_NITS * 1023.0).round() as u32;
        let video = code | (code << 10) | (code << 20) | (3 << 30);
        let pixels = run(HdrEncoding::Pq, [0; 8], video);
        if pixels[0][0].is_nan() {
            return;
        }
        for channel in 0..3 {
            close(pixels[0][channel], PQ_203_NITS, 0.01);
        }
    }
    #[test]
    fn translucent_ui_keeps_bright_video_from_washing_it_out() {
        let code = (0.7518 * 1023.0_f32).round() as u32;
        let video = code | (code << 10) | (code << 20) | (3 << 30);
        let pixels = run(HdrEncoding::Scrgb, [0, 0, 0, 128, 0, 0, 0, 128], video);
        if pixels[0][0].is_nan() {
            return;
        }
        close(pixels[0][0], 3.74, 0.25);
    }
}
