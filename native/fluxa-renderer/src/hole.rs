use std::sync::Mutex;

use bytemuck::{Pod, Zeroable};
use egui::Rect;
use egui_wgpu::{Callback, CallbackResources, CallbackTrait, ScreenDescriptor};
use wgpu::util::DeviceExt;

pub fn shape(rect: Rect, radius: f32) -> egui::PaintCallback {
    Callback::new_paint_callback(
        rect,
        Hole {
            rect,
            radius,
            bind: Mutex::new(None),
        },
    )
}

pub fn is_hole(callback: &egui::PaintCallback) -> bool {
    callback.callback.is::<Callback>()
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniform {
    rect: [f32; 4],
    radius: [f32; 4],
}

struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

pub(crate) fn install(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    resources: &mut CallbackResources,
) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("hole.wgsl"));
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fluxa-hole"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(size_of::<Uniform>() as u64),
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fluxa-hole"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let erase = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fluxa-hole"),
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
                format,
                blend: Some(wgpu::BlendState {
                    color: erase,
                    alpha: erase,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    resources.insert(Pipeline { pipeline, layout });
}

struct Hole {
    rect: Rect,
    radius: f32,
    bind: Mutex<Option<wgpu::BindGroup>>,
}

impl CallbackTrait for Hole {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(pipeline) = resources.get::<Pipeline>() else {
            return Vec::new();
        };
        let scale = screen.pixels_per_point;
        let uniform = Uniform {
            rect: [
                self.rect.min.x * scale,
                self.rect.min.y * scale,
                self.rect.width() * scale,
                self.rect.height() * scale,
            ],
            radius: [self.radius * scale, 0.0, 0.0, 0.0],
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-hole"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluxa-hole"),
            layout: &pipeline.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        *self.bind.lock().unwrap() = Some(bind);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let (Some(pipeline), Some(bind)) =
            (resources.get::<Pipeline>(), &*self.bind.lock().unwrap())
        else {
            return;
        };
        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
