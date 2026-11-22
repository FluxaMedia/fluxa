use super::*;
use serde_json::Value;
use std::sync::{Arc, Mutex, RwLock};

mod css;
mod defaults;
mod parse;
mod scoped;
pub(crate) mod vars;

const DEFAULT_SHEET: &str = include_str!("../../../shared/skins/default/style.css");

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
    pub width_vw: Option<f32>,
    pub column_width: Option<f32>,
    pub columns: Option<u32>,
    pub height_vh: Option<f32>,
    pub aspect: Option<f32>,
    pub weight: Option<f32>,
    pub stretch: f32,
    pub squash: f32,
    pub magnify: f32,
    pub reach: f32,
    pub justify: Option<Justify>,
    pub grow: Option<f32>,
    pub hidden: bool,
    pub fill_space: bool,
    pub order: i32,
    pub padding_box: Option<[f32; 4]>,
    pub gap_set: Option<f32>,
    pub vars: u32,
}

pub(crate) const MAX_GRADIENT_STOPS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Gradient {
    pub angle: f32,
    pub stops: [(f32, Color32); MAX_GRADIENT_STOPS],
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
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Style {
    pub(crate) fn blank() -> Self {
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
            width_vw: None,
            column_width: None,
            columns: None,
            height_vh: None,
            aspect: None,
            weight: None,
            stretch: 0.0,
            squash: 0.0,
            magnify: 0.0,
            reach: 1.0,
            justify: None,
            grow: None,
            hidden: false,
            fill_space: false,
            order: 0,
            padding_box: None,
            gap_set: None,
            vars: 0,
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
            Length::Space(space) => space.px(),
            Length::Px(px) => px,
            Length::Metric(metric) => match metric {
                Metric::PosterWidth => crate::metrics::theme().poster_width,
                Metric::PosterHeight => crate::metrics::theme().poster_height,
                Metric::ContinueWidth => crate::metrics::theme().continue_width,
                Metric::IconButton => crate::vars::px("icon-button-size"),
                Metric::ControlHeight => crate::vars::px("screen-control-height"),
                Metric::Gutter => crate::vars::px("gutter"),
                Metric::SynopsisSize => hero_synopsis_size(),
                Metric::LabelSize => crate::vars::px("nav-label-size") + crate::dp::Dp::D2.px(),
                Metric::ContinueHeight => crate::metrics::theme().continue_height,
                Metric::WideWidth => crate::vars::px("collection-wide-width"),
                Metric::WideHeight => crate::vars::px("collection-wide-height"),
                Metric::SquareSize => crate::vars::px("collection-square-size"),
                Metric::CollectionWidth => crate::vars::px("collection-poster-width"),
                Metric::CollectionHeight => crate::vars::px("collection-poster-height"),
                Metric::CardTitle => crate::vars::px("screen-card-title-size"),
                Metric::CardSubtitle => crate::vars::px("screen-card-subtitle-size"),
                Metric::CardPadding => crate::vars::px("card-content-padding"),
            },
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
            Corner::Pill => height * crate::vars::px("pill-ratio"),
            Corner::Card => crate::vars::px("card-radius"),
            Corner::Dialog => crate::vars::px("dialog-radius"),
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
    Var(String),
    Mix(Box<Paint>, f32, Box<Paint>),
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
    width_vw: Option<f32>,
    column_width: Option<Length>,
    columns: Option<u32>,
    height_vh: Option<f32>,
    aspect: Option<f32>,
    weight: Option<f32>,
    stretch: Option<f32>,
    squash: Option<f32>,
    magnify: Option<f32>,
    reach: Option<f32>,
    justify: Option<Justify>,
    grow: Option<f32>,
    hidden: Option<bool>,
    fill_space: Option<bool>,
    order: Option<i32>,
    padding_box: Option<[Length; 4]>,
    vars: u32,
    auto_states: Option<bool>,
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
    Input(String),
    Theme(String),
    Window(String),
    MinWidth(f32),
    MaxWidth(f32),
    MinHeight(f32),
    MaxHeight(f32),
}

impl Query {
    fn matches(&self) -> bool {
        let metrics = crate::metrics::view();
        match self {
            Query::Platform(name) => metrics.platform == name,
            Query::Input(name) => crate::metrics::input() == name,
            Query::Theme(id) => fluxa_theme::theme::selected_theme_id() == *id,
            Query::Window(name) => metrics.window_class == name,
            Query::MinWidth(width) => metrics.width >= *width,
            Query::MaxWidth(width) => metrics.width <= *width,
            Query::MinHeight(height) => metrics.height >= *height,
            Query::MaxHeight(height) => metrics.height <= *height,
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
    rules: std::collections::HashMap<String, Rule, crate::fast::Build>,
    keyframes: std::collections::HashMap<String, Vec<(f32, Patch)>, crate::fast::Build>,
}

static SHEET: RwLock<Option<Arc<Sheet>>> = RwLock::new(None);
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn generation() -> u64 {
    GENERATION.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn sheet() -> Arc<Sheet> {
    if let Some(sheet) = SHEET.read().ok().and_then(|slot| slot.clone()) {
        return sheet;
    }
    let parsed = Arc::new(Sheet::parse(DEFAULT_SHEET).expect("bundled fluxa.css"));
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(parsed.clone());
    }
    parsed
}

pub fn reload_styles(text: &str) -> Result<(), String> {
    vars::load(text)?;
    let (parsed, motion, curve) = Sheet::parse_with_motion(text)?;
    crate::motion::set_styled(motion);
    if let Some(points) = curve {
        crate::motion::set_curve(points);
    }
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(Arc::new(parsed));
        GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}

impl Sheet {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        Self::parse_inner(text, false).map(|(sheet, ..)| sheet)
    }

    pub(crate) fn parse_with_motion(
        text: &str,
    ) -> Result<(Self, [Option<f32>; 3], Option<[f32; 4]>), String> {
        Self::parse_inner(text, true)
    }

    fn parse_inner(
        text: &str,
        lenient: bool,
    ) -> Result<(Self, [Option<f32>; 3], Option<[f32; 4]>), String> {
        let mut root: Value = if text.trim_start().starts_with('{') {
            serde_json::from_str(text).map_err(|error| error.to_string())?
        } else {
            css::to_value(text)?
        };
        let motion = root.as_object_mut().and_then(|map| map.remove("@motion"));
        let sheet = parse::sheet_from_value(&root, lenient)?;
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

#[derive(Clone, Debug)]
pub(crate) struct Frame {
    pub name: std::sync::Arc<str>,
    pub states: u8,
    pub text: Option<Color32>,
    pub font_size: Option<f32>,
    pub weight: Option<f32>,
    pub vars: u32,
}

impl Frame {
    pub(crate) fn plain(name: std::sync::Arc<str>, states: u8) -> Self {
        Self {
            name,
            states,
            text: None,
            font_size: None,
            weight: None,
            vars: 0,
        }
    }

    pub(crate) fn inheriting(mut self, style: &Style) -> Self {
        self.text = style.text_declared.then_some(style.text);
        self.font_size = style.font_size;
        self.weight = style.weight;
        self.vars = style.vars;
        self
    }
}

struct Scope {
    frame: Frame,
    signature: u64,
}

impl Scope {
    fn new(parent: u64, frame: Frame) -> Self {
        let signature = crate::fast::hash((
            parent,
            &frame.name,
            frame.text.map(|color| color.to_array()),
            frame.font_size.map(f32::to_bits),
            frame.weight.map(f32::to_bits),
            frame.vars,
            frame.states,
        ));
        Self { frame, signature }
    }
}

fn scope_signature() -> u64 {
    SCOPES.with(|stack| stack.borrow().last().map_or(0, |scope| scope.signature))
}

pub(crate) const HOVERED: u8 = 1;
pub(crate) const FOCUSED: u8 = 2;
pub(crate) const PRESSED: u8 = 4;
pub(crate) const SELECTED: u8 = 8;

thread_local! {
    static SCOPES: std::cell::RefCell<Vec<Scope>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn inherit(style: &mut Style) {
    SCOPES.with(|stack| {
        let stack = stack.borrow();
        if !style.text_declared
            && let Some(text) = stack.iter().rev().find_map(|scope| scope.frame.text)
        {
            style.text = text;
            style.text_declared = true;
        }
        if style.font_size.is_none() {
            style.font_size = stack.iter().rev().find_map(|scope| scope.frame.font_size);
        }
        if style.weight.is_none() {
            style.weight = stack.iter().rev().find_map(|scope| scope.frame.weight);
        }
    });
}

fn in_scope(name: &str) -> bool {
    SCOPES.with(|stack| stack.borrow().iter().any(|scope| *scope.frame.name == *name))
}

fn in_scope_with(name: &str, flag: u8) -> bool {
    SCOPES.with(|stack| {
        stack
            .borrow()
            .iter()
            .any(|scope| *scope.frame.name == *name && scope.frame.states & flag != 0)
    })
}

pub(crate) fn within<R>(frames: &[Frame], body: impl FnOnce() -> R) -> R {
    SCOPES.with(|stack| {
        let mut stack = stack.borrow_mut();
        for frame in frames {
            let parent = stack.last().map_or(0, |scope| scope.signature);
            stack.push(Scope::new(parent, frame.clone()));
        }
    });
    let result = body();
    SCOPES.with(|stack| {
        let mut stack = stack.borrow_mut();
        let keep = stack.len().saturating_sub(frames.len());
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
    if let Some(name) = text
        .strip_prefix("var(--")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return Some(Paint::Var(name.trim().to_owned()));
    }
    if let Some(inner) = text
        .strip_prefix("color-mix(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_mix(inner);
    }
    let (name, factor) = match text.split_once('*') {
        Some((name, factor)) => (name, factor.parse().ok()?),
        None => (text, 1.0),
    };
    Some(Paint::Named(name.to_owned(), factor))
}

fn split_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

fn mix_stop(text: &str) -> Option<(Paint, Option<f32>)> {
    match text.rsplit_once(' ') {
        Some((color, percent)) if percent.ends_with('%') => Some((
            parse_paint(color.trim())?,
            Some(percent.trim_end_matches('%').parse::<f32>().ok()? / 100.0),
        )),
        _ => Some((parse_paint(text)?, None)),
    }
}

fn parse_mix(inner: &str) -> Option<Paint> {
    let parts = split_commas(inner);
    let [method, first, second] = parts.as_slice() else {
        return None;
    };
    if !method.starts_with("in ") {
        return None;
    }
    let (first, first_share) = mix_stop(first)?;
    let (second, second_share) = mix_stop(second)?;
    let share = first_share.or(second_share.map(|share| 1.0 - share))?;
    Some(Paint::Mix(Box::new(first), share, Box::new(second)))
}

fn mix_colors(a: Color32, share: f32, b: Color32) -> Color32 {
    let a = a.to_array().map(|c| c as f32);
    let b = b.to_array().map(|c| c as f32);
    let blended = [0, 1, 2, 3].map(|i| a[i] * share + b[i] * (1.0 - share));
    Color32::from_rgba_premultiplied(
        blended[0].round() as u8,
        blended[1].round() as u8,
        blended[2].round() as u8,
        blended[3].round() as u8,
    )
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

fn resolve_paint(paint: &Paint) -> Color32 {
    resolve_paint_at(paint, 0)
}

fn resolve_paint_at(paint: &Paint, depth: u8) -> Color32 {
    match paint {
        Paint::Var(name) => scoped::lookup(name)
            .or_else(|| crate::vars::raw(name))
            .filter(|_| depth < 4)
            .and_then(|value| parse_paint(value.trim()))
            .map_or(Color32::MAGENTA, |inner| {
                resolve_paint_at(&inner, depth + 1)
            }),
        Paint::Mix(first, share, second) => mix_colors(
            resolve_paint_at(first, depth),
            *share,
            resolve_paint_at(second, depth),
        ),
        Paint::Rgba(color) => *color,
        Paint::Ink(level) => ink(*level),
        Paint::Named(name, factor) => {
            let color = match name.as_str() {
                "background" => background(),
                "theme.settings.description" => ink(crate::vars::byte("settings-swatch-alpha")),
                "theme.settings.description.compact" => {
                    ink(crate::vars::byte("settings-swatch-alpha-compact"))
                }
                "theme.surface" => crate::metrics::theme().surface,
                "theme.border" => crate::metrics::theme().border,
                "theme.input.background" => crate::metrics::theme().input_background,
                "theme.input.border" => crate::metrics::theme().input_border,
                "theme.input.border.focus" => crate::metrics::theme()
                    .input_border_focus
                    .unwrap_or_else(accent),
                "theme.toggle.knob.on" => {
                    if crate::metrics::theme().accent_foreground
                        == crate::metrics::theme().text_primary
                    {
                        crate::metrics::theme().text_primary
                    } else {
                        crate::metrics::theme().background
                    }
                }
                "theme.button.primary" => crate::metrics::theme().button_primary,
                "theme.button.primary.text" => crate::metrics::theme().button_primary_text,
                "theme.button.border" => {
                    Color32::from_white_alpha(crate::vars::byte("button-border"))
                }
                "text.primary" => text_primary(),
                "accent" => accent(),
                "danger" => danger_color(),
                "accent.foreground" => crate::metrics::theme().accent_foreground,
                "surface.raised" => crate::metrics::theme().surface_raised,
                "text.secondary" => crate::metrics::theme().text_secondary,
                "text.muted" => crate::metrics::theme().text_muted,
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

fn apply(style: &mut Style, patch: &Patch) {
    if let Some(paint) = &patch.fill {
        style.fill = resolve_paint(paint);
        style.gradient = None;
    }
    if let Some(spec) = &patch.gradient {
        let mut stops = [(0.0, Color32::TRANSPARENT); MAX_GRADIENT_STOPS];
        let count = spec.stops.len().min(MAX_GRADIENT_STOPS);
        for (index, (at, paint)) in spec.stops.iter().take(MAX_GRADIENT_STOPS).enumerate() {
            let fallback = if count > 1 {
                index as f32 / (count - 1) as f32
            } else {
                0.0
            };
            stops[index] = (at.unwrap_or(fallback), resolve_paint(paint));
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
            color: resolve_paint(&spec.color),
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
    if let Some(width) = patch.column_width {
        style.column_width = Some(width.px());
    }
    if let Some(columns) = patch.columns {
        style.columns = Some(columns);
    }
    if let Some(share) = patch.width_vw {
        style.width_vw = Some(share);
    }
    if let Some(share) = patch.height_vh {
        style.height_vh = Some(share);
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
        style.text = resolve_paint(paint);
        style.text_declared = true;
    }
    if let Some(paint) = &patch.border {
        style.border = resolve_paint(paint);
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
        style.padding_box = Some([style.padding; 4]);
    }
    if let Some(sides) = patch.padding_box {
        style.padding_box = Some(sides.map(Length::px));
    }
    if let Some(gap) = patch.gap {
        style.gap = gap.px();
        style.gap_set = Some(style.gap);
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
    if let Some(justify) = patch.justify {
        style.justify = Some(justify);
    }
    if let Some(grow) = patch.grow {
        style.grow = Some(grow);
    }
    if let Some(hidden) = patch.hidden {
        style.hidden = hidden;
    }
    if let Some(fill_space) = patch.fill_space {
        style.fill_space = fill_space;
    }
    if let Some(order) = patch.order {
        style.order = order;
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
        width_vw: to.width_vw,
        column_width: to.column_width,
        columns: to.columns,
        height_vh: to.height_vh,
        aspect: to.aspect,
        weight: to.weight,
        stretch: to.stretch,
        squash: to.squash,
        magnify: to.magnify,
        reach: to.reach,
        justify: to.justify,
        grow: to.grow,
        hidden: to.hidden,
        fill_space: to.fill_space,
        order: to.order,
        padding_box: to.padding_box,
        gap_set: to.gap_set,
        vars: to.vars,
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
        None => crate::motion::fast(),
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
    )
}

/// Rule names older skins used for the canonical button names. They keep
/// parsing and matching, so a third-party sheet that styles `.action-primary`
/// or `button.primary` still restyles `btn-primary` nodes and widgets.
const ALIASES: [(&str, &str); 6] = [
    ("button.primary", "btn-primary"),
    ("btn-secondary", "btn-secondary"),
    ("button.selected", "btn-selected"),
    ("button.danger", "btn-danger"),
    ("action-primary", "btn-primary"),
    ("action-secondary", "btn-secondary"),
];

fn canonical(name: &str) -> &str {
    ALIASES
        .iter()
        .find(|(legacy, _)| *legacy == name)
        .map_or(name, |(_, canonical)| canonical)
}

pub(crate) fn has_rule(rule: &str) -> bool {
    sheet().rules.contains_key(canonical(rule))
}

pub fn reload_styles_with(extra: &str) -> Result<(), String> {
    if extra.trim() == DEFAULT_SHEET.trim() {
        return reload_styles(DEFAULT_SHEET);
    }
    reload_styles(&format!("{DEFAULT_SHEET}\n{extra}"))
}

pub(crate) fn base(rule: &str) -> Style {
    base_with(rule, &[])
}

pub(crate) fn uses_order() -> bool {
    static CACHE: Mutex<(u64, bool)> = Mutex::new((0, false));
    let generation = generation();
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if cache.0 != generation {
        let sheet = sheet();
        let declared = |layer: &Layer| layer.base.order.is_some();
        let used = sheet.rules.values().any(|rule| {
            declared(&rule.layer) || rule.media.iter().any(|(_, layer)| declared(layer))
        });
        *cache = (generation, used);
    }
    cache.1
}

pub(crate) fn hero_synopsis_size() -> f32 {
    base_with("hero-synopsis-size", &[])
        .font_size
        .unwrap_or(16.0)
        .max(14.0)
}

pub(crate) fn base_with(rule: &str, modifiers: &[&str]) -> Style {
    struct Entry {
        rule: Box<str>,
        modifiers: Box<[Box<str>]>,
        scope: u64,
        style: Style,
    }
    #[derive(Default)]
    struct Memo {
        generation: u64,
        metrics: Option<crate::metrics::MetricsKey>,
        entries: std::collections::HashMap<u64, Vec<Entry>, crate::fast::Build>,
    }
    thread_local! {
        static MEMO: std::cell::RefCell<Memo> = std::cell::RefCell::new(Memo::default());
    }
    let generation = generation();
    let metrics = crate::metrics::active_key();
    let scope = scope_signature();
    let key = crate::fast::hash((rule, modifiers, scope));
    let cached = MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        if memo.generation != generation || memo.metrics != metrics {
            memo.entries.clear();
            memo.generation = generation;
            memo.metrics = metrics;
        }
        memo.entries.get(&key).and_then(|bucket| {
            bucket
                .iter()
                .find(|entry| {
                    entry.scope == scope
                        && *entry.rule == *rule
                        && entry.modifiers.len() == modifiers.len()
                        && entry
                            .modifiers
                            .iter()
                            .zip(modifiers)
                            .all(|(known, wanted)| **known == **wanted)
                })
                .map(|entry| entry.style)
        })
    });
    if let Some(style) = cached {
        return style;
    }
    let sheet = sheet();
    let style = base_in(&sheet, rule, modifiers);
    MEMO.with(|memo| {
        memo.borrow_mut()
            .entries
            .entry(key)
            .or_default()
            .push(Entry {
                rule: rule.into(),
                modifiers: modifiers
                    .iter()
                    .map(|modifier| (*modifier).into())
                    .collect(),
                scope,
                style,
            });
    });
    style
}

fn base_in(sheet: &Sheet, rule: &str, modifiers: &[&str]) -> Style {
    let layers = layers(sheet, rule, modifiers);
    let own = scoped::merge(layers.iter().map(|layer| layer.base.vars));
    let _own = scoped::Own::enter(own);
    let mut style = Style::blank();
    for layer in layers {
        apply(&mut style, &layer.base);
    }
    style.vars = own;
    inherit(&mut style);
    style
}

pub(crate) fn color(rule: &str) -> Color32 {
    base_in(&sheet(), rule, &[]).fill
}

fn chain<'a>(sheet: &'a Sheet, name: &str) -> Vec<&'a Rule> {
    let mut rules = Vec::new();
    let mut next = Some(canonical(name));
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

fn kinds<'a>(sheet: &'a Sheet, modifiers: &[&str]) -> Vec<&'a Rule> {
    modifiers
        .iter()
        .filter(|modifier| modifier.starts_with('<'))
        .flat_map(|modifier| chain(sheet, modifier))
        .collect()
}

fn layers<'a>(sheet: &'a Sheet, name: &str, modifiers: &[&str]) -> Vec<&'a Layer> {
    let mut found = Vec::new();
    let extra = modifiers
        .iter()
        .filter(|modifier| !modifier.starts_with('<') && sheet.rules.contains_key(canonical(modifier)));
    let rules = kinds(sheet, modifiers)
        .into_iter()
        .chain(chain(sheet, name))
        .chain(extra.flat_map(|modifier| chain(sheet, modifier)));
    for rule in rules {
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
    kinds(sheet, modifiers)
        .into_iter()
        .chain(chain(sheet, name))
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

pub(crate) fn resolve(context: &crate::gfx::Context, rule: &str, id: Id, state: State) -> Style {
    resolve_with(context, rule, id, state, &[])
}

pub(crate) fn resolve_with(
    context: &crate::gfx::Context,
    rule: &str,
    id: Id,
    state: State,
    modifiers: &[&str],
) -> Style {
    resolve_in(context, &sheet(), rule, id, state, modifiers)
}

fn resolve_in(
    context: &crate::gfx::Context,
    sheet: &Sheet,
    rule: &str,
    id: Id,
    state: State,
    modifiers: &[&str],
) -> Style {
    if !sheet.rules.contains_key(canonical(rule)) {
        return Style {
            fill: Color32::MAGENTA,
            text: Color32::MAGENTA,
            ..Style::blank()
        };
    }
    let layers = layers(sheet, rule, modifiers);
    let rules: Vec<&Rule> = kinds(sheet, modifiers)
        .into_iter()
        .chain(chain(sheet, rule))
        .collect();
    let pick = |field: fn(&Rule) -> Option<&str>| rules.iter().rev().find_map(|r| field(r));
    let own = scoped::merge(layers.iter().map(|layer| layer.base.vars));
    let _own = scoped::Own::enter(own);
    let mut base = Style::blank();
    for layer in &layers {
        apply(&mut base, &layer.base);
    }
    base.vars = own;
    inherit(&mut base);
    let derive = layers
        .iter()
        .rev()
        .find_map(|layer| layer.base.auto_states)
        .unwrap_or(false)
        && base.gradient.is_none()
        && layers.iter().any(|layer| layer.base.fill.is_some());
    if !state.enabled {
        for layer in &layers {
            apply(&mut base, &layer.disabled);
        }
        if derive && layers.iter().all(|layer| layer.disabled == Patch::default()) {
            base.fill = ink(tone::soft());
            base.text = ink(tone::dim());
            base.text_declared = true;
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
        let undeclared = layers.iter().all(|layer| *patch(layer) == Patch::default());
        let derived = derive && undeclared && key == "hover" && result.fill.a() > 0;
        if key != "select" && undeclared && !derived {
            continue;
        }
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
                apply(&mut target, patch(layer));
            }
            if derived {
                target.fill = scoped::hover_fill(result.fill);
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
            apply(&mut target, &variant.layer.base);
            if state.hovered {
                apply(&mut target, &variant.layer.hover);
            }
            result = mix(result, target, t);
        }
    }
    match rules.iter().rev().find_map(|r| r.animation.as_ref()) {
        Some(animation) => animate(context, sheet, animation, result),
        None => result,
    }
}

fn animate(
    context: &crate::gfx::Context,
    sheet: &Sheet,
    animation: &Animation,
    rest: Style,
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
        apply(&mut style, patch);
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
    fn color_mix_blends_toward_transparent_and_follows_var() {
        let half = parse_paint("color-mix(in srgb, #FFFFFF 50%, transparent)").unwrap();
        assert_eq!(resolve_paint(&half).a(), 128);
        let linked = parse_paint("var(--color-text-primary)").unwrap();
        assert_eq!(resolve_paint(&linked), crate::metrics::text_primary());
    }

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
        resolve(&context, rule, Id::new("test"), state)
    }

    #[test]
    fn bundled_sheet_parses_every_rule() {
        let sheet = Sheet::parse(DEFAULT_SHEET).expect("parses");
        for name in [
            "btn-primary",
            "btn-secondary",
            "btn-selected",
            "btn-danger",
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
        apply(&mut style, &sheet.rules["a"].layer.base);
        assert_eq!(style.padding, Space::S12.px());
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
            "btn-secondary",
            State {
                enabled: true,
                ..State::default()
            },
        );
        let hover = resolved(
            "btn-secondary",
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
            apply(&mut style, &layer.base);
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
                apply(&mut style, &layer.base);
            }
            (style.padding, style.gap)
        };
        assert_eq!(padding(&[]), (4.0, 0.0));
        assert_eq!(padding(&["tight"]), (4.0, 1.0));
        within(&[Frame::plain("settings".into(), 0)], || assert_eq!(padding(&[]), (10.0, 0.0)));
        assert_eq!(padding(&[]), (4.0, 0.0));
    }

    #[test]
    fn direction_and_align_parse_from_css() {
        let sheet =
            Sheet::parse("bar { direction: row; align: center; gap: s8; }").expect("parses");
        let mut style = Style::blank();
        for layer in layers(&sheet, "bar", &[]) {
            apply(&mut style, &layer.base);
        }
        assert_eq!(style.direction, Some(Direction::Row));
        assert_eq!(style.align, Some(Alignment::Center));
    }

    #[test]
    fn flex_properties_parse_from_css() {
        let sheet = Sheet::parse(
            ".bar { display: none; justify-content: space-between; flex-grow: 2; align-items: stretch; order: 3; }",
        )
        .expect("parses");
        let mut style = Style::blank();
        for layer in layers(&sheet, "bar", &[]) {
            apply(&mut style, &layer.base);
        }
        assert!(style.hidden);
        assert_eq!(style.justify, Some(Justify::SpaceBetween));
        assert_eq!(style.grow, Some(2.0));
        assert_eq!(style.align, Some(Alignment::Stretch));
        assert_eq!(style.order, 3);
    }

    #[test]
    fn flex_fill_parses_from_css() {
        let sheet =
            Sheet::parse(".pane { flex-fill: 1; } .rest { flex-fill: 0; }").expect("parses");
        let fill = |name: &str| {
            let mut style = Style::blank();
            for layer in layers(&sheet, name, &[]) {
                apply(&mut style, &layer.base);
            }
            style.fill_space
        };
        assert!(fill("pane"));
        assert!(!fill("rest"));
    }

    #[test]
    fn descendants_inherit_text_size_and_weight() {
        let context = crate::gfx::Context::default();
        let parent = Style {
            text: Color32::from_rgb(1, 2, 3),
            text_declared: true,
            font_size: Some(21.0),
            weight: Some(700.0),
            ..Style::blank()
        };
        let frames = [Frame::plain("panel".into(), 0).inheriting(&parent)];
        let (own, plain) = within(&frames, || {
            let state = State {
                enabled: true,
                ..Default::default()
            };
            (
                resolve(&context, "btn-secondary", Id::new("c"), state),
                resolve(&context, "divider", Id::new("d"), state),
            )
        });
        assert_eq!(own.text, text_primary());
        assert_eq!(plain.font_size, Some(21.0));
        assert!(own.font_size.is_some());
        assert_eq!(plain.text, Color32::from_rgb(1, 2, 3));
        assert_eq!(plain.weight, Some(700.0));
    }

    fn on(rule: &str, sheet: &Sheet, state: State, frames: &[Frame]) -> Style {
        let context = crate::gfx::Context::default();
        within(frames, || {
            resolve_in(&context, sheet, rule, Id::new(("scoped", rule)), state, &[])
        })
    }

    const LIVE: State = State {
        hovered: false,
        pressed: false,
        focused: false,
        enabled: true,
        selected: false,
    };

    #[test]
    fn scoped_custom_properties_resolve_per_element() {
        let sheet = Sheet::parse(
            ".a { --tone: #ff0000; background: var(--tone); }\n\
             .b { --tone: #00ff00; background: var(--tone); }\n\
             .c { background: var(--tone); }\n\
             .d { --tone: var(--color-accent); background: var(--tone); }",
        )
        .expect("parses");
        let fill = |rule| on(rule, &sheet, LIVE, &[]).fill;
        assert_eq!(fill("a"), Color32::from_rgb(255, 0, 0));
        assert_eq!(fill("b"), Color32::from_rgb(0, 255, 0));
        assert_eq!(fill("d"), accent());
        // Without a scoped declaration the :root table answers (unknown names are magenta).
        assert_eq!(fill("c"), Color32::MAGENTA);
    }

    #[test]
    fn scoped_properties_reach_descendants_and_nearest_wins() {
        let sheet = Sheet::parse(
            ".panel { --tone: #ff0000; }\n\
             .inner { --tone: #0000ff; }\n\
             .leaf { background: var(--tone); }",
        )
        .expect("parses");
        let panel = on("panel", &sheet, LIVE, &[]);
        assert_ne!(panel.vars, 0);
        let outer = Frame::plain("panel".into(), 0).inheriting(&panel);
        let inner = Frame::plain("inner".into(), 0)
            .inheriting(&on("inner", &sheet, LIVE, std::slice::from_ref(&outer)));
        let leaf = |frames: &[Frame]| on("leaf", &sheet, LIVE, frames).fill;
        assert_eq!(leaf(&[outer.clone()]), Color32::from_rgb(255, 0, 0));
        assert_eq!(leaf(&[outer, inner]), Color32::from_rgb(0, 0, 255));
    }

    #[test]
    fn same_rule_scoped_properties_merge_across_selectors() {
        let sheet = Sheet::parse(
            ".a { --one: #010203; }\n.a { --two: #040506; background: var(--one); color: var(--two); }",
        )
        .expect("parses");
        let style = on("a", &sheet, LIVE, &[]);
        assert_eq!(style.fill, Color32::from_rgb(1, 2, 3));
        assert_eq!(style.text, Color32::from_rgb(4, 5, 6));
    }

    #[test]
    fn root_properties_keep_resolving_everywhere() {
        let sheet = Sheet::parse(".a { background: var(--color-accent); }").expect("parses");
        assert_eq!(on("a", &sheet, LIVE, &[]).fill, accent());
    }

    fn hovered() -> State {
        State {
            hovered: true,
            ..LIVE
        }
    }

    #[test]
    fn derived_hover_matches_the_declared_hover_of_the_default_skin() {
        let declared = Sheet::parse(
            ".o { background: var(--color-accent); }\n\
             .o:hover { background: color-mix(in srgb, var(--color-accent) 85%, transparent); }\n\
             .e { background: var(--color-error); }\n\
             .e:hover { background: color-mix(in srgb, var(--color-error) 85%, transparent); }\n\
             .t { background: color-mix(in srgb, var(--color-text-primary) 10.98%, transparent); }\n\
             .t:hover { background: color-mix(in srgb, var(--color-text-primary) 18.039%, transparent); }",
        )
        .expect("parses");
        let derived = Sheet::parse(
            ".o, .e, .t { states: auto; }\n\
             .o { background: var(--color-accent); }\n\
             .e { background: var(--color-error); }\n\
             .t { background: color-mix(in srgb, var(--color-text-primary) 10.98%, transparent); }",
        )
        .expect("parses");
        for rule in ["o", "e", "t"] {
            assert_eq!(
                on(rule, &derived, hovered(), &[]).fill,
                on(rule, &declared, hovered(), &[]).fill,
                "{rule}"
            );
            assert_ne!(
                on(rule, &derived, hovered(), &[]).fill,
                on(rule, &derived, LIVE, &[]).fill,
                "{rule}"
            );
        }
    }

    #[test]
    fn derived_disabled_is_the_shared_disabled_look() {
        let sheet =
            Sheet::parse(".a { states: auto; background: #123456; color: #ffffff; }")
                .expect("parses");
        let off = on("a", &sheet, State::default(), &[]);
        assert_eq!(off.fill, ink(tone::soft()));
        assert_eq!(off.text, ink(tone::dim()));
    }

    #[test]
    fn states_stay_off_unless_requested_and_declared_variants_win() {
        let plain = Sheet::parse(".a { background: #336699; }").expect("parses");
        assert_eq!(
            on("a", &plain, hovered(), &[]).fill,
            on("a", &plain, LIVE, &[]).fill
        );
        let explicit = Sheet::parse(
            ".a { states: auto; background: #336699; }\n.a:hover { background: #ffffff; }",
        )
        .expect("parses");
        assert_eq!(
            on("a", &explicit, hovered(), &[]).fill,
            Color32::from_rgb(255, 255, 255)
        );
    }

    #[test]
    fn gradients_keep_color_tokens_live() {
        let sheet = Sheet::parse(
            ".fade { background: linear-gradient(to bottom, \
             color-mix(in srgb, var(--color-background) 40%, transparent) 0%, \
             var(--color-surface) 100%); }",
        )
        .expect("parses");
        let gradient = sheet.rules["fade"].layer.base.gradient.as_ref().expect("gradient");
        assert_eq!(gradient.stops.len(), 2);
    }

    #[test]
    fn legacy_button_names_resolve_to_the_canonical_rules() {
        let sheet = Sheet::parse(
            "button.primary { background: #112233; } .action-secondary:hover { background: #445566; } \
             chip.selected { color: #ffffff; }",
        )
        .expect("parses");
        assert!(sheet.rules.contains_key("btn-primary"));
        assert!(sheet.rules.contains_key("btn-secondary"));
        assert_eq!(sheet.rules["chip"].variants[0].modifiers, ["selected"]);
        assert_eq!(canonical("action-primary"), "btn-primary");
        assert_eq!(canonical("button.danger"), "btn-danger");
        assert_eq!(canonical("chip"), "chip");
    }

    #[test]
    fn missing_transition_uses_the_fast_token() {
        assert_eq!(seconds(None), crate::motion::fast());
        assert_eq!(seconds(Some("0s")), 0.0);
    }

    #[test]
    fn kind_defaults_sit_under_the_class() {
        let sheet = Sheet::parse(
            "row { gap: 8; padding: 4; } .tight { gap: 2; } text { font-size: 15; }",
        )
        .expect("parses");
        let gap = |modifiers: &[&str]| {
            let mut style = Style::blank();
            for layer in layers(&sheet, "tight", modifiers) {
                apply(&mut style, &layer.base);
            }
            (style.gap, style.padding)
        };
        assert_eq!(gap(&["<row>"]), (2.0, 4.0));
        assert_eq!(gap(&["<text>"]).1, 0.0);
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
    fn media_lists_tv_height_and_input_parse_from_css() {
        let sheet = Sheet::parse(
            "@media (size: compact), (max-height: 479px) { .a { display: none; } }\n\
             @media (tv) { .a { gap: 9px; } }\n\
             @media (input: touch) { .a { gap: 3px; } }",
        )
        .expect("parses");
        let queries: Vec<Query> = sheet.rules["a"]
            .media
            .iter()
            .map(|(query, _)| query.clone())
            .collect();
        assert_eq!(queries.len(), 4);
        for expected in [
            Query::Window("compact".to_owned()),
            Query::MaxHeight(479.0),
            Query::Platform("tv".to_owned()),
            Query::Input("touch".to_owned()),
        ] {
            assert!(queries.contains(&expected));
        }
        assert!(Sheet::parse("@media (input: stylus) { .a { gap: 1px; } }").is_err());
    }

    #[test]
    fn hero_sizes_come_from_the_bundled_sheet() {
        let sheet = Sheet::parse(DEFAULT_SHEET).expect("parses");
        let mut style = Style::blank();
        for layer in layers(&sheet, "hero-sizes", &[]) {
            apply(&mut style, &layer.base);
        }
        assert_eq!(style.width, Some(980.0));
        assert_eq!(style.scale, 1.65);
        assert!(sheet.rules["hero-metadata-size"].media.len() >= 3);
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
                |_| half = Some(animate(&context, &sheet, animation, Style::blank())),
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
        apply(&mut style, &rule.layer.base);
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
        let variants = within(&[Frame::plain("bar".into(), 0)], || stateful(&sheet, "label", &[]));
        assert_eq!(variants.len(), 1);
        let idle = within(&[Frame::plain("bar".into(), 0)], || ancestors_active(variants[0]));
        let hot = within(&[Frame::plain("bar".into(), HOVERED)], || ancestors_active(variants[0]));
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
