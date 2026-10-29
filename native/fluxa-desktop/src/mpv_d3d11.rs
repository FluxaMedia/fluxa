//! Windows video path for a DX12 host. wgpu runs on DX12 while mpv renders on
//! a D3D11 device of its own, on the same adapter, so `hwdec=d3d11va` decodes
//! straight into the frame mpv draws instead of being copied around a Vulkan
//! renderer. Frames cross over as shared textures ordered by two shared fences:
//!
//! * `forward`: D3D11 signals frame `n` once it is drawn, and the DX12 queue
//!   waits for it before the host samples the texture.
//! * `back`: the DX12 queue signals `n` when the host takes frame `n`. Every
//!   earlier frame was submitted before that point, so D3D11 waits for it
//!   before drawing over a texture one of those frames used.

use std::ffi::c_void;

use windows::Win32::Foundation::{CloseHandle, GENERIC_ALL, HANDLE, HMODULE};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_FENCE_FLAG_SHARED, D3D11_RESOURCE_MISC_SHARED,
    D3D11_RESOURCE_MISC_SHARED_NTHANDLE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT, D3D11CreateDevice, ID3D11Device, ID3D11Device5, ID3D11DeviceContext,
    ID3D11DeviceContext4, ID3D11Fence, ID3D11Multithread, ID3D11Texture2D,
};
use windows::Win32::Graphics::Direct3D12::{D3D12_FENCE_FLAG_SHARED, ID3D12Fence, ID3D12Resource};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_SHARED_RESOURCE_READ, DXGI_SHARED_RESOURCE_WRITE, IDXGIAdapter,
    IDXGIFactory4, IDXGIResource1,
};
use windows::core::{Interface, PCWSTR};

/// `DXGI_FORMAT` handed to mpv, matching the wgpu `Bgra8Unorm` view.
pub const FORMAT: i32 = DXGI_FORMAT_B8G8R8A8_UNORM.0;

/// The D3D11 half, owned by the mpv render thread.
pub struct Bridge {
    device: ID3D11Device,
    context: ID3D11DeviceContext4,
    forward: ID3D11Fence,
    back: ID3D11Fence,
    textures: Vec<ID3D11Texture2D>,
    count: u64,
}

/// The DX12 half, owned by whoever hands frames to the host.
pub struct Host {
    forward: ID3D12Fence,
    back: ID3D12Fence,
}

// The interfaces are free-threaded and the immediate context is only used from
// the render thread, with multithread protection on for mpv's decoder threads.
unsafe impl Send for Bridge {}
unsafe impl Send for Host {}
unsafe impl Sync for Host {}

pub struct Created {
    pub bridge: Bridge,
    pub host: Host,
    pub views: Vec<wgpu::TextureView>,
}

fn step<T>(what: &str, result: windows::core::Result<T>) -> Result<T, String> {
    result.map_err(|error| format!("{what}: {error}"))
}

/// Calls a COM method that returns its object through an out pointer.
fn out<T>(
    what: &str,
    call: impl FnOnce(*mut Option<T>) -> windows::core::Result<()>,
) -> Result<T, String> {
    let mut slot = None;
    step(what, call(&mut slot))?;
    slot.ok_or_else(|| format!("{what}: no object returned"))
}

fn raw_dx12(device: &wgpu::Device) -> Result<impl std::ops::Deref<Target = wgpu::hal::dx12::Device>, String> {
    unsafe { device.as_hal::<wgpu::hal::api::Dx12>() }
        .ok_or_else(|| "wgpu is not using the DX12 backend".to_owned())
}

pub fn is_dx12(device: &wgpu::Device) -> bool {
    unsafe { device.as_hal::<wgpu::hal::api::Dx12>() }.is_some()
}

impl Bridge {
    pub fn create(device: &wgpu::Device, count: usize, size: [u32; 2]) -> Result<Created, String> {
        let hal = raw_dx12(device)?;
        let d12 = hal.raw_device();

        let luid = unsafe { d12.GetAdapterLuid() };
        let factory: IDXGIFactory4 = step("create DXGI factory", unsafe { CreateDXGIFactory1() })?;
        let adapter: IDXGIAdapter = step("find the DX12 adapter", unsafe {
            factory.EnumAdapterByLuid(luid)
        })?;

        let levels: [D3D_FEATURE_LEVEL; 2] = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];
        let mut device11 = None;
        let mut context11 = None;
        step("create D3D11 device", unsafe {
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
                Some(&levels),
                D3D11_SDK_VERSION,
                Some(&mut device11),
                None,
                Some(&mut context11),
            )
        })?;
        let device11: ID3D11Device = device11.ok_or("D3D11 device missing")?;
        let context11: ID3D11DeviceContext = context11.ok_or("D3D11 context missing")?;
        // mpv's decoder threads share this device, so it must serialise calls.
        let multithread: ID3D11Multithread = step("D3D11 multithread", device11.cast())?;
        let _ = unsafe { multithread.SetMultithreadProtected(true) };
        let context: ID3D11DeviceContext4 = step("D3D11.4 context", context11.cast())?;
        let device5: ID3D11Device5 = step("D3D11.5 device", device11.cast())?;

        // forward: created by D3D11, opened by DX12.
        let forward: ID3D11Fence = out("create forward fence", |slot| unsafe {
            device5.CreateFence(0, D3D11_FENCE_FLAG_SHARED, slot)
        })?;
        let handle = step("share forward fence", unsafe {
            forward.CreateSharedHandle(None, GENERIC_ALL.0, PCWSTR::null())
        })?;
        let forward12: Result<ID3D12Fence, String> =
            out("open forward fence on DX12", |slot| unsafe {
                d12.OpenSharedHandle(handle, slot)
            });
        let _ = unsafe { CloseHandle(handle) };
        let forward12 = forward12?;

        // back: created by DX12, opened by D3D11.
        let back12: ID3D12Fence = step("create back fence", unsafe {
            d12.CreateFence(0, D3D12_FENCE_FLAG_SHARED)
        })?;
        let handle = step("share back fence", unsafe {
            d12.CreateSharedHandle(&back12, None, GENERIC_ALL.0, PCWSTR::null())
        })?;
        let back: Result<ID3D11Fence, String> = out("open back fence on D3D11", |slot| unsafe {
            device5.OpenSharedFence(handle, slot)
        });
        let _ = unsafe { CloseHandle(handle) };
        let back = back?;

        let mut textures = Vec::with_capacity(count);
        let mut views = Vec::with_capacity(count);
        for _ in 0..count {
            let (texture, view) = shared_texture(device, d12, &device11, size)?;
            textures.push(texture);
            views.push(view);
        }

        Ok(Created {
            bridge: Bridge {
                device: device11,
                context,
                forward,
                back,
                textures,
                count: count as u64,
            },
            host: Host {
                forward: forward12,
                back: back12,
            },
            views,
        })
    }

    /// The `ID3D11Device*` mpv's render context is created on.
    pub fn device_ptr(&self) -> *mut c_void {
        self.device.as_raw()
    }

    pub fn texture_ptr(&self, serial: u64) -> *mut c_void {
        self.textures[((serial - 1) % self.count) as usize].as_raw()
    }

    pub fn view_index(&self, serial: u64) -> usize {
        ((serial - 1) % self.count) as usize
    }

    /// Queue the wait that keeps frame `serial` from overwriting a texture the
    /// host may still be sampling.
    pub fn before_draw(&self, serial: u64) -> Result<(), String> {
        if serial > self.count {
            let value = serial - self.count + 1;
            step("wait for the host", unsafe {
                self.context.Wait(&self.back, value)
            })?;
        }
        Ok(())
    }

    pub fn after_draw(&self, serial: u64) -> Result<(), String> {
        step("signal frame", unsafe {
            self.context.Signal(&self.forward, serial)
        })?;
        unsafe { self.context.Flush() };
        Ok(())
    }
}

impl Host {
    /// Called on the thread that submits to the host queue, right before the
    /// frame is sampled.
    pub fn take(&self, device: &wgpu::Device, serial: u64) -> Result<(), String> {
        let hal = raw_dx12(device)?;
        let queue = hal.raw_queue();
        step("queue wait for frame", unsafe {
            queue.Wait(&self.forward, serial)
        })?;
        step("queue release of earlier frames", unsafe {
            queue.Signal(&self.back, serial)
        })
    }
}

fn shared_texture(
    device: &wgpu::Device,
    d12: &windows::Win32::Graphics::Direct3D12::ID3D12Device,
    device11: &ID3D11Device,
    size: [u32; 2],
) -> Result<(ID3D11Texture2D, wgpu::TextureView), String> {
    let description = D3D11_TEXTURE2D_DESC {
        Width: size[0],
        Height: size[1],
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        CPUAccessFlags: 0,
        MiscFlags: (D3D11_RESOURCE_MISC_SHARED.0 | D3D11_RESOURCE_MISC_SHARED_NTHANDLE.0) as u32,
    };
    let mut texture = None;
    step("create shared texture", unsafe {
        device11.CreateTexture2D(&description, None, Some(&mut texture))
    })?;
    let texture: ID3D11Texture2D = texture.ok_or("shared texture missing")?;
    let resource: IDXGIResource1 = step("shared texture handle interface", texture.cast())?;
    let handle: HANDLE = step("share texture", unsafe {
        resource.CreateSharedHandle(
            None,
            (DXGI_SHARED_RESOURCE_READ | DXGI_SHARED_RESOURCE_WRITE).0,
            PCWSTR::null(),
        )
    })?;
    let opened: Result<ID3D12Resource, String> =
        out("open texture on DX12", |slot| unsafe {
            d12.OpenSharedHandle(handle, slot)
        });
    let _ = unsafe { CloseHandle(handle) };
    let opened = opened?;

    let extent = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let hal_texture = unsafe {
        wgpu::hal::dx12::Device::texture_from_raw(
            opened,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureDimension::D2,
            extent,
            1,
            1,
        )
    };
    let wgpu_texture = unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Dx12>(
            hal_texture,
            &wgpu::TextureDescriptor {
                label: Some("fluxa-mpv-frame-d3d11"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::TextureUses::RESOURCE,
        )
    };
    let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
    Ok((texture, view))
}
