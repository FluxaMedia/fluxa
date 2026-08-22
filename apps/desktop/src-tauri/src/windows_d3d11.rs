use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709, DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020,
    DXGI_FORMAT, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    IDXGIDevice, IDXGIFactory2, IDXGISwapChain1, IDXGISwapChain3, DXGI_SCALING_STRETCH,
    DXGI_SWAP_CHAIN_COLOR_SPACE_SUPPORT_FLAG_PRESENT, DXGI_SWAP_CHAIN_DESC1,
    DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT, DXGI_USAGE_SHADER_INPUT,
};

pub struct D3d11Context {
    pub device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    format: DXGI_FORMAT,
    hwnd: HWND,
    width: AtomicI32,
    height: AtomicI32,
    hdr: AtomicBool,
}

unsafe impl Send for D3d11Context {}

fn display_wants_hdr(swap_chain: &IDXGISwapChain1) -> bool {
    let Ok(swap_chain3) = swap_chain.cast::<IDXGISwapChain3>() else {
        return false;
    };
    let Ok(support) =
        (unsafe { swap_chain3.CheckColorSpaceSupport(DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020) })
    else {
        return false;
    };
    support & DXGI_SWAP_CHAIN_COLOR_SPACE_SUPPORT_FLAG_PRESENT.0 as u32 != 0
}

impl D3d11Context {
    pub fn new(hwnd: isize, width: i32, height: i32, hdr_enabled: bool) -> Result<Self, String> {
        let hwnd = HWND(hwnd as *mut _);
        let width = width.max(2);
        let height = height.max(2);

        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
        }
        .map_err(|e| format!("D3D11CreateDevice failed: {e}"))?;
        let device = device.ok_or("D3D11CreateDevice returned no device")?;
        let context = context.ok_or("D3D11CreateDevice returned no context")?;

        let dxgi_device: IDXGIDevice = device
            .cast()
            .map_err(|e| format!("ID3D11Device -> IDXGIDevice failed: {e}"))?;
        let adapter = unsafe { dxgi_device.GetAdapter() }
            .map_err(|e| format!("IDXGIDevice::GetAdapter failed: {e}"))?;
        let factory: IDXGIFactory2 = unsafe { adapter.GetParent() }
            .map_err(|e| format!("IDXGIAdapter::GetParent(IDXGIFactory2) failed: {e}"))?;

        let sdr_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width as u32,
            Height: height as u32,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT | DXGI_USAGE_SHADER_INPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: windows::Win32::Graphics::Dxgi::Common::DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: 0,
        };

        let swap_chain =
            unsafe { factory.CreateSwapChainForHwnd(&device, hwnd, &sdr_desc, None, None) }
                .map_err(|e| format!("CreateSwapChainForHwnd failed: {e}"))?;

        let mut format = DXGI_FORMAT_R8G8B8A8_UNORM;
        let mut hdr = false;
        if hdr_enabled && display_wants_hdr(&swap_chain) {
            if let Ok(swap_chain3) = swap_chain.cast::<IDXGISwapChain3>() {
                if unsafe { swap_chain3.SetColorSpace1(DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020) }
                    .is_ok()
                {
                    hdr = true;
                    format = DXGI_FORMAT_R16G16B16A16_FLOAT;
                }
            }
        }

        let swap_chain = if hdr {
            drop(swap_chain);
            let hdr_desc = DXGI_SWAP_CHAIN_DESC1 {
                Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
                ..sdr_desc
            };
            let sc =
                unsafe { factory.CreateSwapChainForHwnd(&device, hwnd, &hdr_desc, None, None) }
                    .map_err(|e| format!("CreateSwapChainForHwnd (HDR) failed: {e}"))?;
            if let Ok(sc3) = sc.cast::<IDXGISwapChain3>() {
                let _ = unsafe { sc3.SetColorSpace1(DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709) };
            }
            sc
        } else {
            swap_chain
        };

        Ok(Self {
            device,
            context,
            swap_chain,
            format,
            hwnd,
            width: AtomicI32::new(width),
            height: AtomicI32::new(height),
            hdr: AtomicBool::new(hdr),
        })
    }

    pub fn device_ptr(&self) -> *mut std::ffi::c_void {
        self.device.as_raw()
    }

    pub fn is_hdr(&self) -> bool {
        self.hdr.load(Ordering::Acquire)
    }

    pub fn dxgi_format(&self) -> i32 {
        self.format.0
    }

    pub fn resize(&self, width: i32, height: i32) -> Result<(), String> {
        let width = width.max(2);
        let height = height.max(2);
        if self.width.load(Ordering::Acquire) == width
            && self.height.load(Ordering::Acquire) == height
        {
            return Ok(());
        }
        unsafe { self.context.ClearState() };
        unsafe {
            self.swap_chain.ResizeBuffers(
                0,
                width as u32,
                height as u32,
                self.format,
                Default::default(),
            )
        }
        .map_err(|e| format!("IDXGISwapChain1::ResizeBuffers failed: {e}"))?;
        self.width.store(width, Ordering::Release);
        self.height.store(height, Ordering::Release);
        Ok(())
    }

    pub fn width(&self) -> i32 {
        self.width.load(Ordering::Acquire)
    }

    pub fn height(&self) -> i32 {
        self.height.load(Ordering::Acquire)
    }

    pub fn back_buffer(&self) -> Result<ID3D11Texture2D, String> {
        unsafe { self.swap_chain.GetBuffer(0) }
            .map_err(|e| format!("IDXGISwapChain1::GetBuffer failed: {e}"))
    }

    pub fn present(&self, vsync: bool) -> Result<(), String> {
        unsafe {
            self.swap_chain
                .Present(if vsync { 1 } else { 0 }, Default::default())
        }
        .ok()
        .map_err(|e| format!("IDXGISwapChain1::Present failed: {e}"))
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}
