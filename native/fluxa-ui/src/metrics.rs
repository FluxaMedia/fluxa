use super::*;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct MetricsKey {
    epoch: u64,
    generation: u64,
    width: u32,
    height: u32,
    side_inset: u32,
    tv: bool,
    mobile: bool,
    landscape: bool,
}

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

    pub(crate) fn compact(&self) -> bool {
        self.platform == "mobile"
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Theme {
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub danger: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub button_primary: Color32,
    pub button_primary_text: Color32,
    pub input_background: Color32,
    pub input_border: Color32,
    pub input_border_focus: Option<Color32>,
    pub space: [f32; 12],
    pub radius: [f32; 2],
    pub icon: [f32; 3],
    pub sheet_handle: Vec2,
    pub poster_width: f32,
    pub poster_height: f32,
    pub continue_width: f32,
    pub continue_height: f32,
}

const CONTINUE_CARD_SCALE: f32 = 1.05;

static VIEW: RwLock<Option<(MetricsKey, View)>> = RwLock::new(None);
static THEME: RwLock<Option<(MetricsKey, Theme)>> = RwLock::new(None);

static BUTTON_HEIGHT: AtomicU32 = AtomicU32::new(0x4230_0000);
static ICON_BUTTON: AtomicU32 = AtomicU32::new(0x4220_0000);
static CONTROL_RADIUS: AtomicU32 = AtomicU32::new(0x4110_0000);
static MOTION: [AtomicU32; 3] = [
    AtomicU32::new(0x3DF5_C28F),
    AtomicU32::new(0x3E38_51EC),
    AtomicU32::new(0x3E99_999A),
];
static INPUT: AtomicU8 = AtomicU8::new(0);

pub(crate) fn motion_seconds(pace: usize) -> f32 {
    f32::from_bits(MOTION[pace].load(Ordering::Relaxed))
}

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

pub(crate) fn view() -> View {
    VIEW.read()
        .ok()
        .and_then(|slot| slot.map(|(_, view)| view))
        .unwrap_or_else(|| View::of(Viewport::new(1280, 800, UiFormFactor::Desktop)))
}

pub(crate) fn active_key() -> Option<MetricsKey> {
    VIEW.read().ok().and_then(|slot| slot.map(|(key, _)| key))
}

pub(crate) fn theme() -> Theme {
    if let Some((_, theme)) = THEME.read().ok().and_then(|slot| *slot) {
        return theme;
    }
    enter(Viewport::new(1280, 800, UiFormFactor::Desktop));
    THEME
        .read()
        .ok()
        .and_then(|slot| slot.map(|(_, theme)| theme))
        .expect("theme is built on first use")
}

pub(crate) fn enter(viewport: Viewport) {
    let key = MetricsKey {
        epoch: fluxa_theme::theme::active_epoch(),
        generation: crate::vars::generation(),
        width: viewport.width.to_bits(),
        height: viewport.height.to_bits(),
        side_inset: viewport.side_inset.to_bits(),
        tv: viewport.is_tv(),
        mobile: viewport.form_factor == UiFormFactor::Mobile,
        landscape: poster_overlay::landscape(),
    };
    if active_key() != Some(key) {
        if let Ok(mut slot) = VIEW.write() {
            *slot = Some((key, View::of(viewport)));
        }
    }
    let built = THEME
        .read()
        .ok()
        .is_some_and(|slot| slot.is_some_and(|(built, _)| built == key));
    if built {
        return;
    }
    let theme = build(viewport);
    publish(&theme);
    if let Ok(mut slot) = THEME.write() {
        *slot = Some((key, theme));
    }
}

fn color(name: &str, fallback: Color32) -> Color32 {
    crate::vars::color(name).unwrap_or(fallback)
}

fn build(viewport: Viewport) -> Theme {
    let background = color("color-background", Color32::from_rgb(7, 7, 9));
    let surface = color("color-surface", Color32::from_rgb(20, 20, 22));
    let surface_raised = color("color-surface-raised", Color32::from_rgb(28, 28, 31));
    let border = color("color-border", Color32::from_white_alpha(20));
    let text_primary = color("color-text-primary", Color32::WHITE);
    let mut theme = Theme {
        background,
        surface,
        surface_raised,
        border,
        danger: color("color-error", Color32::from_rgb(255, 107, 107)),
        text_primary,
        text_secondary: color("color-text-secondary", Color32::from_rgb(184, 184, 190)),
        text_muted: color("color-text-muted", Color32::from_rgb(138, 138, 145)),
        accent: color("color-accent", Color32::WHITE),
        accent_foreground: color("color-accent-foreground", Color32::WHITE),
        button_primary: color("color-button-primary", text_primary),
        button_primary_text: color("color-button-primary-text", background),
        input_background: color("color-input-background", surface_raised),
        input_border: color("color-input-border", border),
        input_border_focus: crate::vars::color("color-input-border-focus"),
        space: [2, 4, 6, 8, 10, 12, 14, 16, 20, 24, 32, 48]
            .map(|step| crate::vars::px(&format!("space-{step}"))),
        radius: [
            crate::vars::px("radius-md"),
            crate::vars::px("radius-sheet"),
        ],
        icon: [
            crate::vars::px("icon-sm"),
            crate::vars::px("icon-md"),
            crate::vars::px("icon-xl"),
        ],
        sheet_handle: Vec2::new(
            crate::vars::px("sheet-handle-width"),
            crate::vars::px("sheet-handle-height"),
        ),
        poster_width: 0.0,
        poster_height: 0.0,
        continue_width: 0.0,
        continue_height: 0.0,
    };
    let gap = crate::vars::px("horizontal-spacing");
    let content = viewport.width - crate::vars::px("page-padding") * 2.0;
    let visible = crate::vars::px("poster-visible-cards");
    theme.poster_width = crate::vars::scaled(
        "poster-card",
        (content - gap * (visible.ceil() - 1.0)) / visible,
    );
    theme.poster_height = theme.poster_width * crate::vars::px("poster-height-ratio");
    let visible = crate::vars::px("continue-visible-cards");
    let width = crate::vars::scaled(
        "continue-card",
        (content - gap * (visible.ceil() - 1.0)) / visible,
    );
    theme.continue_width = width * CONTINUE_CARD_SCALE;
    theme.continue_height =
        width * CONTINUE_CARD_SCALE * crate::vars::px("horizontal-card-height-ratio");
    if poster_overlay::landscape() {
        theme.poster_width = theme.continue_width;
        theme.poster_height = theme.poster_width * 0.5625;
    }
    theme
}

fn publish(theme: &Theme) {
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
    crate::motion::set_curve([
        crate::vars::px("motion-curve-x1"),
        crate::vars::px("motion-curve-y1"),
        crate::vars::px("motion-curve-x2"),
        crate::vars::px("motion-curve-y2"),
    ]);
    crate::style::set_shades(
        ["faint", "veil", "scrim", "strong", "opaque"]
            .map(|name| crate::vars::byte(&format!("shade-{name}"))),
        crate::vars::byte("overlay-panel-alpha"),
    );
    crate::style::set_tones(
        [
            "faint", "soft", "raised", "strong", "dim", "muted", "subtle", "body", "text", "full",
        ]
        .map(|name| crate::vars::byte(&format!("tone-{name}"))),
        ["hairline", "icon", "emphasis"].map(|name| crate::vars::px(&format!("stroke-{name}"))),
    );
    crate::style::set_control_sizes([
        crate::vars::px("icon-button-glyph-ratio"),
        crate::vars::px("slider-track-height"),
        crate::vars::px("slider-knob-radius"),
        crate::vars::px("slider-mark-half"),
    ]);
    let enabled = crate::vars::px("motion-enabled") > 0.0;
    for (cell, name) in MOTION.iter().zip(["fast", "normal", "slow"]) {
        let seconds = if enabled {
            crate::vars::px(&format!("motion-{name}-ms")) / 1000.0
        } else {
            0.0
        };
        cell.store(seconds.to_bits(), Ordering::Relaxed);
    }
    let _ = theme;
}

pub fn surface_color() -> Color32 {
    theme().surface
}

pub fn raised_color() -> Color32 {
    theme().surface_raised
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
    theme().accent
}

pub fn danger_color() -> Color32 {
    theme().danger
}

pub fn text_primary() -> Color32 {
    theme().text_primary
}

pub fn background() -> Color32 {
    theme().background
}

pub fn ink(alpha: u8) -> Color32 {
    let [r, g, b, a] = theme().text_primary.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (u16::from(a) * u16::from(alpha) / 255) as u8)
}

pub fn profile_page_width(viewport_width: f32, desktop_max: f32) -> f32 {
    if view().compact() {
        viewport_width - crate::vars::px("profile-page-inset-mobile")
    } else {
        (viewport_width - crate::vars::px("profile-page-inset-desktop")).min(desktop_max)
    }
}

pub fn navigation_label_size() -> f32 {
    crate::vars::px("nav-label-size") + crate::vars::px("nav-label-boost")
}

pub fn navigation_icon_size() -> f32 {
    crate::vars::px("navigation-icon")
}
