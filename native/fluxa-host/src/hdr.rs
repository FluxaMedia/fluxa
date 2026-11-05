use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use fluxa_renderer::hdr_compose::{HdrCompose, HdrEncoding, HdrFrame};

static REQUESTED: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static PEAK_NITS: AtomicU32 = AtomicU32::new(0);
static REPORTED: AtomicBool = AtomicBool::new(false);

pub fn request(enabled: bool) {
    REQUESTED.store(enabled, Ordering::Release);
}

pub fn requested() -> bool {
    REQUESTED.load(Ordering::Acquire)
}

pub fn active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}

pub(super) fn deactivate() {
    ACTIVE.store(false, Ordering::Release);
}

pub fn peak_nits() -> Option<f32> {
    let bits = PEAK_NITS.load(Ordering::Acquire);
    (bits != 0).then(|| f32::from_bits(bits))
}

const DEFAULT_SDR_WHITE: f32 = 203.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HdrOutput {
    pub format: wgpu::TextureFormat,
    pub color_space: wgpu::SurfaceColorSpace,
}

pub(super) fn pick_hdr_output(capabilities: &wgpu::SurfaceCapabilities) -> Option<HdrOutput> {
    use wgpu::{SurfaceColorSpace as Space, SurfaceColorSpaces as Spaces, TextureFormat as Format};
    [
        (Format::Rgb10a2Unorm, Spaces::BT2100_PQ, Space::Bt2100Pq),
        (
            Format::Rgba16Float,
            Spaces::EXTENDED_SRGB_LINEAR,
            Space::ExtendedSrgbLinear,
        ),
    ]
    .into_iter()
    .find(|(format, flag, _)| capabilities.color_spaces(*format).contains(*flag))
    .map(|(format, _, color_space)| HdrOutput {
        format,
        color_space,
    })
}

pub(super) fn display_in_hdr_mode(display: &wgpu::DisplayHdrInfo) -> bool {
    let reported = display.coarse.and_then(|coarse| coarse.high_dynamic_range);
    reported != Some(false)
}

pub(super) fn describe_display(display: &wgpu::DisplayHdrInfo) -> String {
    let hdr = display.coarse.and_then(|coarse| coarse.high_dynamic_range);
    let luminance = display.luminance;
    format!(
        "hdr={hdr:?} max={:?} white={:?}",
        luminance.and_then(|l| l.max_nits),
        luminance.and_then(|l| l.sdr_white_nits)
    )
}

pub(super) fn lift_video(
    shapes: &mut [egui::epaint::ClippedShape],
    scale: f32,
) -> Option<[f32; 4]> {
    let mut found = None;
    for clipped in shapes.iter_mut() {
        let egui::Shape::Callback(callback) = &clipped.shape else {
            continue;
        };
        if !callback.callback.is::<fluxa_ui::VideoHole>() {
            continue;
        }
        let rect = callback.rect;
        clipped.shape = egui::Shape::Callback(fluxa_renderer::hole::shape(rect, 0.0));
        found = Some([
            rect.min.x * scale,
            rect.min.y * scale,
            rect.width() * scale,
            rect.height() * scale,
        ]);
    }
    found
}

pub(super) fn describe(capabilities: &wgpu::SurfaceCapabilities) -> String {
    capabilities
        .format_capabilities
        .iter()
        .map(|entry| format!("{:?}={:?}", entry.format, entry.color_spaces))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) struct HdrState {
    compose: HdrCompose,
    output: HdrOutput,
    display: String,
    ui: Option<(wgpu::TextureView, [u32; 2])>,
    ui_format: wgpu::TextureFormat,
    video: Option<wgpu::TextureView>,
    video_rect: Option<[f32; 4]>,
    sdr_white: f32,
    pub(super) dirty: bool,
}

impl HdrState {
    pub(super) fn new(
        device: &wgpu::Device,
        output: HdrOutput,
        ui_format: wgpu::TextureFormat,
        display: &wgpu::DisplayHdrInfo,
    ) -> Self {
        let encoding = match output.color_space {
            wgpu::SurfaceColorSpace::ExtendedSrgbLinear => HdrEncoding::Scrgb,
            _ => HdrEncoding::Pq,
        };
        let state = Self {
            compose: HdrCompose::new(device, output.format, encoding),
            output,
            display: describe_display(display),
            ui: None,
            ui_format,
            video: None,
            video_rect: None,
            sdr_white: DEFAULT_SDR_WHITE,
            dirty: false,
        };
        state.publish(display);
        state
    }

    pub(super) fn refresh(&mut self, display: &wgpu::DisplayHdrInfo) {
        self.publish(display);
        self.sdr_white = display
            .luminance
            .and_then(|luminance| luminance.sdr_white_nits)
            .filter(|nits| nits.is_finite() && *nits >= 80.0)
            .unwrap_or(DEFAULT_SDR_WHITE);
    }

    fn publish(&self, display: &wgpu::DisplayHdrInfo) {
        let peak = display
            .luminance
            .and_then(|luminance| luminance.max_nits)
            .filter(|nits| nits.is_finite() && *nits > 0.0)
            .unwrap_or(0.0);
        PEAK_NITS.store(peak.to_bits(), Ordering::Release);
        ACTIVE.store(true, Ordering::Release);
    }

    pub(super) fn set_video(&mut self, view: Option<wgpu::TextureView>) {
        self.video = view;
        self.dirty = true;
    }

    pub(super) fn report_first_video(&self, adapter: &wgpu::AdapterInfo) {
        if self.video.is_none() || REPORTED.swap(true, Ordering::AcqRel) {
            return;
        }
        fluxa_crash::capture(
            "HDR output active",
            &[
                ("gpu.adapter", &adapter.name),
                ("gpu.backend", &format!("{:?}", adapter.backend)),
                ("hdr.format", &format!("{:?}", self.output.format)),
                ("hdr.color_space", &format!("{:?}", self.output.color_space)),
                ("hdr.display", &self.display),
                (
                    "hdr.video_rect",
                    if self.video_rect.is_some() {
                        "found"
                    } else {
                        "missing"
                    },
                ),
            ],
        );
    }

    pub(super) fn set_video_rect(&mut self, rect: Option<[f32; 4]>) {
        self.video_rect = rect;
    }

    pub(super) fn ui_target(&mut self, device: &wgpu::Device, size: [u32; 2]) -> wgpu::TextureView {
        if let Some((view, current)) = &self.ui
            && *current == size
        {
            return view.clone();
        }
        let view = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("fluxa-hdr-ui"),
                size: wgpu::Extent3d {
                    width: size[0],
                    height: size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.ui_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.ui = Some((view.clone(), size));
        view
    }

    pub(super) fn compose(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) -> Option<wgpu::CommandBuffer> {
        let (ui, _) = self.ui.as_ref()?;
        let video = self.video.as_ref().zip(self.video_rect);
        Some(self.compose.compose(
            device,
            queue,
            &HdrFrame {
                ui,
                ui_encoded: !self.ui_format.is_srgb(),
                video,
                sdr_white_nits: self.sdr_white,
                target,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities(
        entries: &[(wgpu::TextureFormat, wgpu::SurfaceColorSpaces)],
    ) -> wgpu::SurfaceCapabilities {
        wgpu::SurfaceCapabilities {
            format_capabilities: entries
                .iter()
                .map(|&(format, color_spaces)| wgpu::SurfaceFormatCapabilities {
                    format,
                    color_spaces,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn sdr_only_surface_has_no_hdr_output() {
        let caps = capabilities(&[(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::SurfaceColorSpaces::SRGB,
        )]);
        assert_eq!(pick_hdr_output(&caps), None);
    }

    fn display(high_dynamic_range: Option<bool>) -> wgpu::DisplayHdrInfo {
        wgpu::DisplayHdrInfo {
            coarse: Some(wgpu::DisplayCoarseRange {
                high_dynamic_range,
                gamut: None,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn sdr_display_blocks_hdr_output() {
        assert!(!display_in_hdr_mode(&display(Some(false))));
    }

    #[test]
    fn unknown_display_keeps_hdr_output() {
        assert!(display_in_hdr_mode(&wgpu::DisplayHdrInfo::default()));
        assert!(display_in_hdr_mode(&display(None)));
        assert!(display_in_hdr_mode(&display(Some(true))));
    }

    #[test]
    fn pq_wins_over_scrgb() {
        let caps = capabilities(&[
            (
                wgpu::TextureFormat::Rgba16Float,
                wgpu::SurfaceColorSpaces::EXTENDED_SRGB_LINEAR,
            ),
            (
                wgpu::TextureFormat::Rgb10a2Unorm,
                wgpu::SurfaceColorSpaces::BT2100_PQ,
            ),
        ]);
        assert_eq!(
            pick_hdr_output(&caps).map(|output| output.color_space),
            Some(wgpu::SurfaceColorSpace::Bt2100Pq)
        );
    }
}
