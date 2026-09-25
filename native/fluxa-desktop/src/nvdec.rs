//! Optional FFmpeg/CUVID VP8 decoder used by the Linux Vulkan artwork path.
//!
//! The decoder is intentionally thread-affine: FFmpeg and its CUDA context are
//! created, used, and destroyed on the same worker thread. Its imported
//! external allocation must outlive this value, and Vulkan must wait for the
//! CUDA producer before reading the returned device pointer.

#![allow(dead_code)]

use std::{
    marker::PhantomData,
    os::fd::AsRawFd,
    ptr::NonNull,
    rc::Rc,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use crate::nvdec_vulkan_buffer::{self, Nv12VulkanBuffer};
use fluxa_artwork::NvdecWebpAnimation;
use wgpu;

const OK: i32 = 0;
const NO_FRAME: i32 = -5;

#[repr(C)]
struct RawDecoder {
    _opaque: [u8; 0],
}

unsafe extern "C" {
    fn fluxa_nvdec_create(out_decoder: *mut *mut RawDecoder, width: i32, height: i32) -> i32;
    fn fluxa_nvdec_destroy(decoder: *mut RawDecoder);
    fn fluxa_nvdec_send_packet(
        decoder: *mut RawDecoder,
        packet_data: *const u8,
        packet_size: usize,
    ) -> i32;
    fn fluxa_nvdec_send_end(decoder: *mut RawDecoder) -> i32;
    fn fluxa_nvdec_receive_frame(
        decoder: *mut RawDecoder,
        destination: u64,
        destination_pitch: usize,
    ) -> i32;
    fn fluxa_nvdec_alloc_output(decoder: *mut RawDecoder, size: usize, out_memory: *mut u64)
    -> i32;
    fn fluxa_nvdec_free_output(decoder: *mut RawDecoder, memory: u64) -> i32;
    fn fluxa_nvdec_import_vulkan_buffer(
        decoder: *mut RawDecoder,
        fd: i32,
        allocation_size: usize,
        buffer_size: usize,
        out_memory: *mut u64,
    ) -> i32;
    fn fluxa_nvdec_import_vulkan_semaphore(decoder: *mut RawDecoder, fd: i32) -> i32;
    fn fluxa_nvdec_import_vulkan_release_semaphore(decoder: *mut RawDecoder, fd: i32) -> i32;
    fn fluxa_nvdec_cuda_device_uuid(decoder: *mut RawDecoder, out_uuid: *mut u8) -> i32;
}

pub(crate) struct NvdecDecoder {
    raw: NonNull<RawDecoder>,
    dimensions: [u32; 2],
    cuda_output: Option<u64>,
    // Drop runs before fields are dropped; it destroys the CUDA import first,
    // then this Vulkan allocation can be released safely.
    output_buffer: Option<Arc<Nv12VulkanBuffer>>,
    // FFmpeg's AVCodecContext and CUDA context must not be used concurrently
    // or moved to another thread after creation.
    _thread_affine: PhantomData<Rc<()>>,
}

/// Messages produced by the asynchronous VP8/NVDEC worker. Frame events are
/// ordered; each references the same shared NV12 allocation, so the consumer
/// must submit its Vulkan release-semaphore signal before the worker can
/// overwrite it with the next frame.
pub(crate) enum NvdecWorkerEvent {
    Frame {
        index: usize,
        duration: std::time::Duration,
        alpha: Option<Arc<Vec<u8>>>,
        buffer: Arc<Nv12VulkanBuffer>,
    },
    Finished {
        decoded_frames: usize,
    },
    Failed {
        decoded_frames: usize,
        error: String,
    },
}

pub(crate) struct NvdecWorker {
    pub(crate) receiver: Receiver<NvdecWorkerEvent>,
    /// Keep the CUDA import and exported Vulkan allocation alive until all
    /// already-submitted GPU reads have completed.
    pub(crate) gpu_completion: Option<Sender<()>>,
}

/// Decode an eligible full-canvas VP8 animated WebP away from the UI thread.
/// The returned receiver gets one frame event at a time and exactly one
/// terminal event unless the receiver is dropped.
pub(crate) fn spawn_vp8_worker(
    animation: NvdecWebpAnimation,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
) -> Result<NvdecWorker, String> {
    let (sender, receiver) = mpsc::channel();
    let worker_sender = sender.clone();
    let terminal_sender = sender.clone();
    let (gpu_completion, gpu_completion_wait) = mpsc::channel();
    thread::Builder::new()
        .name("fluxa-nvdec-vp8".to_owned())
        .spawn(move || {
            let mut decoded_frames = 0;
            match decode_animation(
                animation,
                &device,
                &adapter,
                &worker_sender,
                &mut decoded_frames,
            ) {
                Ok(()) => {
                    if terminal_sender
                        .send(NvdecWorkerEvent::Finished { decoded_frames })
                        .is_ok()
                    {
                        let _ = gpu_completion_wait.recv();
                    }
                }
                Err(error) => {
                    if terminal_sender
                        .send(NvdecWorkerEvent::Failed {
                            decoded_frames,
                            error,
                        })
                        .is_ok()
                    {
                        let _ = gpu_completion_wait.recv();
                    }
                }
            }
        })
        .map_err(|error| format!("failed to spawn NVDEC VP8 worker thread: {error}"))?;
    Ok(NvdecWorker {
        receiver,
        gpu_completion: Some(gpu_completion),
    })
}

fn decode_animation(
    animation: NvdecWebpAnimation,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    sender: &Sender<NvdecWorkerEvent>,
    decoded_frames: &mut usize,
) -> Result<(), String> {
    if animation.frames.is_empty() {
        return Err("NVDEC animation contains no VP8 frames".to_owned());
    }
    let packet_count = animation.frames.len();
    let mut decoder = NvdecDecoder::new(animation.canvas_width, animation.canvas_height)?;
    decoder.create_vulkan_output(device, adapter)?;
    let buffer = decoder
        .output_buffer()
        .ok_or_else(|| "NVDEC shared output buffer was not created".to_owned())?;

    for frame in &animation.frames {
        loop {
            if decoder.send_vp8_frame(&frame.vp8)? {
                break;
            }
            let drained =
                drain_available_frames(&mut decoder, &buffer, &animation, decoded_frames, sender)?;
            if drained == 0 {
                return Err("NVDEC reported VP8 send EAGAIN without a drainable frame".to_owned());
            }
        }
        drain_available_frames(&mut decoder, &buffer, &animation, decoded_frames, sender)?;
    }

    loop {
        if decoder.send_end_of_stream()? {
            break;
        }
        let drained =
            drain_available_frames(&mut decoder, &buffer, &animation, decoded_frames, sender)?;
        if drained == 0 {
            return Err("NVDEC reported flush EAGAIN without a drainable frame".to_owned());
        }
    }
    drain_available_frames(&mut decoder, &buffer, &animation, decoded_frames, sender)?;

    if *decoded_frames != packet_count {
        return Err(format!(
            "NVDEC decoded {} frames for {packet_count} VP8 packets",
            *decoded_frames
        ));
    }
    debug_assert_eq!(*decoded_frames, packet_count);
    Ok(())
}

fn drain_available_frames(
    decoder: &mut NvdecDecoder,
    buffer: &Arc<Nv12VulkanBuffer>,
    animation: &NvdecWebpAnimation,
    decoded_frames: &mut usize,
    sender: &Sender<NvdecWorkerEvent>,
) -> Result<usize, String> {
    let before = *decoded_frames;
    while decoder.receive_shared_nv12()? {
        let index = *decoded_frames;
        let frame = animation.frames.get(index).ok_or_else(|| {
            format!(
                "NVDEC produced more frames than the {} VP8 packets",
                animation.frames.len()
            )
        })?;
        sender
            .send(NvdecWorkerEvent::Frame {
                index,
                duration: frame.duration,
                alpha: frame.alpha.as_ref().map(Arc::clone),
                buffer: Arc::clone(buffer),
            })
            .map_err(|_| "NVDEC frame receiver was dropped".to_owned())?;
        *decoded_frames += 1;
    }
    Ok(*decoded_frames - before)
}

impl NvdecDecoder {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("NVDEC requires positive VP8 frame dimensions".to_owned());
        }
        let width = i32::try_from(width).map_err(|_| "VP8 frame width is too large")?;
        let height = i32::try_from(height).map_err(|_| "VP8 frame height is too large")?;
        let mut raw = std::ptr::null_mut();
        let result = unsafe { fluxa_nvdec_create(&mut raw, width, height) };
        if result != OK {
            return Err(format!("could not create VP8 NVDEC context ({result})"));
        }
        let raw = NonNull::new(raw)
            .ok_or_else(|| "NVDEC returned a null decoder after successful creation".to_owned())?;
        Ok(Self {
            raw,
            dimensions: [width as u32, height as u32],
            cuda_output: None,
            output_buffer: None,
            _thread_affine: PhantomData,
        })
    }

    pub(crate) fn dimensions(&self) -> [u32; 2] {
        self.dimensions
    }

    /// Allocates and imports the shared Vulkan/CUDA NV12 buffer for this
    /// decoder. The decoder owns the buffer so the Vulkan allocation outlives
    /// the CUDA external-memory mapping.
    pub(crate) fn create_vulkan_output(
        &mut self,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<u64, String> {
        if self.output_buffer.is_some() {
            return Err("NVDEC output buffer has already been imported".to_owned());
        }
        self.verify_cuda_vulkan_device(adapter)?;
        let buffer = nvdec_vulkan_buffer::allocate_nv12_buffer_default_pitch(
            device,
            adapter,
            self.dimensions[0],
            self.dimensions[1],
        )?;
        // Keep the allocation alive immediately: if a later CUDA import
        // fails, earlier successful imports remain valid until decoder drop.
        self.output_buffer = Some(Arc::new(buffer));
        let buffer = self.output_buffer.as_ref().expect("just inserted");
        let allocation_size = usize::try_from(buffer.allocation_size)
            .map_err(|_| "Vulkan allocation size does not fit in usize")?;
        let buffer_size =
            usize::try_from(buffer.size).map_err(|_| "Vulkan buffer size does not fit in usize")?;
        let mut device_ptr = 0;
        let result = unsafe {
            fluxa_nvdec_import_vulkan_buffer(
                self.raw.as_ptr(),
                buffer.cuda_fd.as_raw_fd(),
                allocation_size,
                buffer_size,
                &mut device_ptr,
            )
        };
        if result != OK || device_ptr == 0 {
            return Err(format!(
                "could not import Vulkan NV12 buffer into CUDA ({result})"
            ));
        }
        let ready_result = unsafe {
            fluxa_nvdec_import_vulkan_semaphore(
                self.raw.as_ptr(),
                buffer.ready_semaphore.cuda_fd.as_raw_fd(),
            )
        };
        if ready_result != OK {
            return Err(format!(
                "could not import Vulkan ready semaphore into CUDA ({ready_result})"
            ));
        }
        let release_result = unsafe {
            fluxa_nvdec_import_vulkan_release_semaphore(
                self.raw.as_ptr(),
                buffer.release_semaphore.cuda_fd.as_raw_fd(),
            )
        };
        if release_result != OK {
            return Err(format!(
                "could not import Vulkan release semaphore into CUDA ({release_result})"
            ));
        }
        self.cuda_output = Some(device_ptr);
        Ok(device_ptr)
    }

    fn verify_cuda_vulkan_device(&self, adapter: &wgpu::Adapter) -> Result<(), String> {
        let mut cuda_uuid = [0_u8; 16];
        let result =
            unsafe { fluxa_nvdec_cuda_device_uuid(self.raw.as_ptr(), cuda_uuid.as_mut_ptr()) };
        if result != OK {
            return Err(format!(
                "could not identify the CUDA device used by NVDEC ({result})"
            ));
        }

        let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }
            .ok_or_else(|| "wgpu adapter is not backed by Vulkan HAL".to_owned())?;
        let physical_device = hal_adapter.raw_physical_device();
        let instance = hal_adapter.shared_instance().raw_instance();
        let mut id_properties = ash::vk::PhysicalDeviceIDProperties::default();
        let mut properties =
            ash::vk::PhysicalDeviceProperties2::default().push_next(&mut id_properties);
        unsafe { instance.get_physical_device_properties2(physical_device, &mut properties) };
        let vulkan_uuid = id_properties.device_uuid;
        if cuda_uuid != vulkan_uuid {
            return Err(format!(
                "CUDA/NVDEC and Vulkan adapters differ (CUDA UUID {}, Vulkan UUID {}); keeping animated WebP on the CPU fallback",
                format_uuid(cuda_uuid),
                format_uuid(vulkan_uuid),
            ));
        }
        Ok(())
    }

    pub(crate) fn output_buffer(&self) -> Option<Arc<Nv12VulkanBuffer>> {
        self.output_buffer.clone()
    }

    /// Allocates a temporary CUDA output buffer for diagnostics/fallback tests.
    pub(crate) fn allocate_cuda_output(&self, size: usize) -> Result<u64, String> {
        let mut memory = 0;
        let result = unsafe { fluxa_nvdec_alloc_output(self.raw.as_ptr(), size, &mut memory) };
        if result != OK || memory == 0 {
            return Err(format!("could not allocate CUDA NV12 output ({result})"));
        }
        Ok(memory)
    }

    pub(crate) fn free_cuda_output(&self, memory: u64) -> Result<(), String> {
        let result = unsafe { fluxa_nvdec_free_output(self.raw.as_ptr(), memory) };
        if result == OK {
            Ok(())
        } else {
            Err(format!("could not free CUDA NV12 output ({result})"))
        }
    }

    pub(crate) fn send_vp8_frame(&mut self, packet: &[u8]) -> Result<bool, String> {
        let result =
            unsafe { fluxa_nvdec_send_packet(self.raw.as_ptr(), packet.as_ptr(), packet.len()) };
        match result {
            OK => Ok(true),
            NO_FRAME => Ok(false),
            _ => Err(format!("NVDEC could not submit VP8 packet ({result})")),
        }
    }

    pub(crate) fn send_end_of_stream(&mut self) -> Result<bool, String> {
        let result = unsafe { fluxa_nvdec_send_end(self.raw.as_ptr()) };
        match result {
            OK => Ok(true),
            NO_FRAME => Ok(false),
            _ => Err(format!("NVDEC flush failed ({result})")),
        }
    }

    /// Copies one decoded NV12 frame into the CUDA-visible output buffer. `Ok(false)`
    /// means the decoder needs more input or has been fully drained.
    pub(crate) fn receive_nv12(&mut self, destination: u64, pitch: usize) -> Result<bool, String> {
        let result = unsafe { fluxa_nvdec_receive_frame(self.raw.as_ptr(), destination, pitch) };
        match result {
            OK => Ok(true),
            NO_FRAME => Ok(false),
            _ => Err(format!(
                "NVDEC could not copy decoded NV12 planes ({result})"
            )),
        }
    }

    pub(crate) fn receive_shared_nv12(&mut self) -> Result<bool, String> {
        let destination = self
            .cuda_output
            .ok_or_else(|| "NVDEC Vulkan output buffer has not been created".to_owned())?;
        let pitch = self
            .output_buffer
            .as_ref()
            .map(|buffer| buffer.pitch as usize)
            .ok_or_else(|| "NVDEC Vulkan output buffer is unavailable".to_owned())?;
        self.receive_nv12(destination, pitch)
    }
}

fn format_uuid(uuid: [u8; 16]) -> String {
    uuid.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

impl Drop for NvdecDecoder {
    fn drop(&mut self) {
        // Imported CUDA mappings are released by the C owner before its CUDA
        // context and FFmpeg hardware device are torn down.
        unsafe { fluxa_nvdec_destroy(self.raw.as_ptr()) };
    }
}
