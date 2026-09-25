//! Platform-facing contracts shared by Android, desktop, and future webOS
//! hosts. This module intentionally does not depend on JNI, winit, Compose,
//! or a particular windowing toolkit.

use crate::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsBackend {
    Vulkan,
    Gles,
    Metal,
}

impl GraphicsBackend {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Vulkan => "Vulkan",
            Self::Gles => "GLES",
            Self::Metal => "Metal",
        }
    }
}

/// Android hosts must try Vulkan first. GLES is intentionally a fallback for
/// older boxes/TVs whose vendor driver does not expose Vulkan.
pub const ANDROID_BACKEND_ORDER: [GraphicsBackend; 2] =
    [GraphicsBackend::Vulkan, GraphicsBackend::Gles];

pub const APPLE_BACKEND_ORDER: [GraphicsBackend; 1] = [GraphicsBackend::Metal];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceLifecycle {
    Detached,
    Attached {
        width: u32,
        height: u32,
        generation: u64,
    },
}

impl Default for SurfaceLifecycle {
    fn default() -> Self {
        Self::Detached
    }
}

impl SurfaceLifecycle {
    pub fn is_renderable(self) -> bool {
        matches!(self, Self::Attached { width, height, .. } if width > 0 && height > 0)
    }

    pub fn size(self) -> Option<[u32; 2]> {
        match self {
            Self::Detached => None,
            Self::Attached { width, height, .. } => Some([width, height]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceEvent {
    Created { width: u32, height: u32 },
    Resized { width: u32, height: u32 },
    Destroyed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceController {
    lifecycle: SurfaceLifecycle,
    next_generation: u64,
}

impl Default for SurfaceController {
    fn default() -> Self {
        Self {
            lifecycle: SurfaceLifecycle::Detached,
            next_generation: 0,
        }
    }
}

impl SurfaceController {
    pub fn lifecycle(self) -> SurfaceLifecycle {
        self.lifecycle
    }

    pub fn apply(&mut self, event: SurfaceEvent) -> SurfaceLifecycle {
        self.lifecycle = match event {
            SurfaceEvent::Created { width, height } => {
                self.next_generation = self.next_generation.saturating_add(1);
                SurfaceLifecycle::Attached {
                    width,
                    height,
                    generation: self.next_generation,
                }
            }
            SurfaceEvent::Resized { width, height } => match self.lifecycle {
                SurfaceLifecycle::Attached { generation, .. } => SurfaceLifecycle::Attached {
                    width,
                    height,
                    generation,
                },
                SurfaceLifecycle::Detached => SurfaceLifecycle::Detached,
            },
            SurfaceEvent::Destroyed => SurfaceLifecycle::Detached,
        };
        self.lifecycle
    }

    pub fn viewport(self) -> Option<Rect> {
        self.lifecycle
            .size()
            .map(|[width, height]| Rect::new(0.0, 0.0, width as f32, height as f32))
    }
}

/// Convert a user-facing backend setting into wgpu's backend bitset.
pub fn backends_for(preference: GraphicsBackend) -> wgpu::Backends {
    match preference {
        GraphicsBackend::Vulkan => wgpu::Backends::VULKAN,
        GraphicsBackend::Gles => wgpu::Backends::GL,
        GraphicsBackend::Metal => wgpu::Backends::METAL,
    }
}

/// Parse the same backend override used by the desktop smoke app. Android
/// uses the ordered list above in its host adapter rather than selecting GLES
/// as the default.
pub fn backend_override(value: Option<&str>) -> Option<wgpu::Backends> {
    match value.map(str::to_ascii_lowercase).as_deref() {
        Some("vulkan") => Some(wgpu::Backends::VULKAN),
        Some("gl") | Some("gles") | Some("opengl") => Some(wgpu::Backends::GL),
        Some("metal") => Some(wgpu::Backends::METAL),
        Some("dx12") | Some("directx12") => Some(wgpu::Backends::DX12),
        Some("all") => Some(wgpu::Backends::all()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_is_vulkan_first_with_gles_fallback() {
        assert_eq!(
            ANDROID_BACKEND_ORDER,
            [GraphicsBackend::Vulkan, GraphicsBackend::Gles]
        );
        assert_eq!(
            backends_for(GraphicsBackend::Vulkan),
            wgpu::Backends::VULKAN
        );
        assert_eq!(backends_for(GraphicsBackend::Gles), wgpu::Backends::GL);
    }

    #[test]
    fn surface_generation_survives_resize_and_changes_on_recreation() {
        let mut controller = SurfaceController::default();
        assert!(!controller.lifecycle().is_renderable());
        let first = controller.apply(SurfaceEvent::Created {
            width: 1920,
            height: 1080,
        });
        assert_eq!(first.size(), Some([1920, 1080]));
        let resized = controller.apply(SurfaceEvent::Resized {
            width: 1280,
            height: 720,
        });
        assert_eq!(
            resized,
            SurfaceLifecycle::Attached {
                width: 1280,
                height: 720,
                generation: 1
            }
        );
        controller.apply(SurfaceEvent::Destroyed);
        let second = controller.apply(SurfaceEvent::Created {
            width: 3840,
            height: 2160,
        });
        assert_eq!(
            second,
            SurfaceLifecycle::Attached {
                width: 3840,
                height: 2160,
                generation: 2
            }
        );
    }

    #[test]
    fn resize_before_create_does_not_attach_a_surface() {
        let mut controller = SurfaceController::default();
        assert_eq!(
            controller.apply(SurfaceEvent::Resized {
                width: 100,
                height: 100
            }),
            SurfaceLifecycle::Detached
        );
    }
}
