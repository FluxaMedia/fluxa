//! Linux/Vulkan allocation shared with CUDA for one pitched NV12 frame.
//!
//! This module is used by the Linux animated-WebP NVDEC path. It assumes the
//! `wgpu::Device` was opened with
//! `VK_KHR_external_memory_fd` enabled (as Fluxa's Linux device setup does).

use std::{
    ffi::CStr,
    os::fd::{FromRawFd, OwnedFd},
};

use ash::{Device as AshDevice, Instance as AshInstance, vk};
use wgpu;

const EXTERNAL_MEMORY_FD: &CStr = c"VK_KHR_external_memory_fd";
const EXTERNAL_SEMAPHORE_FD: &CStr = c"VK_KHR_external_semaphore_fd";
const DEFAULT_PITCH_ALIGNMENT: u32 = 256;

/// The same allocation is visible to CUDA via [`Self::cuda_fd`] and to wgpu
/// as a read-only storage buffer. `cuda_fd` is an owned, export-created FD.
/// The CUDA interop layer imports a duplicate, so this original descriptor
/// remains owned here and is closed when the allocation is dropped.
///
/// Field order is deliberate: drop the wgpu object before closing the export
/// FD. The wgpu buffer owns and destroys the Vulkan buffer and Vulkan memory.
pub struct Nv12VulkanBuffer {
    pub buffer: wgpu::Buffer,
    pub cuda_fd: OwnedFd,
    pub ready_semaphore: ExternalBinarySemaphore,
    pub release_semaphore: ExternalBinarySemaphore,
    pub pitch: u32,
    /// Allocated Vulkan buffer size, including row padding.
    pub size: u64,
    /// Size of the Vulkan allocation exported to CUDA (may exceed `size`).
    pub allocation_size: u64,
    pub width: u32,
    pub height: u32,
}

/// Exportable Vulkan binary semaphore used for one direction of CUDA interop.
/// The ash device is a lightweight function-table clone; its Vulkan device is
/// still owned by wgpu and must outlive this guard.
pub struct ExternalBinarySemaphore {
    device: AshDevice,
    pub semaphore: vk::Semaphore,
    pub cuda_fd: OwnedFd,
}

impl Drop for ExternalBinarySemaphore {
    fn drop(&mut self) {
        unsafe { self.device.destroy_semaphore(self.semaphore, None) };
    }
}

/// Allocate an opaque-FD-exportable buffer for an NV12 frame and wrap its
/// `VkBuffer` in this wgpu device as `STORAGE`.
///
/// Rows are padded to 256 bytes by default. `pitch_alignment` must be a
/// non-zero power of two and can be increased if the CUDA/NVDEC importer has
/// a stricter pitch requirement. Odd frame dimensions are padded at the right
/// and bottom edges in the NV12 plane layout.
///
/// The returned buffer layout is `height` rows of Y followed by `ceil(height / 2)`
/// rows of interleaved UV, each with `pitch` bytes. The buffer size is
/// `pitch * (height + ceil(height / 2))`.
///
/// # Synchronization
/// Exporting memory does not synchronize CUDA and Vulkan. The CUDA producer
/// must signal an external semaphore after writing the NV12 planes; wgpu
/// compute must wait for that signal before reading. Conversely, before CUDA
/// reuses/overwrites this allocation, it must wait for Vulkan's completion
/// semaphore. Do not rely on CPU submission order or on the FD itself for
/// visibility or execution ordering.
pub fn allocate_nv12_buffer(
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
    pitch_alignment: u32,
) -> Result<Nv12VulkanBuffer, String> {
    let (pitch, size) = nv12_layout(width, height, pitch_alignment)?;

    // HAL access is unsafe because it exposes backend-owned objects. We only
    // borrow those objects and create/destroy resources through their device.
    let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu device is not backed by Vulkan HAL".to_owned())?;
    let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu adapter is not backed by Vulkan HAL".to_owned())?;

    let physical_device = hal_adapter.raw_physical_device();
    if physical_device != hal_device.raw_physical_device() {
        return Err("wgpu adapter and device refer to different Vulkan physical devices".into());
    }

    let instance = hal_adapter.shared_instance().raw_instance();
    let ash_device = hal_device.raw_device();
    require_external_fd_extension(hal_device.enabled_device_extensions())?;
    if !hal_device
        .enabled_device_extensions()
        .iter()
        .any(|extension| *extension == EXTERNAL_SEMAPHORE_FD)
    {
        return Err(format!(
            "wgpu Vulkan device did not enable {}",
            EXTERNAL_SEMAPHORE_FD.to_string_lossy()
        ));
    }

    let max_size = device.limits().max_buffer_size;
    if size > max_size {
        return Err(format!(
            "NV12 buffer size {size} exceeds wgpu device max_buffer_size {max_size}"
        ));
    }
    let max_storage_binding_size = u64::from(device.limits().max_storage_buffer_binding_size);
    if size > max_storage_binding_size {
        return Err(format!(
            "NV12 buffer size {size} exceeds wgpu max_storage_buffer_binding_size {max_storage_binding_size}"
        ));
    }

    let dedicated_only = verify_exportable_buffer(instance, physical_device)?;

    let ready_semaphore = create_external_binary_semaphore(instance, physical_device, ash_device)?;
    let release_semaphore =
        create_external_binary_semaphore(instance, physical_device, ash_device)?;

    let vk_buffer = create_external_buffer(ash_device, size)?;
    let mut guard = VulkanBufferGuard::new(ash_device, vk_buffer);

    let requirements = unsafe { ash_device.get_buffer_memory_requirements(vk_buffer) };
    if requirements.size < size {
        return Err(format!(
            "Vulkan returned an invalid allocation requirement: {} < requested {size}",
            requirements.size
        ));
    }
    let memory_type_index =
        choose_memory_type(instance, physical_device, requirements.memory_type_bits)?;

    let memory = allocate_exportable_memory(
        ash_device,
        vk_buffer,
        requirements,
        memory_type_index,
        dedicated_only,
    )?;
    guard.memory = Some(memory);

    unsafe {
        ash_device
            .bind_buffer_memory(vk_buffer, memory, 0)
            .map_err(|error| {
                format!("failed to bind external Vulkan memory to NV12 buffer: {error}")
            })?;
    }

    let export_loader = ash::khr::external_memory_fd::Device::new(instance, ash_device);
    let fd_info = vk::MemoryGetFdInfoKHR::default()
        .memory(memory)
        .handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
    let raw_fd = unsafe { export_loader.get_memory_fd(&fd_info) }
        .map_err(|error| format!("failed to export Vulkan memory as opaque FD: {error}"))?;
    if raw_fd < 0 {
        return Err(format!("Vulkan returned invalid exported FD {raw_fd}"));
    }
    // vkGetMemoryFdKHR returns a newly owned descriptor on success.
    let cuda_fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

    let descriptor = wgpu::BufferDescriptor {
        label: Some("NVDEC shared NV12 frame"),
        size,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    };

    // Transfer destruction responsibility to wgpu-hal. `from_raw_managed`
    // retains the allocation size from Vulkan's memory requirements so HAL
    // can destroy both raw handles when the wgpu buffer is dropped.
    let (vk_buffer, memory, allocation_size) = guard.transfer();
    let hal_buffer = unsafe {
        wgpu::hal::vulkan::Buffer::from_raw_managed(vk_buffer, memory, 0, allocation_size)
    };
    // SAFETY: this VkBuffer was created from this device's ash handle with
    // STORAGE_BUFFER usage, bound to the managed allocation above, and sized
    // exactly as in the descriptor. It is not mapped or used for queue copies.
    let buffer =
        unsafe { device.create_buffer_from_hal::<wgpu::hal::api::Vulkan>(hal_buffer, &descriptor) };

    Ok(Nv12VulkanBuffer {
        buffer,
        cuda_fd,
        ready_semaphore,
        release_semaphore,
        pitch,
        size,
        allocation_size,
        width,
        height,
    })
}

fn create_external_binary_semaphore(
    instance: &AshInstance,
    physical_device: vk::PhysicalDevice,
    device: &AshDevice,
) -> Result<ExternalBinarySemaphore, String> {
    let handle_type = vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD;
    let external_info = vk::PhysicalDeviceExternalSemaphoreInfo::default().handle_type(handle_type);
    let mut properties = vk::ExternalSemaphoreProperties::default();
    unsafe {
        instance.get_physical_device_external_semaphore_properties(
            physical_device,
            &external_info,
            &mut properties,
        );
    }
    let supported = properties.external_semaphore_features;
    if !supported.contains(vk::ExternalSemaphoreFeatureFlags::EXPORTABLE)
        || !supported.contains(vk::ExternalSemaphoreFeatureFlags::IMPORTABLE)
        || !properties.compatible_handle_types.contains(handle_type)
    {
        return Err("Vulkan device cannot import/export OPAQUE_FD binary semaphores".into());
    }

    let mut export_info = vk::ExportSemaphoreCreateInfo::default().handle_types(handle_type);
    let create_info = vk::SemaphoreCreateInfo::default().push_next(&mut export_info);
    let semaphore = unsafe { device.create_semaphore(&create_info, None) }
        .map_err(|error| format!("failed to create external Vulkan semaphore: {error}"))?;
    let export_loader = ash::khr::external_semaphore_fd::Device::new(instance, device);
    let fd_info = vk::SemaphoreGetFdInfoKHR::default()
        .semaphore(semaphore)
        .handle_type(handle_type);
    let raw_fd = match unsafe { export_loader.get_semaphore_fd(&fd_info) } {
        Ok(fd) if fd >= 0 => fd,
        Ok(fd) => {
            unsafe { device.destroy_semaphore(semaphore, None) };
            return Err(format!("Vulkan returned invalid semaphore FD {fd}"));
        }
        Err(error) => {
            unsafe { device.destroy_semaphore(semaphore, None) };
            return Err(format!("failed to export Vulkan semaphore FD: {error}"));
        }
    };
    // vkGetSemaphoreFdKHR returns a newly owned descriptor on success.
    let cuda_fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
    Ok(ExternalBinarySemaphore {
        device: device.clone(),
        semaphore,
        cuda_fd,
    })
}

/// Use a 256-byte CUDA-friendly pitch alignment.
pub fn allocate_nv12_buffer_default_pitch(
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<Nv12VulkanBuffer, String> {
    allocate_nv12_buffer(device, adapter, width, height, DEFAULT_PITCH_ALIGNMENT)
}

fn validate_dimensions(width: u32, height: u32, alignment: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("NV12 width and height must both be non-zero".into());
    }
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err(format!(
            "pitch alignment must be a non-zero power of two; received {alignment}"
        ));
    }
    Ok(())
}

fn nv12_layout(width: u32, height: u32, alignment: u32) -> Result<(u32, u64), String> {
    validate_dimensions(width, height, alignment)?;
    let pitch = align_up(width, alignment).ok_or_else(|| "NV12 pitch overflows u32".to_owned())?;
    let rows = u64::from(height) + u64::from(height).div_ceil(2);
    let size = u64::from(pitch)
        .checked_mul(rows)
        .ok_or_else(|| "NV12 allocation size overflows u64".to_owned())?;
    Ok((pitch, size))
}

fn align_up(value: u32, alignment: u32) -> Option<u32> {
    value
        .checked_add(alignment - 1)
        .map(|n| n & !(alignment - 1))
}

fn require_external_fd_extension(extensions: &[&CStr]) -> Result<(), String> {
    if extensions
        .iter()
        .any(|extension| *extension == EXTERNAL_MEMORY_FD)
    {
        Ok(())
    } else {
        Err(format!(
            "wgpu Vulkan device did not enable {}",
            EXTERNAL_MEMORY_FD.to_string_lossy()
        ))
    }
}

fn verify_exportable_buffer(
    instance: &AshInstance,
    physical_device: vk::PhysicalDevice,
) -> Result<bool, String> {
    let info = vk::PhysicalDeviceExternalBufferInfo::default()
        .flags(vk::BufferCreateFlags::empty())
        .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
        .handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
    let mut properties = vk::ExternalBufferProperties::default();
    unsafe {
        instance.get_physical_device_external_buffer_properties(
            physical_device,
            &info,
            &mut properties,
        );
    }
    let external = properties.external_memory_properties;
    if !external
        .external_memory_features
        .contains(vk::ExternalMemoryFeatureFlags::EXPORTABLE)
    {
        return Err("Vulkan device cannot export STORAGE_BUFFER memory as OPAQUE_FD".into());
    }
    if !external
        .compatible_handle_types
        .contains(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD)
    {
        return Err("Vulkan device reports OPAQUE_FD as incompatible for STORAGE_BUFFER".into());
    }
    Ok(external
        .external_memory_features
        .contains(vk::ExternalMemoryFeatureFlags::DEDICATED_ONLY))
}

fn create_external_buffer(device: &AshDevice, size: u64) -> Result<vk::Buffer, String> {
    let mut external_info = vk::ExternalMemoryBufferCreateInfo::default()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
    let info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .push_next(&mut external_info);
    unsafe { device.create_buffer(&info, None) }
        .map_err(|error| format!("failed to create external Vulkan NV12 buffer: {error}"))
}

fn choose_memory_type(
    instance: &AshInstance,
    physical_device: vk::PhysicalDevice,
    supported_types: u32,
) -> Result<u32, String> {
    let properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let mut fallback = None;
    for index in 0..properties.memory_type_count {
        if supported_types & (1 << index) == 0 {
            continue;
        }
        let flags = properties.memory_types[index as usize].property_flags;
        if flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            return Ok(index);
        }
        fallback.get_or_insert(index);
    }
    fallback.ok_or_else(|| {
        format!(
            "Vulkan has no compatible memory type for NV12 buffer (type bits {supported_types:#x})"
        )
    })
}

fn allocate_exportable_memory(
    device: &AshDevice,
    buffer: vk::Buffer,
    requirements: vk::MemoryRequirements,
    memory_type_index: u32,
    dedicated_only: bool,
) -> Result<vk::DeviceMemory, String> {
    let mut export_info = vk::ExportMemoryAllocateInfo::default()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
    let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().buffer(buffer);
    let info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index)
        .push_next(&mut export_info);
    let info = if dedicated_only {
        info.push_next(&mut dedicated_info)
    } else {
        info
    };
    unsafe { device.allocate_memory(&info, None) }
        .map_err(|error| format!("failed to allocate exportable Vulkan memory: {error}"))
}

struct VulkanBufferGuard<'a> {
    device: &'a AshDevice,
    buffer: Option<vk::Buffer>,
    memory: Option<vk::DeviceMemory>,
}

impl<'a> VulkanBufferGuard<'a> {
    fn new(device: &'a AshDevice, buffer: vk::Buffer) -> Self {
        Self {
            device,
            buffer: Some(buffer),
            memory: None,
        }
    }

    fn transfer(&mut self) -> (vk::Buffer, vk::DeviceMemory, u64) {
        let buffer = self.buffer.take().expect("guard owns Vulkan buffer");
        let memory = self.memory.take().expect("guard owns Vulkan memory");
        // The caller set memory from VkMemoryRequirements before transfer;
        // query it again here to preserve the exact HAL managed allocation span.
        let size = unsafe { self.device.get_buffer_memory_requirements(buffer).size };
        (buffer, memory, size)
    }
}

impl Drop for VulkanBufferGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            if let Some(buffer) = self.buffer.take() {
                self.device.destroy_buffer(buffer, None);
            }
            if let Some(memory) = self.memory.take() {
                self.device.free_memory(memory, None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::nv12_layout;

    #[test]
    fn nv12_layout_accounts_for_odd_webp_frame_dimensions() {
        assert_eq!(nv12_layout(160, 75, 256).unwrap(), (256, 28_928));
        assert_eq!(nv12_layout(161, 75, 256).unwrap(), (256, 28_928));
        assert_eq!(nv12_layout(160, 90, 256).unwrap(), (256, 34_560));
    }
}
