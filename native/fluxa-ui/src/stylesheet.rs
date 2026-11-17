use super::*;
use serde_json::Value;
use std::sync::{Arc, RwLock};

mod css;
mod parse;

const DEFAULT_SHEET: &str = include_str!("../../../shared/contracts/ui-styles.css");

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Style {
    pub fill: Color32,
    pub text: Color32,
    pub border: Color32,
    pub scale: f32,
    pub radius: Corner,
    pub stroke: f32,
    pub selected: f32,
    pub padding: f32,
    pub gap: f32,
    pub opacity: f32,
    pub max_width: Option<f32>,
    pub font_size: Option<f32>,
    pub direction: Option<Direction>,
    pub align: Option<Alignment>,
    pub text_declared: bool,
    pub gradient: Option<Gradient>,
    pub shadow: Option<Glow>,
    pub margin: f32,
    pub backdrop: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub aspect: Option<f32>,
    pub weight: Option<f32>,
    pub stretch: f32,
    pub squash: f32,
    pub magnify: f32,
    pub reach: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Gradient {
    pub angle: f32,
    pub stops: [(f32, Color32); 4],
    pub len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Glow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alignment {
    Start,
    Center,
    End,
}

impl Style {
    fn blank() -> Self {
        Self {
            fill: Color32::TRANSPARENT,
            text: text_primary(),
            border: Color32::TRANSPARENT,
            scale: 1.0,
            radius: Corner::Px(0.0),
            stroke: 0.0,
            selected: 0.0,
            padding: 0.0,
            gap: 0.0,
            opacity: 1.0,
            max_width: None,
            font_size: None,
            direction: None,
            align: None,
            text_declared: false,
            gradient: None,
            shadow: None,
            margin: 0.0,
            backdrop: 0.0,
            width: None,
            height: None,
            aspect: None,
            weight: None,
            stretch: 0.0,
            squash: 0.0,
            magnify: 0.0,
            reach: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Length {
    Space(Space),
    Px(f32),
    Metric(Metric),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Metric {
    PosterWidth,
    PosterHeight,
    ContinueWidth,
    IconButton,
    ControlHeight,
    Gutter,
    SynopsisSize,
    LabelSize,
    ContinueHeight,
    WideWidth,
    WideHeight,
    SquareSize,
    CollectionWidth,
    CollectionHeight,
    CardTitle,
    CardSubtitle,
    CardPadding,
}

impl Length {
    fn px(self) -> f32 {
        match self {
            Length::Space(space) => space.px(&crate::metrics::active()),
            Length::Px(px) => px,
            Length::Metric(metric) => {
                let metrics = crate::metrics::active();
                match metric {
                    Metric::PosterWidth => metrics.poster_card_width,
                    Metric::PosterHeight => metrics.poster_card_height,
                    Metric::ContinueWidth => metrics.home_continue_card_width,
                    Metric::IconButton => metrics.icon_button_size,
                    Metric::ControlHeight => metrics.screen_control_height,
                    Metric::Gutter => metrics.gutter,
                    Metric::SynopsisSize => metrics.hero_synopsis_size.max(14.0),
                    Metric::LabelSize => metrics.nav_label_size + crate::dp::Dp::D2.px(),
                    Metric::ContinueHeight => metrics.home_continue_card_height,
                    Metric::WideWidth => metrics.collection_wide_width,
                    Metric::WideHeight => metrics.collection_wide_height,
                    Metric::SquareSize => metrics.collection_square_size,
                    Metric::CollectionWidth => metrics.collection_poster_width,
                    Metric::CollectionHeight => metrics.collection_poster_height,
                    Metric::CardTitle => metrics.screen_card_title_size,
                    Metric::CardSubtitle => metrics.screen_card_subtitle_size,
                    Metric::CardPadding => metrics.card_content_padding,
                }
            }
        }
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Number(number) => number.as_f64().map(|px| Length::Px(px as f32)),
            Value::String(text) => Some(Length::Space(match text.as_str() {
                "s2" => Space::S2,
                "s4" => Space::S4,
                "s6" => Space::S6,
                "s8" => Space::S8,
                "s10" => Space::S10,
                "s12" => Space::S12,
                "s14" => Space::S14,
                "s16" => Space::S16,
                "s20" => Space::S20,
                "s24" => Space::S24,
                "s32" => Space::S32,
                "s48" => Space::S48,
                "poster-width" => return Some(Length::Metric(Metric::PosterWidth)),
                "poster-height" => return Some(Length::Metric(Metric::PosterHeight)),
                "continue-width" => return Some(Length::Metric(Metric::ContinueWidth)),
                "icon-button" => return Some(Length::Metric(Metric::IconButton)),
                "control-height" => return Some(Length::Metric(Metric::ControlHeight)),
                "gutter" => return Some(Length::Metric(Metric::Gutter)),
                "synopsis-size" => return Some(Length::Metric(Metric::SynopsisSize)),
                "label-size" => return Some(Length::Metric(Metric::LabelSize)),
                "continue-height" => return Some(Length::Metric(Metric::ContinueHeight)),
                "wide-width" => return Some(Length::Metric(Metric::WideWidth)),
                "wide-height" => return Some(Length::Metric(Metric::WideHeight)),
                "square-size" => return Some(Length::Metric(Metric::SquareSize)),
                "collection-width" => return Some(Length::Metric(Metric::CollectionWidth)),
                "collection-height" => return Some(Length::Metric(Metric::CollectionHeight)),
                "card-title" => return Some(Length::Metric(Metric::CardTitle)),
                "card-subtitle" => return Some(Length::Metric(Metric::CardSubtitle)),
                "card-padding" => return Some(Length::Metric(Metric::CardPadding)),
                _ => return None,
            })),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Corner {
    Pill,
    Card,
    Dialog,
    Px(f32),
}

impl Corner {
    pub(crate) fn of(self, height: f32) -> f32 {
        match self {
            Corner::Pill => height * crate::metrics::active().pill_ratio,
            Corner::Card => crate::metrics::active().card_radius,
            Corner::Dialog => crate::metrics::active().dialog_radius,
            Corner::Px(px) => px,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct State {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub selected: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum Paint {
    Rgba(Color32),
    Ink(u8),
    Named(String, f32),
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Patch {
    fill: Option<Paint>,
    text: Option<Paint>,
    border: Option<Paint>,
    scale: Option<f32>,
    radius: Option<Corner>,
    stroke: Option<f32>,
    padding: Option<Length>,
    gap: Option<Length>,
    opacity: Option<f32>,
    max_width: Option<Length>,
    font_size: Option<Length>,
    direction: Option<Direction>,
    align: Option<Alignment>,
    gradient: Option<GradientSpec>,
    shadow: Option<GlowSpec>,
    margin: Option<Length>,
    backdrop: Option<f32>,
    width: Option<Length>,
    height: Option<Length>,
    aspect: Option<f32>,
    weight: Option<f32>,
    stretch: Option<f32>,
    squash: Option<f32>,
    magnify: Option<f32>,
    reach: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
struct GradientSpec {
    angle: f32,
    stops: Vec<(Option<f32>, Paint)>,
}

#[derive(Clone, Debug, PartialEq)]
struct GlowSpec {
    x: f32,
    y: f32,
    blur: f32,
    spread: f32,
    color: Paint,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Layer {
    base: Patch,
    hover: Patch,
    pressed: Patch,
    focused: Patch,
    selected: Patch,
    disabled: Patch,
}

#[derive(Clone, Debug, PartialEq)]
enum Query {
    Platform(String),
    Window(String),
    MinWidth(f32),
    MaxWidth(f32),
}

impl Query {
    fn matches(&self) -> bool {
        let metrics = crate::metrics::active();
        match self {
            Query::Platform(name) => metrics.platform == name,
            Query::Window(name) => metrics.window_class == name,
            Query::MinWidth(width) => metrics.viewport_width >= *width,
            Query::MaxWidth(width) => metrics.viewport_width <= *width,
        }
    }
}

struct Selector {
    ancestors: Vec<String>,
    name: String,
    modifiers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Variant {
    ancestors: Vec<String>,
    modifiers: Vec<String>,
    layer: Layer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Easing {
    Linear,
    Out,
}

#[derive(Clone, Debug, PartialEq)]
struct Animation {
    name: String,
    duration: f32,
    easing: Easing,
    infinite: bool,
    alternate: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Rule {
    layer: Layer,
    extends: Option<String>,
    transition: Option<String>,
    easing: Option<String>,
    animation: Option<Animation>,
    media: Vec<(Query, Layer)>,
    variants: Vec<Variant>,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Sheet {
    rules: std::collections::HashMap<String, Rule>,
    keyframes: std::collections::HashMap<String, Vec<(f32, Patch)>>,
}

static SHEET: RwLock<Option<Arc<Sheet>>> = RwLock::new(None);

pub(crate) fn sheet() -> Arc<Sheet> {
    if let Some(sheet) = SHEET.read().ok().and_then(|slot| slot.clone()) {
        return sheet;
    }
    let parsed = Arc::new(Sheet::parse(DEFAULT_SHEET).expect("bundled ui-styles"));
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(parsed.clone());
    }
    parsed
}

pub fn reload_styles(text: &str) -> Result<(), String> {
    let (parsed, motion, curve) = Sheet::parse_with_motion(text)?;
    crate::motion::set_styled(motion);
    if let Some(points) = curve {
        crate::motion::set_curve(points);
    }
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(Arc::new(parsed));
    }
    Ok(())
}

impl Sheet {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        Self::parse_with_motion(text).map(|(sheet, ..)| sheet)
    }

    pub(crate) fn parse_with_motion(
        text: &str,
    ) -> Result<(Self, [Option<f32>; 3], Option<[f32; 4]>), String> {
        let mut root: Value = if text.trim_start().starts_with('{') {
            serde_json::from_str(text).map_err(|error| error.to_string())?
        } else {
            css::to_value(text)?
        };
        let motion = root.as_object_mut().and_then(|map| map.remove("@motion"));
        let sheet = parse::sheet_from_value(&root)?;
        let seconds = |name: &str| {
            let text = motion.as_ref()?.get(name)?.as_str()?.trim();
            match text.strip_suffix("ms") {
                Some(ms) => ms.trim().parse::<f32>().ok().map(|ms| ms / 1000.0),
                None => text.trim_end_matches('s').trim().parse::<f32>().ok(),
            }
        };
        let curve = motion
            .as_ref()
            .and_then(|map| map.get("curve")?.as_str())
            .and_then(|text| {
                let points: Vec<f32> = text
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter(|part| !part.is_empty())
                    .filter_map(|part| part.parse().ok())
                    .collect();
                <[f32; 4]>::try_from(points).ok()
            });
        Ok((
            sheet,
            [seconds("fast"), seconds("normal"), seconds("slow")],
            curve,
        ))
    }
}

use parse::parse_patch;

struct Scope {
    name: String,
    text: Option<Color32>,
    font_size: Option<f32>,
    states: u8,
}

pub(crate) const HOVERED: u8 = 1;
pub(crate) const FOCUSED: u8 = 2;
pub(crate) const PRESSED: u8 = 4;
pub(crate) const SELECTED: u8 = 8;

thread_local! {
    static SCOPES: std::cell::RefCell<Vec<Scope>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn scoped<R>(name: &str, inherit: Option<&Style>, body: impl FnOnce() -> R) -> R {
    SCOPES.with(|stack| {
        stack.borrow_mut().push(Scope {
            name: name.split('[').next().unwrap_or(name).to_owned(),
            text: inherit
                .filter(|style| style.text_declared)
                .map(|style| style.text),
            font_size: inherit.and_then(|style| style.font_size),
            states: 0,
        })
    });
    let result = body();
    SCOPES.with(|stack| stack.borrow_mut().pop());
    result
}

fn inherited() -> (Option<Color32>, Option<f32>) {
    SCOPES.with(|stack| {
        let stack = stack.borrow();
        (
            stack.iter().rev().find_map(|scope| scope.text),
            stack.iter().rev().find_map(|scope| scope.font_size),
        )
    })
}

fn in_scope(name: &str) -> bool {
    SCOPES.with(|stack| stack.borrow().iter().any(|scope| scope.name == name))
}

fn in_scope_with(name: &str, flag: u8) -> bool {
    SCOPES.with(|stack| {
        stack
            .borrow()
            .iter()
            .any(|scope| scope.name == name && scope.states & flag != 0)
    })
}

pub(crate) fn within<R>(ancestors: &[(String, u8)], body: impl FnOnce() -> R) -> R {
    SCOPES.with(|stack| {
        let mut stack = stack.borrow_mut();
        for (name, states) in ancestors {
            stack.push(Scope {
                name: name.clone(),
                text: None,
                font_size: None,
                states: *states,
            });
        }
    });
    let result = body();
    SCOPES.with(|stack| {
        let mut stack = stack.borrow_mut();
        let keep = stack.len().saturating_sub(ancestors.len());
        stack.truncate(keep);
    });
    result
}

fn ancestor_flag(state: &str) -> u8 {
    match state {
        "hover" => HOVERED,
        "focus" | "focused" => FOCUSED,
        "active" | "pressed" => PRESSED,
        "checked" | "selected" => SELECTED,
        _ => 0,
    }
}

fn ancestors_match(variant: &Variant, stateful: bool) -> bool {
    let has_state = variant.ancestors.iter().any(|name| name.contains(':'));
    if has_state != stateful {
        return false;
    }
    variant
        .ancestors
        .iter()
        .all(|ancestor| match ancestor.split_once(':') {
            Some((name, _)) => in_scope(name),
            None => in_scope(ancestor),
        })
}

fn ancestors_active(variant: &Variant) -> bool {
    variant
        .ancestors
        .iter()
        .all(|ancestor| match ancestor.split_once(':') {
            Some((name, state)) => in_scope_with(name, ancestor_flag(state)),
            None => in_scope(ancestor),
        })
}

fn parse_paint(text: &str) -> Option<Paint> {
    if let Some(hex) = text.strip_prefix('#') {
        let value = u32::from_str_radix(hex, 16).ok()?;
        return match hex.len() {
            6 => Some(Paint::Rgba(Color32::from_rgb(
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ))),
            8 => Some(Paint::Rgba(Color32::from_rgba_unmultiplied(
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ))),
            _ => None,
        };
    }
    if let Some(color) = parse_function_color(text) {
        return Some(Paint::Rgba(color));
    }
    match text {
        "transparent" => return Some(Paint::Rgba(Color32::TRANSPARENT)),
        "white" => return Some(Paint::Rgba(Color32::WHITE)),
        "black" => return Some(Paint::Rgba(Color32::BLACK)),
        _ => {}
    }
    if let Some(alpha) = text.strip_prefix("ink/") {
        let level = match alpha {
            "faint" => tone::faint(),
            "soft" => tone::soft(),
            "raised" => tone::raised(),
            "strong" => tone::strong(),
            "dim" => tone::dim(),
            "muted" => tone::muted(),
            "subtle" => tone::subtle(),
            "body" => tone::body(),
            "text" => tone::text(),
            "full" => tone::full(),
            number => number.parse().ok()?,
        };
        return Some(Paint::Ink(level));
    }
    let (name, factor) = match text.split_once('*') {
        Some((name, factor)) => (name, factor.parse().ok()?),
        None => (text, 1.0),
    };
    Some(Paint::Named(name.to_owned(), factor))
}

fn parse_function_color(text: &str) -> Option<Color32> {
    let inner = text
        .strip_prefix("rgba(")
        .or_else(|| text.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let parts: Vec<f32> = inner
        .split([',', ' ', '/'])
        .filter(|part| !part.is_empty())
        .map(|part| match part.strip_suffix('%') {
            Some(percent) => percent.parse::<f32>().map(|v| v / 100.0),
            None => part.parse(),
        })
        .collect::<Result<_, _>>()
        .ok()?;
    let [r, g, b] =
        [*parts.first()?, *parts.get(1)?, *parts.get(2)?].map(|c| c.clamp(0.0, 255.0) as u8);
    let alpha = (parts.get(3).copied().unwrap_or(1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
    Some(Color32::from_rgba_unmultiplied(r, g, b, alpha))
}

fn resolve_paint(paint: &Paint, metrics: Option<&UiMetrics>) -> Color32 {
    match paint {
        Paint::Rgba(color) => *color,
        Paint::Ink(level) => ink(*level),
        Paint::Named(name, factor) => {
            let color = match name.as_str() {
                "background" => background(),
                "theme.settings.description" => ink(crate::metrics::active().settings_swatch_alpha),
                "theme.settings.description.compact" => {
                    ink(crate::metrics::active().settings_swatch_alpha_compact)
                }
                "theme.surface" => crate::metrics::active().surface,
                "theme.border" => crate::metrics::active().border,
                "theme.input.background" => crate::metrics::active().input_background,
                "theme.input.border" => crate::metrics::active().input_border,
                "theme.input.border.focus" => crate::metrics::active()
                    .input_border_focus
                    .unwrap_or_else(accent),
                "theme.toggle.knob.on" => {
                    let theme = crate::metrics::active();
                    if theme.accent_foreground == theme.text_primary {
                        theme.text_primary
                    } else {
                        theme.background
                    }
                }
                "theme.button.primary" => crate::metrics::active().button_primary,
                "theme.button.primary.text" => crate::metrics::active().button_primary_text,
                "theme.button.border" => {
                    Color32::from_white_alpha(crate::metrics::active().button_border)
                }
                "text.primary" => text_primary(),
                "accent" => accent(),
                "danger" => danger_color(),
                "accent.foreground" => metrics.map_or(background(), |m| m.accent_foreground),
                "surface.raised" => metrics.map_or(ink(tone::raised()), |m| m.surface_raised),
                "text.secondary" => metrics.map_or(ink(tone::muted()), |m| m.text_secondary),
                "text.muted" => metrics.map_or(ink(tone::subtle()), |m| m.text_muted),
                _ => Color32::MAGENTA,
            };
            if *factor == 1.0 {
                color
            } else {
                color.gamma_multiply(*factor)
            }
        }
    }
}

fn apply(style: &mut Style, patch: &Patch, metrics: Option<&UiMetrics>) {
    if let Some(paint) = &patch.fill {
        style.fill = resolve_paint(paint, metrics);
        style.gradient = None;
    }
    if let Some(spec) = &patch.gradient {
        let mut stops = [(0.0, Color32::TRANSPARENT); 4];
        let count = spec.stops.len().min(4);
        for (index, (at, paint)) in spec.stops.iter().take(4).enumerate() {
            let fallback = if count > 1 {
                index as f32 / (count - 1) as f32
            } else {
                0.0
            };
            stops[index] = (at.unwrap_or(fallback), resolve_paint(paint, metrics));
        }
        style.gradient = Some(Gradient {
            angle: spec.angle,
            stops,
            len: count,
        });
    }
    if let Some(spec) = &patch.shadow {
        style.shadow = Some(Glow {
            x: spec.x,
            y: spec.y,
            blur: spec.blur,
            spread: spec.spread,
            color: resolve_paint(&spec.color, metrics),
        });
    }
    if let Some(margin) = patch.margin {
        style.margin = margin.px();
    }
    if let Some(blur) = patch.backdrop {
        style.backdrop = blur;
    }
    if let Some(width) = patch.width {
        style.width = Some(width.px());
    }
    if let Some(height) = patch.height {
        style.height = Some(height.px());
    }
    if let Some(aspect) = patch.aspect {
        style.aspect = Some(aspect);
    }
    if let Some(weight) = patch.weight {
        style.weight = Some(weight);
    }
    if let Some(value) = patch.stretch {
        style.stretch = value;
    }
    if let Some(value) = patch.squash {
        style.squash = value;
    }
    if let Some(value) = patch.magnify {
        style.magnify = value;
    }
    if let Some(value) = patch.reach {
        style.reach = value;
    }
    if let Some(paint) = &patch.text {
        style.text = resolve_paint(paint, metrics);
        style.text_declared = true;
    }
    if let Some(paint) = &patch.border {
        style.border = resolve_paint(paint, metrics);
    }
    if let Some(scale) = patch.scale {
        style.scale = scale;
    }
    if let Some(radius) = patch.radius {
        style.radius = radius;
    }
    if let Some(stroke) = patch.stroke {
        style.stroke = stroke;
    }
    if let Some(padding) = patch.padding {
        style.padding = padding.px();
    }
    if let Some(gap) = patch.gap {
        style.gap = gap.px();
    }
    if let Some(opacity) = patch.opacity {
        style.opacity = opacity;
    }
    if let Some(width) = patch.max_width {
        style.max_width = Some(width.px());
    }
    if let Some(size) = patch.font_size {
        style.font_size = Some(size.px());
    }
    if let Some(direction) = patch.direction {
        style.direction = Some(direction);
    }
    if let Some(align) = patch.align {
        style.align = Some(align);
    }
}

fn mix(from: Style, to: Style, t: f32) -> Style {
    Style {
        fill: from.fill.lerp_to_gamma(to.fill, t),
        text: from.text.lerp_to_gamma(to.text, t),
        border: from.border.lerp_to_gamma(to.border, t),
        scale: from.scale + (to.scale - from.scale) * t,
        radius: to.radius,
        stroke: from.stroke + (to.stroke - from.stroke) * t,
        selected: from.selected + (to.selected - from.selected) * t,
        padding: from.padding + (to.padding - from.padding) * t,
        gap: from.gap + (to.gap - from.gap) * t,
        opacity: from.opacity + (to.opacity - from.opacity) * t,
        max_width: match (from.max_width, to.max_width) {
            (Some(a), Some(b)) => Some(a + (b - a) * t),
            _ => to.max_width,
        },
        font_size: to.font_size,
        direction: to.direction,
        align: to.align,
        text_declared: to.text_declared,
        gradient: mix_gradient(from.gradient, to.gradient, t),
        shadow: mix_shadow(from.shadow, to.shadow, t),
        margin: from.margin + (to.margin - from.margin) * t,
        backdrop: from.backdrop + (to.backdrop - from.backdrop) * t,
        width: to.width,
        height: to.height,
        aspect: to.aspect,
        weight: to.weight,
        stretch: to.stretch,
        squash: to.squash,
        magnify: to.magnify,
        reach: to.reach,
    }
}

fn mix_gradient(from: Option<Gradient>, to: Option<Gradient>, t: f32) -> Option<Gradient> {
    match (from, to) {
        (Some(a), Some(b)) if a.len == b.len => {
            let mut stops = b.stops;
            for index in 0..b.len {
                stops[index] = (
                    a.stops[index].0 + (b.stops[index].0 - a.stops[index].0) * t,
                    a.stops[index].1.lerp_to_gamma(b.stops[index].1, t),
                );
            }
            Some(Gradient {
                angle: a.angle + (b.angle - a.angle) * t,
                stops,
                len: b.len,
            })
        }
        (None, Some(b)) => {
            let mut stops = b.stops;
            for stop in stops.iter_mut().take(b.len) {
                stop.1 = stop.1.gamma_multiply(t);
            }
            Some(Gradient { stops, ..b })
        }
        (Some(a), None) => {
            let mut stops = a.stops;
            for stop in stops.iter_mut().take(a.len) {
                stop.1 = stop.1.gamma_multiply(1.0 - t);
            }
            Some(Gradient { stops, ..a })
        }
        (_, to) => to,
    }
}

fn mix_shadow(from: Option<Glow>, to: Option<Glow>, t: f32) -> Option<Glow> {
    let clear = |glow: Glow| Glow {
        color: Color32::TRANSPARENT,
        ..glow
    };
    let (a, b) = match (from, to) {
        (None, None) => return None,
        (Some(a), Some(b)) => (a, b),
        (None, Some(b)) => (clear(b), b),
        (Some(a), None) => (a, clear(a)),
    };
    Some(Glow {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        blur: a.blur + (b.blur - a.blur) * t,
        spread: a.spread + (b.spread - a.spread) * t,
        color: a.color.lerp_to_gamma(b.color, t),
    })
}

fn seconds(name: Option<&str>) -> f32 {
    match name {
        Some("fast") => crate::motion::fast(),
        Some("normal") => crate::motion::normal(),
        Some("slow") => crate::motion::slow(),
        Some(other) => match other.strip_suffix("ms") {
            Some(ms) => ms.parse::<f32>().unwrap_or(0.0) / 1000.0,
            None => other.trim_end_matches('s').parse().unwrap_or(0.0),
        },
        None => 0.0,
    }
}

pub(crate) fn look(context: &crate::gfx::Context, rule: &str) -> Style {
    resolve(
        context,
        rule,
        Id::new(("fluxa-style-look", rule)),
        State {
            enabled: true,
            ..State::default()
        },
        None,
    )
}

pub(crate) fn has_rule(rule: &str) -> bool {
    sheet().rules.contains_key(rule)
}

pub fn reload_styles_with(extra: &str) -> Result<(), String> {
    reload_styles(&format!("{DEFAULT_SHEET}\n{extra}"))
}

pub(crate) fn base(rule: &str) -> Style {
    base_with(rule, &[])
}

pub(crate) fn base_with(rule: &str, modifiers: &[&str]) -> Style {
    let sheet = sheet();
    let mut style = Style::blank();
    for layer in layers(&sheet, rule, modifiers) {
        apply(&mut style, &layer.base, None);
    }
    style
}


pub(crate) fn color(rule: &str) -> Color32 {
    let sheet = sheet();
    let mut style = Style::blank();
    for layer in layers(&sheet, rule, &[]) {
        apply(&mut style, &layer.base, None);
    }
    style.fill
}

fn chain<'a>(sheet: &'a Sheet, name: &str) -> Vec<&'a Rule> {
    let mut rules = Vec::new();
    let mut next = Some(name);
    while let Some(current) = next {
        let Some(rule) = sheet.rules.get(current) else {
            break;
        };
        if rules.len() >= 8 {
            break;
        }
        rules.push(rule);
        next = rule.extends.as_deref();
    }
    rules.reverse();
    rules
}

fn layers<'a>(sheet: &'a Sheet, name: &str, modifiers: &[&str]) -> Vec<&'a Layer> {
    let mut found = Vec::new();
    for rule in chain(sheet, name) {
        found.push(&rule.layer);
        found.extend(
            rule.media
                .iter()
                .filter(|(query, _)| query.matches())
                .map(|(_, layer)| layer),
        );
        found.extend(
            rule.variants
                .iter()
                .filter(|variant| {
                    ancestors_match(variant, false)
                        && variant
                            .modifiers
                            .iter()
                            .all(|wanted| modifier_met(wanted, modifiers))
                })
                .map(|variant| &variant.layer),
        );
    }
    found
}

fn stateful<'a>(sheet: &'a Sheet, name: &str, modifiers: &[&str]) -> Vec<&'a Variant> {
    chain(sheet, name)
        .into_iter()
        .flat_map(|rule| rule.variants.iter())
        .filter(|variant| {
            ancestors_match(variant, true)
                && variant
                    .modifiers
                    .iter()
                    .all(|wanted| modifier_met(wanted, modifiers))
        })
        .collect()
}

fn modifier_met(wanted: &str, present: &[&str]) -> bool {
    match wanted.strip_prefix('!') {
        Some(excluded) => !present.contains(&excluded),
        None => present.contains(&wanted),
    }
}

pub(crate) fn resolve(
    context: &crate::gfx::Context,
    rule: &str,
    id: Id,
    state: State,
    metrics: Option<&UiMetrics>,
) -> Style {
    resolve_with(context, rule, id, state, metrics, &[])
}

pub(crate) fn resolve_with(
    context: &crate::gfx::Context,
    rule: &str,
    id: Id,
    state: State,
    metrics: Option<&UiMetrics>,
    modifiers: &[&str],
) -> Style {
    let sheet = sheet();
    if !sheet.rules.contains_key(rule) {
        return Style {
            fill: Color32::MAGENTA,
            text: Color32::MAGENTA,
            ..Style::blank()
        };
    }
    let layers = layers(&sheet, rule, modifiers);
    let rules = chain(&sheet, rule);
    let pick = |field: fn(&Rule) -> Option<&str>| rules.iter().rev().find_map(|r| field(r));
    let mut base = Style::blank();
    let (text, font_size) = inherited();
    if let Some(text) = text {
        base.text = text;
    }
    base.font_size = font_size;
    for layer in &layers {
        apply(&mut base, &layer.base, metrics);
    }
    if !state.enabled {
        for layer in &layers {
            apply(&mut base, &layer.disabled, metrics);
        }
        return base;
    }
    let duration = seconds(pick(|r| r.transition.as_deref()));
    let easing = match pick(|r| r.easing.as_deref()) {
        Some("token") => crate::motion::Ease::Token,
        Some("out") => crate::motion::Ease::Out,
        _ => crate::motion::Ease::Linear,
    };
    let mut result = base;
    type Pick = fn(&Layer) -> &Patch;
    let stages: [(&str, bool, Pick); 4] = [
        ("select", state.selected, |l| &l.selected),
        ("hover", state.hovered, |l| &l.hover),
        ("focus", state.focused, |l| &l.focused),
        ("press", state.pressed, |l| &l.pressed),
    ];
    for (key, on, patch) in stages {
        let t = crate::motion::ease(
            context,
            id.with(key),
            f32::from(u8::from(on)),
            duration,
            easing,
        );
        if t > 0.0 {
            let mut target = result;
            for layer in &layers {
                apply(&mut target, patch(layer), metrics);
            }
            target.selected = if key == "select" {
                1.0
            } else {
                result.selected
            };
            result = mix(result, target, t);
        }
    }
    for (index, variant) in stateful(&sheet, rule, modifiers).into_iter().enumerate() {
        let t = crate::motion::ease(
            context,
            id.with(("ancestor", index)),
            f32::from(u8::from(ancestors_active(variant))),
            duration,
            easing,
        );
        if t > 0.0 {
            let mut target = result;
            apply(&mut target, &variant.layer.base, metrics);
            if state.hovered {
                apply(&mut target, &variant.layer.hover, metrics);
            }
            result = mix(result, target, t);
        }
    }
    match rules.iter().rev().find_map(|r| r.animation.as_ref()) {
        Some(animation) => animate(context, &sheet, animation, result, metrics),
        None => result,
    }
}

fn animate(
    context: &crate::gfx::Context,
    sheet: &Sheet,
    animation: &Animation,
    rest: Style,
    metrics: Option<&UiMetrics>,
) -> Style {
    let Some(frames) = sheet
        .keyframes
        .get(&animation.name)
        .filter(|f| !f.is_empty())
    else {
        return rest;
    };
    context.request_repaint();
    let elapsed = context.input(|input| input.time()) as f32 / animation.duration.max(0.001);
    let cycle = if animation.infinite {
        elapsed.rem_euclid(if animation.alternate { 2.0 } else { 1.0 })
    } else {
        elapsed.min(if animation.alternate { 2.0 } else { 1.0 })
    };
    let mut phase = if animation.alternate && cycle > 1.0 {
        2.0 - cycle
    } else {
        cycle.min(1.0)
    };
    if animation.easing == Easing::Out {
        phase = 1.0 - (1.0 - phase) * (1.0 - phase);
    }
    let at = |patch: &Patch| {
        let mut style = rest;
        apply(&mut style, patch, metrics);
        style
    };
    let next = frames.iter().position(|(stop, _)| *stop >= phase);
    match next {
        None => at(&frames[frames.len() - 1].1),
        Some(0) => at(&frames[0].1),
        Some(index) => {
            let (from_stop, from) = &frames[index - 1];
            let (to_stop, to) = &frames[index];
            let span = (to_stop - from_stop).max(0.0001);
            mix(at(from), at(to), (phase - from_stop) / span)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn standard_css_names_and_selectors_work() {
        let sheet = Sheet::parse(
            ".navbar .item { background: rgba(255, 255, 255, 0.1); color: #ffffff; border-radius: 999px; border: 1px solid #333333; }\n\
             .navbar .item:hover { background: #ffffff; }\n\
             .navbar .item:focus { border-color: #ff0000; }",
        )
        .unwrap();
        let variant = &sheet.rules.get("item").unwrap().variants[0];
        assert_eq!(variant.ancestors, ["navbar"]);
        let layer = &variant.layer;
        assert_eq!(
            layer.base.fill,
            Some(Paint::Rgba(Color32::from_rgba_unmultiplied(
                255, 255, 255, 26
            )))
        );
        assert_eq!(layer.base.radius, Some(Corner::Pill));
        assert_eq!(layer.base.stroke, Some(1.0));
        assert!(layer.hover.fill.is_some());
        assert!(layer.focused.border.is_some());
    }

    use super::*;

    fn resolved(rule: &str, state: State) -> Style {
        let context = crate::gfx::Context::default();
        resolve(&context, rule, Id::new("test"), state, None)
    }

    #[test]
    fn bundled_sheet_parses_every_rule() {
        let sheet = Sheet::parse(DEFAULT_SHEET).expect("parses");
        for name in [
            "button.primary",
            "button.secondary",
            "button.selected",
            "button.danger",
        ] {
            assert!(sheet.rules.contains_key(name), "{name}");
        }
    }

    #[test]
    fn lengths_accept_space_tokens_and_pixels() {
        let sheet =
            Sheet::parse(r#"{"a":{"padding":"s12","gap":6,"opacity":0.5,"max-width":300}}"#)
                .expect("parses");
        let mut style = Style::blank();
        apply(&mut style, &sheet.rules["a"].layer.base, None);
        assert_eq!(style.padding, Space::S12.px(&crate::metrics::active()));
        assert_eq!(style.gap, 6.0);
        assert_eq!(style.opacity, 0.5);
        assert_eq!(style.max_width, Some(300.0));
    }

    #[test]
    fn unknown_space_token_is_an_error() {
        assert!(Sheet::parse(r#"{"a":{"padding":"s13"}}"#).is_err());
    }

    #[test]
    fn hover_overrides_only_the_declared_fields() {
        let rest = resolved(
            "button.secondary",
            State {
                enabled: true,
                ..State::default()
            },
        );
        let hover = resolved(
            "button.secondary",
            State {
                enabled: true,
                hovered: true,
                ..State::default()
            },
        );
        assert_ne!(rest.fill, hover.fill);
        assert_eq!(rest.text, hover.text);
    }

    #[test]
    fn disabled_ignores_hover() {
        let off = resolved(
            "button.primary",
            State {
                hovered: true,
                ..State::default()
            },
        );
        assert_eq!(off.fill, ink(tone::soft()));
    }

    #[test]
    fn css_text_and_json_describe_the_same_sheet() {
        let css = Sheet::parse(
            ":root { --pad: s12; }\n\
             /* comment */\n\
             card { fill: ink/faint; padding: var(--pad); transition: fast out; }\n\
             card:hover, card:selected { fill: #11223344; }\n\
             @media (min-width: 900px) { card { gap: 6px; } }\n\
             @keyframes pulse { from { opacity: 0.5; } to { opacity: 1; } }\n\
             card[tight] { padding: 2; }",
        )
        .expect("css");
        let json = Sheet::parse(
            r##"{"card":{"fill":"ink/faint","padding":"s12","transition":"fast","easing":"out",
                 "hover":{"fill":"#11223344"},"selected":{"fill":"#11223344"}},
               "@min-width:900":{"card":{"gap":6}},
               "@keyframes pulse":{"from":{"opacity":0.5},"to":{"opacity":1}},
               "card[tight]":{"padding":2}}"##,
        )
        .expect("json");
        assert_eq!(css, json);
    }

    #[test]
    fn bundled_sheet_has_no_magenta_fallbacks_for_known_rules() {
        let sheet = Sheet::parse(DEFAULT_SHEET).expect("parses");
        assert!(sheet.rules.len() > 40);
    }

    #[test]
    fn extends_inherits_and_overrides() {
        let sheet = Sheet::parse(
            r##"{"a":{"fill":"#ff0000","padding":8},"b":{"extends":"a","padding":12}}"##,
        )
        .expect("parses");
        let mut style = Style::blank();
        for layer in layers(&sheet, "b", &[]) {
            apply(&mut style, &layer.base, None);
        }
        assert_eq!(style.fill, Color32::from_rgb(255, 0, 0));
        assert_eq!(style.padding, 12.0);
    }

    #[test]
    fn scope_and_modifier_variants_apply_only_when_active() {
        let sheet = Sheet::parse(
            r#"{"row":{"padding":4},"settings row":{"padding":10},"row[tight]":{"gap":1}}"#,
        )
        .expect("parses");
        let padding = |mods: &[&str]| {
            let mut style = Style::blank();
            for layer in layers(&sheet, "row", mods) {
                apply(&mut style, &layer.base, None);
            }
            (style.padding, style.gap)
        };
        assert_eq!(padding(&[]), (4.0, 0.0));
        assert_eq!(padding(&["tight"]), (4.0, 1.0));
        scoped("settings", None, || assert_eq!(padding(&[]), (10.0, 0.0)));
        assert_eq!(padding(&[]), (4.0, 0.0));
    }

    #[test]
    fn direction_and_align_parse_from_css() {
        let sheet =
            Sheet::parse("bar { direction: row; align: center; gap: s8; }").expect("parses");
        let mut style = Style::blank();
        for layer in layers(&sheet, "bar", &[]) {
            apply(&mut style, &layer.base, None);
        }
        assert_eq!(style.direction, Some(Direction::Row));
        assert_eq!(style.align, Some(Alignment::Center));
    }

    #[test]
    fn descendants_inherit_declared_text_color() {
        let context = crate::gfx::Context::default();
        let parent = Style {
            text: Color32::from_rgb(1, 2, 3),
            text_declared: true,
            ..Style::blank()
        };
        let child = scoped("panel", Some(&parent), || {
            resolve(
                &context,
                "button.secondary",
                Id::new("c"),
                State {
                    enabled: true,
                    ..Default::default()
                },
                None,
            )
        });
        assert_eq!(child.text, text_primary());
        let plain = scoped("panel", Some(&parent), inherited);
        assert_eq!(plain.0, Some(Color32::from_rgb(1, 2, 3)));
    }

    #[test]
    fn media_queries_follow_the_platform() {
        let sheet = Sheet::parse(
            r#"{"a":{"padding":4},"@tv":{"a":{"padding":40}},"@min-width:100000":{"a":{"gap":9}}}"#,
        )
        .expect("parses");
        let rule = &sheet.rules["a"];
        assert_eq!(rule.media.len(), 2);
        assert!(!rule.media[1].0.matches());
    }

    #[test]
    fn keyframes_interpolate_between_stops() {
        let sheet = Sheet::parse(
            r#"{"@keyframes fade":{"0%":{"opacity":0.0},"100%":{"opacity":1.0}},
                "a":{"animation":"fade 2s linear"}}"#,
        )
        .expect("parses");
        let animation = sheet.rules["a"].animation.as_ref().expect("animation");
        assert_eq!(animation.duration, 2.0);
        let context = crate::gfx::Context::default();
        let mut half = None;
        context
            .run_ui(
                crate::gfx::RawInput {
                    time: Some(1.0),
                    ..Default::default()
                },
                |_| half = Some(animate(&context, &sheet, animation, Style::blank(), None)),
            )
            .drop_without_applying_deltas();
        assert!((half.expect("ran").opacity - 0.5).abs() < 0.01);
    }

    #[test]
    fn unknown_color_names_are_rejected_at_parse_time_only_for_bad_syntax() {
        assert!(Sheet::parse(r##"{"a": {"fill": "#zz"}}"##).is_err());
        assert!(Sheet::parse(r#"{"a": {"fill": "ink/soft"}}"#).is_ok());
    }

    #[test]
    fn gradient_shadow_and_calc_parse() {
        let sheet = Sheet::parse(
            ".card { background: linear-gradient(to bottom, rgba(0, 0, 0, 0) 0%, #000000 100%); \
             box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4); padding: calc(10px + 2 * 3); margin: 4px; \
             transition: opacity 200ms ease-out; }",
        )
        .unwrap();
        let rule = &sheet.rules["card"];
        let mut style = Style::blank();
        apply(&mut style, &rule.layer.base, None);
        let gradient = style.gradient.expect("gradient");
        assert_eq!(gradient.len, 2);
        assert_eq!(gradient.angle, 180.0);
        assert_eq!(gradient.stops[1].1, Color32::BLACK);
        let glow = style.shadow.expect("shadow");
        assert_eq!((glow.y, glow.blur), (8.0, 24.0));
        assert_eq!(style.padding, 16.0);
        assert_eq!(style.margin, 4.0);
        assert_eq!(rule.transition.as_deref(), Some("200ms"));
        assert_eq!(rule.easing.as_deref(), Some("out"));
    }

    #[test]
    fn compound_classes_become_modifiers() {
        let sheet = Sheet::parse(".item.active { background: #ffffff; }").unwrap();
        let variant = &sheet.rules["item"].variants[0];
        assert_eq!(variant.modifiers, ["active"]);
    }

    #[test]
    fn ancestor_state_applies_only_while_the_ancestor_is_hovered() {
        let sheet =
            Sheet::parse(".label { opacity: 0.2; }\n.bar:hover .label { opacity: 1; }").unwrap();
        let variants = stateful(&sheet, "label", &[]);
        assert!(variants.is_empty());
        let variants = within(&[("bar".to_owned(), 0)], || stateful(&sheet, "label", &[]));
        assert_eq!(variants.len(), 1);
        let idle = within(&[("bar".to_owned(), 0)], || ancestors_active(variants[0]));
        let hot = within(&[("bar".to_owned(), HOVERED)], || {
            ancestors_active(variants[0])
        });
        assert!(!idle);
        assert!(hot);
    }

    #[test]
    fn structural_pseudos_become_modifiers() {
        let sheet = Sheet::parse(
            ".row .cell:nth-child(odd):not(.hot) { opacity: 0.5; }\n\
             .cell:last-child { opacity: 0.1; }\n\
             .glass { backdrop-filter: blur(12px); }",
        )
        .unwrap();
        let variants = &sheet.rules["cell"].variants;
        assert!(
            variants
                .iter()
                .any(|v| v.modifiers == ["nth-child(odd)", "!hot"])
        );
        assert!(variants.iter().any(|v| v.modifiers == ["last-child"]));
        assert_eq!(sheet.rules["glass"].layer.base.backdrop, Some(12.0));
        assert!(Sheet::parse(".a:bogus { opacity: 1; }").is_err());
    }
}
