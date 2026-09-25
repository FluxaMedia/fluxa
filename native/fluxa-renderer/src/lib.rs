#![forbid(unsafe_code)]

use std::{
    fmt,
    time::{Duration, Instant},
};

#[cfg(feature = "desktop-demo")]
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub mod egui_wgpu_backend;
pub mod platform;
pub mod style;
pub mod svg_icons;
pub mod ui;
pub use fluxa_theme::{Color, theme};

const QUAD_VERTICES: &[f32] = &[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const SHADER: &str = r#"
struct Globals {
    viewport: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct Instance {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) fill_color: vec4<f32>,
    @location(4) accent_color: vec4<f32>,
    @location(5) shape: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) fill_color: vec4<f32>,
    @location(2) accent_color: vec4<f32>,
    @location(3) shape: vec4<f32>,
};

@vertex
fn vs_main(@location(0) vertex: vec2<f32>, instance: Instance) -> VertexOutput {
    let pixel = instance.position + vertex * instance.size;
    let ndc = vec2<f32>(
        pixel.x / globals.viewport.x * 2.0 - 1.0,
        1.0 - pixel.y / globals.viewport.y * 2.0,
    );
    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 0.0, 1.0);
    output.local_position = vertex;
    output.fill_color = instance.fill_color;
    output.accent_color = instance.accent_color;
    output.shape = instance.shape;
    return output;
}

fn rounded_box_distance(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let safe_radius = min(radius, min(half_size.x, half_size.y));
    let distance_to_corner = abs(point) - half_size + vec2<f32>(safe_radius, safe_radius);
    return length(max(distance_to_corner, vec2<f32>(0.0, 0.0)))
        + min(max(distance_to_corner.x, distance_to_corner.y), 0.0)
        - safe_radius;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let radius = input.shape.x;
    let border_width = input.shape.y;
    let opacity = input.shape.z;
    let focus = input.shape.w;
    let point = (input.local_position - vec2<f32>(0.5, 0.5)) * 2.0;
    let distance_to_edge = rounded_box_distance(point, vec2<f32>(0.5, 0.5), radius);
    let edge_alpha = 1.0 - smoothstep(0.0, 0.015, distance_to_edge);
    let inner_alpha = 1.0 - smoothstep(-border_width, 0.0, distance_to_edge);
    let gradient = mix(input.fill_color, input.accent_color, input.local_position.y);
    let border = mix(input.accent_color, vec4<f32>(1.0, 1.0, 1.0, 1.0), focus);
    let color = mix(border, gradient, inner_alpha);
    return vec4<f32>(color.rgb, color.a * edge_alpha * opacity);
}
"#;

const BACKGROUND_SHADER: &str = r#"
@group(0) @binding(0)
var background_texture: texture_2d<f32>;

@group(0) @binding(1)
var background_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) vertex: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(vertex * 2.0 - vec2<f32>(1.0, 1.0), 0.0, 1.0);
    output.uv = vertex;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(background_texture, background_sampler, vec2<f32>(input.uv.x, 1.0 - input.uv.y));
    return vec4<f32>(color.rgb * 0.76, color.a);
}
"#;

fn ambient_light(u: f32, v: f32) -> f32 {
    let glow = |cx: f32, cy: f32, rx: f32, ry: f32| {
        let d = ((u - cx) / rx).powi(2) + ((v - cy) / ry).powi(2);
        (-d * 2.2).exp()
    };
    glow(0.18, -0.05, 0.75, 0.8) + 0.4 * glow(0.95, 1.05, 0.6, 0.6)
}

const AMBIENT_SIZE: (u32, u32) = (640, 360);

pub fn ambient_background() -> image::RgbaImage {
    let (width, height) = AMBIENT_SIZE;
    image::RgbaImage::from_fn(width, height, |x, y| {
        let light = 24.0 * ambient_light(x as f32 / width as f32, y as f32 / height as f32);
        let hash = (x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263)).wrapping_mul(1_274_126_177);
        let grain = ((hash >> 24) as f32 / 255.0 - 0.5) * 3.0;
        let value = (11.0 + light + grain).clamp(0.0, 255.0) as u8;
        image::Rgba([value, value, value, 255])
    })
}

pub fn ambient_glow() -> image::RgbaImage {
    let (width, height) = AMBIENT_SIZE;
    image::RgbaImage::from_fn(width, height, |x, y| {
        let light = ambient_light(x as f32 / width as f32, y as f32 / height as f32).min(1.0);
        image::Rgba([255, 255, 255, (light * 255.0) as u8])
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderNode {
    pub bounds: Rect,
    pub fill: Color,
    pub accent: Color,
    pub radius: f32,
    pub border_width: f32,
    pub opacity: f32,
    pub focus: f32,
    pub layer: i16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderScene {
    pub clear_color: Color,
    nodes: Vec<RenderNode>,
}

impl RenderScene {
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn push(&mut self, node: RenderNode) {
        self.nodes.push(node);
    }

    pub fn nodes(&self) -> &[RenderNode] {
        &self.nodes
    }

    pub fn sort(&mut self) {
        self.nodes.sort_unstable_by_key(|node| node.layer);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseOutCubic,
    EaseInOutCubic,
}

impl Easing {
    fn sample(self, value: f32) -> f32 {
        let value = value.clamp(0.0, 1.0);
        match self {
            Self::Linear => value,
            Self::EaseOutCubic => 1.0 - (1.0 - value).powi(3),
            Self::EaseInOutCubic => {
                if value < 0.5 {
                    4.0 * value.powi(3)
                } else {
                    1.0 - (-2.0 * value + 2.0).powi(3) / 2.0
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub elapsed: Duration,
    pub duration: Duration,
    pub easing: Easing,
}

impl Tween {
    pub fn new(from: f32, to: f32, duration: Duration, easing: Easing) -> Self {
        Self {
            from,
            to,
            elapsed: Duration::ZERO,
            duration,
            easing,
        }
    }

    pub fn tick(&mut self, delta: Duration) -> f32 {
        self.elapsed = (self.elapsed + delta).min(self.duration);
        let progress = if self.duration.is_zero() {
            1.0
        } else {
            self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
        };
        self.from + (self.to - self.from) * self.easing.sample(progress)
    }

    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Frame clock shared by native UI animation stores.
/// Delta time is capped so a paused/minimized window does not jump through an
/// entire transition when it becomes visible again.
#[derive(Debug)]
pub struct AnimationClock {
    last_frame: Instant,
    delta: Duration,
}

impl Default for AnimationClock {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: Duration::ZERO,
        }
    }
}

impl AnimationClock {
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        self.delta = now
            .saturating_duration_since(self.last_frame)
            .min(Duration::from_millis(100));
        self.last_frame = now;
        self.delta
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    viewport: [f32; 2],
    padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    position: [f32; 2],
    size: [f32; 2],
    fill_color: [f32; 4],
    accent_color: [f32; 4],
    shape: [f32; 4],
}

impl From<RenderNode> for Instance {
    fn from(node: RenderNode) -> Self {
        Self {
            position: [node.bounds.x, node.bounds.y],
            size: [node.bounds.width.max(0.0), node.bounds.height.max(0.0)],
            fill_color: [node.fill.r, node.fill.g, node.fill.b, node.fill.a],
            accent_color: [node.accent.r, node.accent.g, node.accent.b, node.accent.a],
            shape: [
                node.radius,
                node.border_width,
                node.opacity.clamp(0.0, 1.0),
                node.focus.clamp(0.0, 1.0),
            ],
        }
    }
}

#[derive(Debug)]
pub enum RendererError {
    Adapter(String),
    Device(String),
    SurfaceFormat,
    Asset(String),
    Surface(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(error) => write!(formatter, "renderer adapter: {error}"),
            Self::Device(error) => write!(formatter, "renderer device: {error}"),
            Self::SurfaceFormat => formatter.write_str("renderer surface has no compatible format"),
            Self::Asset(error) => write!(formatter, "renderer asset: {error}"),
            Self::Surface(error) => write!(formatter, "renderer surface: {error}"),
        }
    }
}

impl std::error::Error for RendererError {}

#[cfg(feature = "desktop-demo")]
pub struct WgpuRenderer {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    background_pipeline: wgpu::RenderPipeline,
    background_bind_group: wgpu::BindGroup,
    _background_texture: wgpu::Texture,
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
}

#[cfg(feature = "desktop-demo")]
impl WgpuRenderer {
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        let size = window.inner_size();
        let mut instance_descriptor =
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(window.clone()));
        instance_descriptor.backends = wgpu::Backends::all();
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| RendererError::Adapter(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| RendererError::Adapter(error.to_string()))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("fluxa-native-renderer-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            })
            .await
            .map_err(|error| RendererError::Device(error.to_string()))?;
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(RendererError::SurfaceFormat)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let background_image = ambient_background();
        let background_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fluxa-native-renderer-background"),
            size: wgpu::Extent3d {
                width: background_image.width(),
                height: background_image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &background_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &background_image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * background_image.width()),
                rows_per_image: Some(background_image.height()),
            },
            wgpu::Extent3d {
                width: background_image.width(),
                height: background_image.height(),
                depth_or_array_layers: 1,
            },
        );
        let background_view =
            background_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let background_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fluxa-native-renderer-background-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let background_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fluxa-native-renderer-background-shader"),
            source: wgpu::ShaderSource::Wgsl(BACKGROUND_SHADER.into()),
        });
        let background_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluxa-native-renderer-background-layout"),
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
        let background_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluxa-native-renderer-background-bind-group"),
            layout: &background_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&background_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&background_sampler),
                },
            ],
        });
        let background_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fluxa-native-renderer-background-pipeline-layout"),
                bind_group_layouts: &[Some(&background_layout)],
                immediate_size: 0,
            });
        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fluxa-native-renderer-background-pipeline"),
            layout: Some(&background_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &background_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &background_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fluxa-native-renderer-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-native-renderer-globals"),
            contents: bytemuck::bytes_of(&Globals {
                viewport: [config.width as f32, config.height as f32],
                padding: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluxa-native-renderer-globals-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluxa-native-renderer-globals-bind-group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluxa-native-renderer-pipeline-layout"),
            bind_group_layouts: &[Some(&globals_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fluxa-native-renderer-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x2,
                            2 => Float32x2,
                            3 => Float32x4,
                            4 => Float32x4,
                            5 => Float32x4,
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-native-renderer-quad-vertices"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-native-renderer-quad-indices"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_capacity = 64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluxa-native-renderer-instances"),
            size: (std::mem::size_of::<Instance>() * instance_capacity) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            background_pipeline,
            background_bind_group,
            _background_texture: background_texture,
            pipeline,
            globals_buffer,
            globals_bind_group,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_capacity,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::bytes_of(&Globals {
                viewport: [self.config.width as f32, self.config.height as f32],
                padding: [0.0, 0.0],
            }),
        );
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn render(&mut self, scene: &RenderScene) -> Result<(), RendererError> {
        let instances: Vec<Instance> = scene.nodes.iter().copied().map(Instance::from).collect();
        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fluxa-native-renderer-instances-grown"),
                size: (std::mem::size_of::<Instance>() * self.instance_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !instances.is_empty() {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendererError::Surface(
                    "surface validation failure".to_owned(),
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fluxa-native-renderer-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluxa-native-renderer-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: scene.clear_color.r as f64,
                            g: scene.clear_color.g as f64,
                            b: scene.clear_color.b as f64,
                            a: scene.clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.background_pipeline);
            pass.set_bind_group(0, &self.background_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..instances.len() as u32);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

/// A scene pass that can be composed into an application's existing WGPU
/// surface. Unlike [`WgpuRenderer`], it does not own a window or surface; the
/// host can run it before another renderer (egui, video, or overlays) in the
/// same command encoder.
pub struct SceneRenderer {
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SceneRenderStats {
    pub instance_count: usize,
    pub instance_upload_bytes: usize,
    pub prepare_ms: f64,
    pub upload_ms: f64,
    pub encode_ms: f64,
}

impl SceneRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fluxa-scene-pass-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-scene-pass-globals"),
            contents: bytemuck::bytes_of(&Globals {
                viewport: [1.0, 1.0],
                padding: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluxa-scene-pass-globals-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fluxa-scene-pass-globals-bind-group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluxa-scene-pass-pipeline-layout"),
            bind_group_layouts: &[Some(&globals_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fluxa-scene-pass-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x2,
                            2 => Float32x2,
                            3 => Float32x4,
                            4 => Float32x4,
                            5 => Float32x4,
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-scene-pass-quad-vertices"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fluxa-scene-pass-quad-indices"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_capacity = 64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fluxa-scene-pass-instances"),
            size: (std::mem::size_of::<Instance>() * instance_capacity) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            globals_buffer,
            globals_bind_group,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_capacity,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        scene: &RenderScene,
        viewport: [u32; 2],
    ) -> SceneRenderStats {
        let prepare_started = Instant::now();
        let instances: Vec<Instance> = scene.nodes().iter().copied().map(Instance::from).collect();
        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fluxa-scene-pass-instances-grown"),
                size: (std::mem::size_of::<Instance>() * self.instance_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1_000.0;
        let instance_upload_bytes = std::mem::size_of_val(instances.as_slice());
        let upload_started = Instant::now();
        queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::bytes_of(&Globals {
                viewport: [viewport[0].max(1) as f32, viewport[1].max(1) as f32],
                padding: [0.0, 0.0],
            }),
        );
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
        let upload_ms = upload_started.elapsed().as_secs_f64() * 1_000.0;
        let encode_started = Instant::now();
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.globals_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..instances.len() as u32);
        SceneRenderStats {
            instance_count: instances.len(),
            instance_upload_bytes,
            prepare_ms,
            upload_ms,
            encode_ms: encode_started.elapsed().as_secs_f64() * 1_000.0,
        }
    }
}

pub fn sample_home_scene(time_seconds: f32, focused_card: usize) -> RenderScene {
    sample_home_scene_for_viewport(time_seconds, focused_card, 1280, 800)
}

pub fn sample_home_scene_for_viewport(
    time_seconds: f32,
    focused_card: usize,
    viewport_width: u32,
    viewport_height: u32,
) -> RenderScene {
    let scale_x = viewport_width.max(1) as f32 / 1280.0;
    let scale_y = viewport_height.max(1) as f32 / 800.0;
    let scale = |rect: Rect| {
        Rect::new(
            rect.x * scale_x,
            rect.y * scale_y,
            rect.width * scale_x,
            rect.height * scale_y,
        )
    };
    let mut scene = RenderScene {
        clear_color: Color::rgb(0.015, 0.015, 0.02),
        nodes: Vec::with_capacity(32),
    };
    scene.push(RenderNode {
        bounds: scale(Rect::new(0.0, 0.0, 224.0, 800.0)),
        fill: Color::rgba(0.025, 0.028, 0.04, 0.96),
        accent: Color::rgba(0.11, 0.12, 0.17, 0.96),
        radius: 0.0,
        border_width: 0.0,
        opacity: 1.0,
        focus: 0.0,
        layer: 0,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(28.0, 26.0, 148.0, 42.0)),
        fill: Color::rgba(0.82, 0.18, 0.10, 0.95),
        accent: Color::rgba(0.98, 0.50, 0.18, 0.95),
        radius: 0.08,
        border_width: 0.01,
        opacity: 1.0,
        focus: 0.0,
        layer: 0,
    });
    for index in 0..5 {
        scene.push(RenderNode {
            bounds: scale(Rect::new(24.0, 132.0 + index as f32 * 58.0, 176.0, 40.0)),
            fill: if index == 0 {
                Color::rgba(0.84, 0.22, 0.14, 0.88)
            } else {
                Color::rgba(0.09, 0.10, 0.14, 0.72)
            },
            accent: if index == 0 {
                Color::rgba(0.98, 0.46, 0.16, 0.92)
            } else {
                Color::rgba(0.16, 0.18, 0.24, 0.80)
            },
            radius: 0.08,
            border_width: 0.008,
            opacity: 1.0,
            focus: 0.0,
            layer: 1,
        });
    }
    scene.push(RenderNode {
        bounds: scale(Rect::new(252.0, 26.0, 392.0, 42.0)),
        fill: Color::rgba(0.06, 0.07, 0.10, 0.86),
        accent: Color::rgba(0.16, 0.18, 0.25, 0.86),
        radius: 0.08,
        border_width: 0.008,
        opacity: 1.0,
        focus: 0.0,
        layer: 1,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(1084.0, 26.0, 156.0, 42.0)),
        fill: Color::rgba(0.07, 0.08, 0.11, 0.90),
        accent: Color::rgba(0.16, 0.18, 0.25, 0.90),
        radius: 0.08,
        border_width: 0.008,
        opacity: 1.0,
        focus: 0.0,
        layer: 1,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(252.0, 96.0, 988.0, 296.0)),
        fill: Color::rgba(0.025, 0.03, 0.05, 0.72),
        accent: Color::rgba(0.20, 0.28, 0.48, 0.68),
        radius: 0.05,
        border_width: 0.012,
        opacity: 1.0,
        focus: 0.0,
        layer: 1,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(292.0, 142.0, 300.0, 48.0)),
        fill: Color::rgba(0.95, 0.95, 0.98, 0.92),
        accent: Color::rgba(0.66, 0.68, 0.74, 0.92),
        radius: 0.04,
        border_width: 0.0,
        opacity: 1.0,
        focus: 0.0,
        layer: 2,
    });
    for index in 0..3 {
        scene.push(RenderNode {
            bounds: scale(Rect::new(
                292.0,
                218.0 + index as f32 * 24.0,
                260.0 - index as f32 * 38.0,
                10.0,
            )),
            fill: Color::rgba(0.72, 0.74, 0.80, 0.62),
            accent: Color::rgba(0.35, 0.38, 0.48, 0.62),
            radius: 0.04,
            border_width: 0.0,
            opacity: 1.0,
            focus: 0.0,
            layer: 2,
        });
    }
    scene.push(RenderNode {
        bounds: scale(Rect::new(292.0, 304.0, 112.0, 40.0)),
        fill: Color::rgba(0.90, 0.24, 0.14, 0.96),
        accent: Color::rgba(1.0, 0.52, 0.20, 0.96),
        radius: 0.08,
        border_width: 0.01,
        opacity: 1.0,
        focus: 0.0,
        layer: 2,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(416.0, 304.0, 124.0, 40.0)),
        fill: Color::rgba(0.12, 0.14, 0.20, 0.88),
        accent: Color::rgba(0.28, 0.32, 0.42, 0.88),
        radius: 0.08,
        border_width: 0.01,
        opacity: 1.0,
        focus: 0.0,
        layer: 2,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(870.0, 120.0, 300.0, 220.0)),
        fill: Color::rgba(0.10, 0.13, 0.22, 0.34),
        accent: Color::rgba(0.38, 0.18, 0.22, 0.56),
        radius: 0.06,
        border_width: 0.008,
        opacity: 1.0,
        focus: 0.0,
        layer: 2,
    });
    scene.push(RenderNode {
        bounds: scale(Rect::new(252.0, 424.0, 196.0, 28.0)),
        fill: Color::rgba(0.92, 0.93, 0.96, 0.90),
        accent: Color::rgba(0.52, 0.55, 0.64, 0.90),
        radius: 0.04,
        border_width: 0.0,
        opacity: 1.0,
        focus: 0.0,
        layer: 2,
    });
    for index in 0..5 {
        let wave = ((time_seconds * 1.4 + index as f32 * 0.38).sin() + 1.0) * 0.5;
        scene.push(RenderNode {
            bounds: scale(Rect::new(
                252.0 + index as f32 * 192.0,
                470.0 - wave * 5.0,
                174.0,
                252.0,
            )),
            fill: Color::rgba(0.06 + index as f32 * 0.018, 0.08, 0.13, 0.86),
            accent: Color::rgba(
                0.20 + wave * 0.18,
                0.25 + wave * 0.10,
                0.48 + wave * 0.18,
                0.92,
            ),
            radius: 0.045,
            border_width: if index == focused_card { 0.026 } else { 0.008 },
            opacity: 1.0,
            focus: if index == focused_card { 1.0 } else { 0.0 },
            layer: 3,
        });
        scene.push(RenderNode {
            bounds: scale(Rect::new(268.0 + index as f32 * 192.0, 744.0, 100.0, 8.0)),
            fill: Color::rgba(0.82, 0.83, 0.88, 0.75),
            accent: Color::rgba(0.36, 0.39, 0.50, 0.75),
            radius: 0.02,
            border_width: 0.0,
            opacity: 1.0,
            focus: 0.0,
            layer: 4,
        });
    }
    scene.sort();
    scene
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_reaches_target() {
        let mut tween = Tween::new(0.0, 1.0, Duration::from_millis(200), Easing::EaseOutCubic);

        assert_eq!(tween.tick(Duration::from_millis(200)), 1.0);
        assert!(tween.finished());
    }

    #[test]
    fn scene_sort_keeps_painter_order() {
        let mut scene = RenderScene::default();
        scene.push(RenderNode {
            layer: 4,
            ..RenderNode::default()
        });
        scene.push(RenderNode {
            layer: 1,
            ..RenderNode::default()
        });

        scene.sort();

        assert_eq!(scene.nodes()[0].layer, 1);
        assert_eq!(scene.nodes()[1].layer, 4);
    }
}
