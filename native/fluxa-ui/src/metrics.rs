use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scaled {
    pub ratio: f32,
    pub min: f32,
    pub max: f32,
    pub offset: f32,
}

impl Scaled {
    pub fn of(self, basis: f32) -> f32 {
        (basis * self.ratio + self.offset).clamp(self.min, self.max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosterSizes {
    pub mark: Scaled,
    pub badge: Scaled,
    pub tag: Scaled,
    pub caption: Scaled,
    pub meta_bar: Scaled,
    pub meta_text: Scaled,
    pub landscape_text: Scaled,
    pub score_bar: Scaled,
    pub score_minimal: Scaled,
    pub score_number: Scaled,
    pub score_caption: Scaled,
    pub sash: Scaled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerInfo {
    pub left: f32,
    pub label_gap: f32,
    pub label_font: f32,
    pub title_font: f32,
    pub bottom: f32,
    pub scale: f32,
    pub description_rows: usize,
    pub width: Scaled,
    pub logo_width: f32,
    pub logo_height: f32,
}

/// Geometry consumed by every shared Rust screen.
///
/// These values are read from `shared/contracts/ui-tokens.json` through the
/// same parser used by the low-level renderer. The small fallbacks are only a
/// malformed-token safety net; they are not a second design system. Keeping
/// the lookup here prevents platform hosts from growing their own fixed UI
/// dimensions while still allowing the renderer to paint when a user ships a
/// partially migrated custom token document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMetrics {
    pub compact: bool,
    pub platform: &'static str,
    pub form: &'static str,
    pub window_class: &'static str,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub danger: Color32,
    pub motion_fast: f32,
    pub motion_normal: f32,
    pub motion_slow: f32,
    pub motion_curve: [f32; 4],
    pub navigation: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub button_primary: Color32,
    pub button_primary_text: Color32,
    pub card_background: Color32,
    pub input_background: Color32,
    pub input_border: Color32,
    pub input_border_focus: Option<Color32>,
    pub focus: Color32,
    pub home_continue_card_width: f32,
    pub space: [f32; 12],
    pub radius: [f32; 2],
    pub icon: [f32; 3],
    pub sheet_handle: Vec2,
    pub shade: [u8; 5],
    pub tone: [u8; 10],
    pub stroke: [f32; 3],
    pub control_sizes: [f32; 4],
    pub home_continue_card_height: f32,
    pub poster_card_width: f32,
    pub poster_card_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub display: f32,
    pub title: f32,
    pub subtitle: f32,
    pub body: f32,
    pub meta: f32,
    pub label: f32,
}

const CONTINUE_CARD_SCALE: f32 = 1.05;

static TEXT_PRIMARY: AtomicU32 = AtomicU32::new(u32::MAX);
static BACKGROUND: AtomicU32 = AtomicU32::new(0xFF06_0606);
static SURFACE: AtomicU32 = AtomicU32::new(0xFF14_1416);
static SURFACE_RAISED: AtomicU32 = AtomicU32::new(0xFF1C_1C1F);
static BUTTON_HEIGHT: AtomicU32 = AtomicU32::new(0x4230_0000);
static ICON_BUTTON: AtomicU32 = AtomicU32::new(0x4220_0000);
static CONTROL_RADIUS: AtomicU32 = AtomicU32::new(0x4110_0000);
static DANGER: AtomicU32 = AtomicU32::new(0xFFFF_6B6B);
static ACCENT: AtomicU32 = AtomicU32::new(u32::MAX);
static MOTION: [AtomicU32; 3] = [
    AtomicU32::new(0x3DF5_C28F),
    AtomicU32::new(0x3E38_51EC),
    AtomicU32::new(0x3E99_999A),
];

fn seconds(enabled: bool, ms: u64) -> f32 {
    if enabled { ms as f32 / 1000.0 } else { 0.0 }
}

pub(crate) fn motion_seconds(pace: usize) -> f32 {
    f32::from_bits(MOTION[pace].load(Ordering::Relaxed))
}

fn pack(color: Color32) -> u32 {
    u32::from_be_bytes(color.to_srgba_unmultiplied())
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct MetricsKey {
    epoch: u64,
    width: u32,
    height: u32,
    side_inset: u32,
    tv: bool,
    mobile: bool,
    landscape: bool,
}

static INPUT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn set_input(kind: &str) {
    let code = match kind {
        "touch" => 1,
        "pointer" => 2,
        "focus" => 3,
        _ => 0,
    };
    INPUT.store(code, Ordering::Relaxed);
}

pub(crate) fn input() -> &'static str {
    match INPUT.load(Ordering::Relaxed) {
        1 => "touch",
        2 => "pointer",
        3 => "focus",
        _ => match view().form {
            "mobile" => "touch",
            "tv" => "focus",
            _ => "pointer",
        },
    }
}

static ACTIVE: std::sync::RwLock<Option<(MetricsKey, std::sync::Arc<UiMetrics>)>> =
    std::sync::RwLock::new(None);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    pub platform: &'static str,
    pub form: &'static str,
    pub window_class: &'static str,
    pub width: f32,
    pub height: f32,
}

impl View {
    pub(crate) fn of(viewport: Viewport) -> Self {
        View {
            platform: if viewport.is_compact() {
                "mobile"
            } else if viewport.is_tv() {
                "tv"
            } else {
                "desktop"
            },
            form: match viewport.form_factor {
                UiFormFactor::Mobile => "mobile",
                UiFormFactor::Desktop => "desktop",
                UiFormFactor::Tv => "tv",
            },
            window_class: match viewport.size_class() {
                SizeClass::Compact => "compact",
                SizeClass::Medium => "medium",
                SizeClass::Expanded => "expanded",
            },
            width: viewport.width,
            height: viewport.height,
        }
    }
}

static VIEW: std::sync::RwLock<Option<(MetricsKey, View)>> = std::sync::RwLock::new(None);

fn enter_view(key: MetricsKey, viewport: Viewport) {
    let current = VIEW.read().ok().and_then(|slot| *slot);
    if current.is_some_and(|(active, _)| active == key) {
        return;
    }
    if let Ok(mut slot) = VIEW.write() {
        *slot = Some((key, View::of(viewport)));
    }
}

pub(crate) fn view() -> View {
    VIEW.read()
        .ok()
        .and_then(|slot| slot.map(|(_, view)| view))
        .unwrap_or_else(|| View::of(Viewport::new(1280, 800, UiFormFactor::Desktop)))
}

pub(crate) fn active_key() -> Option<MetricsKey> {
    VIEW.read().ok().and_then(|slot| slot.map(|(key, _)| key))
}

pub(crate) fn active() -> std::sync::Arc<UiMetrics> {
    if let Some((_, metrics)) = ACTIVE.read().ok().and_then(|slot| slot.clone()) {
        return metrics;
    }
    Arc::new(UiMetrics::for_viewport(Viewport::new(
        1280,
        800,
        UiFormFactor::Desktop,
    )))
}

fn publish_palette(key: MetricsKey, shared: &Arc<UiMetrics>) {
    let metrics: &UiMetrics = shared;
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = Some((key, shared.clone()));
    }
    TEXT_PRIMARY.store(pack(metrics.text_primary), Ordering::Relaxed);
    BACKGROUND.store(pack(metrics.background), Ordering::Relaxed);
    SURFACE.store(pack(metrics.surface), Ordering::Relaxed);
    SURFACE_RAISED.store(pack(metrics.surface_raised), Ordering::Relaxed);
    ACCENT.store(pack(metrics.accent), Ordering::Relaxed);
    DANGER.store(pack(metrics.danger), Ordering::Relaxed);
    CONTROL_RADIUS.store(
        crate::vars::px("screen-control-radius").to_bits(),
        Ordering::Relaxed,
    );
    BUTTON_HEIGHT.store(
        crate::vars::px("button-height").to_bits(),
        Ordering::Relaxed,
    );
    ICON_BUTTON.store(
        crate::vars::px("icon-button-size").to_bits(),
        Ordering::Relaxed,
    );
    crate::motion::set_curve(metrics.motion_curve);
    crate::style::set_shades(metrics.shade, crate::vars::byte("overlay-panel-alpha"));
    crate::style::set_tones(metrics.tone, metrics.stroke);
    crate::style::set_control_sizes(metrics.control_sizes);
    for (cell, value) in MOTION.iter().zip([
        metrics.motion_fast,
        metrics.motion_normal,
        metrics.motion_slow,
    ]) {
        cell.store(value.to_bits(), Ordering::Relaxed);
    }
}

fn unpack(cell: &AtomicU32) -> Color32 {
    let [r, g, b, a] = cell.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn surface_color() -> Color32 {
    unpack(&SURFACE)
}

pub fn raised_color() -> Color32 {
    unpack(&SURFACE_RAISED)
}

pub fn button_height() -> f32 {
    f32::from_bits(BUTTON_HEIGHT.load(Ordering::Relaxed))
}

pub fn icon_button_size() -> f32 {
    f32::from_bits(ICON_BUTTON.load(Ordering::Relaxed))
}

pub fn control_radius() -> f32 {
    f32::from_bits(CONTROL_RADIUS.load(Ordering::Relaxed))
}

pub fn accent() -> Color32 {
    unpack(&ACCENT)
}

pub fn danger_color() -> Color32 {
    unpack(&DANGER)
}

pub fn text_primary() -> Color32 {
    let [r, g, b, a] = TEXT_PRIMARY.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn background() -> Color32 {
    let [r, g, b, a] = BACKGROUND.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn ink(alpha: u8) -> Color32 {
    let [r, g, b, a] = TEXT_PRIMARY.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, (u16::from(a) * u16::from(alpha) / 255) as u8)
}

impl UiMetrics {
    pub fn profile_page_width(&self, viewport_width: f32, desktop_max: f32) -> f32 {
        if self.compact {
            viewport_width - crate::vars::px("profile-page-inset-mobile")
        } else {
            (viewport_width - crate::vars::px("profile-page-inset-desktop")).min(desktop_max)
        }
    }

    pub fn ink(&self, alpha: u8) -> Color32 {
        let [r, g, b, a] = self.text_primary.to_srgba_unmultiplied();
        Color32::from_rgba_unmultiplied(r, g, b, (u16::from(a) * u16::from(alpha) / 255) as u8)
    }

    pub fn for_viewport(viewport: Viewport) -> Self {
        *Self::shared(viewport)
    }

    pub(crate) fn shared(viewport: Viewport) -> Arc<Self> {
        thread_local! {
            static RECENT: std::cell::RefCell<Vec<(MetricsKey, Arc<UiMetrics>)>> =
                const { std::cell::RefCell::new(Vec::new()) };
        }
        let key = MetricsKey {
            epoch: fluxa_theme::theme::active_epoch(),
            width: viewport.width.to_bits(),
            height: viewport.height.to_bits(),
            side_inset: viewport.side_inset.to_bits(),
            tv: viewport.is_tv(),
            mobile: viewport.form_factor == UiFormFactor::Mobile,
            landscape: poster_overlay::landscape(),
        };
        enter_view(key, viewport);
        let metrics = RECENT.with(|recent| {
            let mut recent = recent.borrow_mut();
            if let Some((_, hit)) = recent.iter().find(|(cached, _)| *cached == key) {
                return hit.clone();
            }
            let computed = Arc::new(Self::compute(viewport));
            if recent.len() >= 4 {
                recent.remove(0);
            }
            recent.push((key, computed.clone()));
            computed
        });
        let published = ACTIVE
            .read()
            .ok()
            .is_some_and(|slot| slot.as_ref().is_some_and(|(active, _)| *active == key));
        if !published {
            publish_palette(key, &metrics);
        }
        metrics
    }

    fn compute(viewport: Viewport) -> Self {
        let snapshot = active_snapshot(&fluxa_theme::theme::selected_theme_id())
            .expect("shared Fluxa theme must be valid");
        let tokens = &snapshot.tokens;
        let dark = &snapshot.theme;
        let number = |path: &[&str], fallback: f32| {
            let mut value = &tokens.layout;
            for key in path {
                let Some(next) = value.get(*key) else {
                    return fallback;
                };
                value = next;
            }
            value.as_f64().map(|value| value as f32).unwrap_or(fallback)
        };
        let platform = if viewport.is_compact() {
            "mobile"
        } else if viewport.is_tv() {
            "tv"
        } else {
            "desktop"
        };
        let window_class = match viewport.size_class() {
            SizeClass::Compact => "compact",
            SizeClass::Medium => "medium",
            SizeClass::Expanded => "expanded",
        };
        let roomy = !viewport.is_compact() && !viewport.is_tv();
        let pick = |roomy_value: f32, compact_value: f32| {
            if roomy { roomy_value } else { compact_value }
        };
        let common_dp = |key, fallback| number(&["common", "dp", key], fallback);
        let common_sp = |key, fallback| number(&["common", "sp", key], fallback);
        let platform_dp = |key, fallback| number(&["platforms", platform, "dp", key], fallback);
        let platform_scaled = |key: &str, ratio: f32, min: f32, max: f32| {
            tokens
                .layout
                .get("platforms")
                .and_then(|value| value.get(platform))
                .and_then(|value| value.get("scaled"))
                .and_then(|value| value.get(key))
                .and_then(|value| value.as_array())
                .and_then(|values| {
                    let at = |index: usize| values.get(index)?.as_f64().map(|v| v as f32);
                    Some(Scaled {
                        ratio: at(0)?,
                        min: at(1)?,
                        max: at(2)?,
                        offset: at(3).unwrap_or(0.0),
                    })
                })
                .unwrap_or(Scaled {
                    ratio,
                    min,
                    max,
                    offset: 0.0,
                })
        };
        let platform_sp = |key, fallback| number(&["platforms", platform, "sp", key], fallback);
        let window_dp =
            |key, fallback| number(&["windowClasses", window_class, "dp", key], fallback);
        let window_sp =
            |key, fallback| number(&["windowClasses", window_class, "sp", key], fallback);
        let color = |token: &str, fallback: Color32| {
            dark.color(token)
                .map(|value| {
                    Color32::from_rgba_unmultiplied(
                        (value.r * 255.0).round() as u8,
                        (value.g * 255.0).round() as u8,
                        (value.b * 255.0).round() as u8,
                        (value.a * 255.0).round() as u8,
                    )
                })
                .unwrap_or(fallback)
        };
        let mut metrics = Self {
            compact: viewport.is_compact(),
            platform,
            form: match viewport.form_factor {
                UiFormFactor::Mobile => "mobile",
                UiFormFactor::Desktop => "desktop",
                UiFormFactor::Tv => "tv",
            },
            window_class,
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            background: color("background", Color32::from_rgb(7, 7, 9)),
            surface: color("surface", Color32::from_rgb(20, 20, 22)),
            surface_raised: color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            border: color("border", Color32::from_white_alpha(20)),
            danger: color("error", Color32::from_rgb(255, 107, 107)),
            navigation: color("navigation", Color32::from_rgb(20, 20, 22)),
            text_primary: color("textPrimary", Color32::WHITE),
            text_secondary: color("textSecondary", Color32::from_rgb(184, 184, 190)),
            text_muted: color("textMuted", Color32::from_rgb(138, 138, 145)),
            accent: color("accent", Color32::WHITE),
            accent_foreground: color("accentForeground", Color32::WHITE),
            button_primary: color("buttonPrimary", color("textPrimary", Color32::WHITE)),
            button_primary_text: color(
                "buttonPrimaryText",
                color("background", Color32::from_rgb(7, 7, 9)),
            ),
            card_background: color(
                "cardBackground",
                color("surface", Color32::from_rgb(20, 20, 22)),
            ),
            input_background: color(
                "inputBackground",
                color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            ),
            input_border: color(
                "inputBorder",
                color("border", Color32::from_white_alpha(20)),
            ),
            input_border_focus: dark
                .color("inputBorderFocus")
                .is_ok()
                .then(|| color("inputBorderFocus", Color32::TRANSPARENT)),
            focus: color("focus", Color32::WHITE),
            motion_fast: seconds(dark.motion.enabled, dark.motion.fast_ms),
            motion_normal: seconds(dark.motion.enabled, dark.motion.normal_ms),
            motion_slow: seconds(dark.motion.enabled, dark.motion.slow_ms),
            motion_curve: [
                number(&["common", "number", "motionCurveX1"], 0.2),
                number(&["common", "number", "motionCurveY1"], 0.0),
                number(&["common", "number", "motionCurveX2"], 0.0),
                number(&["common", "number", "motionCurveY2"], 1.0),
            ],
            // Home's Continue Watching row uses the same responsive size
            // presets as Compose horizontalCardWidth/Height, not the larger
            // episode-card dimensions used by other surfaces.
            home_continue_card_width: window_dp("horizontalCardBase", 196.0) * CONTINUE_CARD_SCALE,
            home_continue_card_height: window_dp("horizontalCardBase", 196.0)
                * CONTINUE_CARD_SCALE
                * number(&["common", "number", "horizontalCardHeightRatio"], 0.56),
            poster_card_width: platform_dp("posterCardWidth", 160.0),
            poster_card_height: platform_dp("posterCardHeight", 240.0),
            radius: [common_dp("radiusMd", 12.0), common_dp("radiusSheet", 24.0)],
            icon: [
                common_dp("iconSm", 16.0),
                common_dp("iconMd", 22.0),
                common_dp("iconXl", 36.0),
            ],
            sheet_handle: Vec2::new(
                common_dp("sheetHandleWidth", 36.0),
                common_dp("sheetHandleHeight", 4.0),
            ),
            tone: [
                common_dp("toneFaint", 10.0) as u8,
                common_dp("toneSoft", 20.0) as u8,
                common_dp("toneRaised", 30.0) as u8,
                common_dp("toneStrong", 50.0) as u8,
                common_dp("toneDim", 90.0) as u8,
                common_dp("toneMuted", 130.0) as u8,
                common_dp("toneSubtle", 170.0) as u8,
                common_dp("toneBody", 210.0) as u8,
                common_dp("toneText", 235.0) as u8,
                common_dp("toneFull", 255.0) as u8,
            ],
            control_sizes: [
                number(&["common", "number", "iconButtonGlyphRatio"], 0.44),
                common_dp("sliderTrackHeight", 4.0),
                common_dp("sliderKnobRadius", 6.0),
                common_dp("sliderMarkHalf", 5.0),
            ],
            stroke: [
                common_dp("strokeHairline", 1.0),
                common_dp("strokeIcon", 1.6),
                common_dp("strokeEmphasis", 2.0),
            ],
            shade: [
                common_dp("shadeFaint", 90.0) as u8,
                common_dp("shadeVeil", 150.0) as u8,
                common_dp("shadeScrim", 180.0) as u8,
                common_dp("shadeStrong", 215.0) as u8,
                common_dp("shadeOpaque", 240.0) as u8,
            ],
            space: [
                common_dp("space2", 2.0),
                common_dp("space4", 4.0),
                common_dp("space6", 6.0),
                common_dp("space8", 8.0),
                common_dp("space10", 10.0),
                common_dp("space12", 12.0),
                common_dp("space14", 14.0),
                common_dp("space16", 16.0),
                common_dp("space20", 20.0),
                common_dp("space24", 24.0),
                common_dp("space32", 32.0),
                common_dp("space48", 48.0),
            ],
        };
        let visible = crate::vars::px("poster-visible-cards");
        let gap = crate::vars::px("horizontal-spacing");
        let content = viewport.width - crate::vars::px("page-padding") * 2.0;
        let width = ((content - gap * (visible.ceil() - 1.0)) / visible).clamp(
            crate::vars::px("poster-card-min"),
            crate::vars::px("poster-card-max"),
        );
        metrics.poster_card_width = width;
        metrics.poster_card_height = width * crate::vars::px("poster-height-ratio");
        let continue_visible = crate::vars::px("continue-visible-cards");
        let width = ((content - gap * (continue_visible.ceil() - 1.0)) / continue_visible).clamp(
            crate::vars::px("continue-card-min"),
            crate::vars::px("continue-card-max"),
        );
        metrics.home_continue_card_width = width * CONTINUE_CARD_SCALE;
        metrics.home_continue_card_height =
            width * CONTINUE_CARD_SCALE * crate::vars::px("horizontal-card-height-ratio");
        if poster_overlay::landscape() {
            metrics.poster_card_width = metrics.home_continue_card_width;
            metrics.poster_card_height = metrics.poster_card_width * 0.5625;
        }
        metrics
    }

    pub fn navigation_label_size(self) -> f32 {
        crate::vars::px("nav-label-size") + crate::vars::px("nav-label-boost")
    }

    pub fn navigation_icon_size(self) -> f32 {
        crate::vars::px("navigation-icon")
    }

    pub fn navigation_item_width(self, label: &str) -> f32 {
        let full = 44.0
            + self.navigation_icon_size()
            + 8.0
            + estimated_navigation_text_width(label, self.navigation_label_size());
        let compact = crate::vars::px("nav-item-height").max(self.navigation_icon_size() + 24.0);
        compact + (full - compact) * crate::navigation::top_label_t()
    }

    pub fn navigation_profile_width(self, profile_name: &str) -> f32 {
        let full = (36.0
            + crate::vars::px("nav-avatar-radius") * 2.0
            + 8.0
            + estimated_navigation_text_width(profile_name, self.navigation_label_size()))
        .clamp(72.0, 200.0);
        let compact = crate::vars::px("nav-item-height");
        compact + (full - compact) * crate::navigation::top_label_t()
    }

    pub fn navigation_bar_width(self, viewport: Viewport, profile_name: &str) -> f32 {
        let compact = viewport.is_compact();
        let tv = viewport.is_tv();
        let margin = if compact {
            crate::vars::px("page-padding")
        } else if tv {
            crate::vars::px("page-padding").max(32.0)
        } else {
            crate::vars::px("page-padding")
        };
        let entries = crate::navigation::nav_entries();
        let nav_width = entries
            .iter()
            .map(|(_, label)| self.navigation_item_width(label))
            .sum::<f32>();
        (if compact {
            (viewport.width - margin * 2.0).max(280.0)
        } else {
            nav_width
                + crate::vars::px("nav-pill-gap") * (entries.len() + 1) as f32
                + self.navigation_profile_width(profile_name)
        })
        .min((viewport.width - margin * 2.0).max(1.0))
    }
}
