//! Small Vulkan/wgpu consumer for NVDEC's shared pitched NV12 output.
//!
//! The external semaphore wait is submitted on the render/UI thread, directly
//! before the corresponding wgpu compute submission. This keeps raw Vulkan
//! queue access serialized with this application's wgpu submissions; callers
//! must not invoke `submit_frame` concurrently from another thread.

#![allow(dead_code)]

use ash::vk;
use wgpu;
use wgpu::util::DeviceExt;

use crate::nvdec_vulkan_buffer::Nv12VulkanBuffer;

pub(crate) struct Nv12RgbaAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pipeline: wgpu::ComputePipeline,
    width: u32,
    height: u32,
}

impl Nv12RgbaAtlas {
    pub(crate) fn new(device: &wgpu::Device, width: u32, height: u32) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("NV12 atlas dimensions must be positive".into());
        }
        if width > device.limits().max_texture_dimension_2d
            || height > device.limits().max_texture_dimension_2d
        {
            return Err("NV12 atlas exceeds the wgpu texture dimension limit".into());
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("NVDEC animated WebP RGBA atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NV12 to RGBA compute"),
            source: wgpu::ShaderSource::Wgsl(include_str!("nv12_to_rgba.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("NV12 to RGBA atlas compute"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Self {
            texture,
            view,
            pipeline,
            width,
            height,
        })
    }

    /// Wait for CUDA's ready signal, convert/downscale one decoded frame, and
    /// signal CUDA that the shared NV12 allocation may be reused. Call only
    /// while wgpu submissions on this queue are serialized on the caller's
    /// thread.
    pub(crate) fn submit_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: &Nv12VulkanBuffer,
        destination_size: [u32; 2],
        tile_origin: [u32; 2],
        alpha_plane: Option<&[u8]>,
    ) -> Result<(), String> {
        let [destination_width, destination_height] = destination_size;
        let [tile_x, tile_y] = tile_origin;
        let tile_right = tile_x
            .checked_add(destination_width)
            .ok_or_else(|| "NV12 atlas tile overflows its width".to_owned())?;
        let tile_bottom = tile_y
            .checked_add(destination_height)
            .ok_or_else(|| "NV12 atlas tile overflows its height".to_owned())?;
        if destination_width == 0
            || destination_height == 0
            || tile_right > self.width
            || tile_bottom > self.height
        {
            return Err("NV12 atlas tile is outside the destination texture".into());
        }

        let hal_queue = unsafe { queue.as_hal::<wgpu::hal::api::Vulkan>() }
            .ok_or_else(|| "NVDEC compute queue is not backed by Vulkan".to_owned())?;
        wait_for_cuda_ready(queue, source.ready_semaphore.semaphore)?;

        // WGSL uniform structs have 16-byte alignment. Eight u32 words keep
        // the host buffer naturally aligned and the final word is padding.
        let values = [
            source.width,
            source.height,
            source.pitch,
            destination_width,
            destination_height,
            tile_x,
            tile_y,
            u32::from(alpha_plane.is_some()),
        ];
        let mut uniform_bytes = [0_u8; 32];
        for (index, value) in values.into_iter().enumerate() {
            uniform_bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NV12 conversion parameters"),
            contents: &uniform_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let alpha_bytes = alpha_plane.unwrap_or(&[0; 4]);
        let mut padded_alpha = alpha_bytes.to_vec();
        padded_alpha.resize(padded_alpha.len().next_multiple_of(4).max(4), 0);
        let alpha_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("WebP animation alpha plane"),
            contents: &padded_alpha,
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("NV12 conversion inputs"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: source.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: alpha_buffer.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("NV12 frame conversion"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("NV12 to RGBA"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                destination_width.div_ceil(8),
                destination_height.div_ceil(8),
                1,
            );
        }
        let command_buffer = encoder.finish();

        // wgpu-hal attaches this external signal to the next `queue.submit`.
        // Submit immediately so the CUDA producer cannot mistake another
        // submission for this frame's release operation.
        hal_queue.add_signal_semaphore(source.release_semaphore.semaphore, None);
        queue.submit([command_buffer]);
        Ok(())
    }

    pub(crate) fn discard_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: &Nv12VulkanBuffer,
    ) -> Result<(), String> {
        let hal_queue = unsafe { queue.as_hal::<wgpu::hal::api::Vulkan>() }
            .ok_or_else(|| "NVDEC compute queue is not backed by Vulkan".to_owned())?;
        wait_for_cuda_ready(queue, source.ready_semaphore.semaphore)?;
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("discard NVDEC frame and release shared buffer"),
        });
        hal_queue.add_signal_semaphore(source.release_semaphore.semaphore, None);
        queue.submit([encoder.finish()]);
        Ok(())
    }

    pub(crate) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(crate) fn size(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

fn wait_for_cuda_ready(queue: &wgpu::Queue, semaphore: vk::Semaphore) -> Result<(), String> {
    let hal_queue = unsafe { queue.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "NVDEC compute queue is not backed by Vulkan".to_owned())?;
    let wait_stage = [vk::PipelineStageFlags::COMPUTE_SHADER];
    let semaphores = [semaphore];
    let submit = vk::SubmitInfo::default()
        .wait_semaphores(&semaphores)
        .wait_dst_stage_mask(&wait_stage);
    unsafe {
        hal_queue
            .raw_device()
            .queue_submit(hal_queue.as_raw(), &[submit], vk::Fence::null())
    }
    .map_err(|error| format!("Vulkan could not wait for NVDEC frame readiness: {error}"))
}

#[cfg(test)]
mod tests {
    use super::wgpu;

    #[test]
    fn nv12_conversion_shader_is_valid_wgsl() {
        let source = include_str!("nv12_to_rgba.wgsl");
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("WGSL parse failed: {}", error.emit_to_string(source)));
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("NV12 conversion WGSL must validate");
    }
}
