use crate::DesktopState;
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderBackend {
    Vulkan,
    #[cfg(target_os = "windows")]
    D3d11,
}

impl RenderBackend {
    pub fn name(self) -> &'static str {
        match self {
            Self::Vulkan => "vulkan",
            #[cfg(target_os = "windows")]
            Self::D3d11 => "d3d11",
        }
    }
}

pub fn read_render_backend(app: &AppHandle) -> RenderBackend {
    let state = app.state::<DesktopState>();
    match crate::storage::read_pref_field(state, "renderBackend").as_deref() {
        Some("vulkan") | None => RenderBackend::Vulkan,
        #[cfg(target_os = "windows")]
        Some("d3d11") => RenderBackend::D3d11,
        _ => RenderBackend::Vulkan,
    }
}
