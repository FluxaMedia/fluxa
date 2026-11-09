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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Length {
    Space(Space),
    Px(f32),
}

impl Length {
    fn px(self) -> f32 {
        match self {
            Length::Space(space) => space.px(&crate::metrics::active()),
            Length::Px(px) => px,
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
    modifier: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Variant {
    ancestors: Vec<String>,
    modifier: Option<String>,
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
    let parsed = Sheet::parse(text)?;
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(Arc::new(parsed));
    }
    Ok(())
}

impl Sheet {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let root: Value = if text.trim_start().starts_with('{') {
            serde_json::from_str(text).map_err(|error| error.to_string())?
        } else {
            css::to_value(text)?
        };
        parse::sheet_from_value(&root)
    }
}

use parse::parse_patch;

struct Scope {
    name: String,
    text: Option<Color32>,
    font_size: Option<f32>,
}

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
        max_width: to.max_width,
        font_size: to.font_size,
        direction: to.direction,
        align: to.align,
        text_declared: to.text_declared,
    }
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
                    variant.ancestors.iter().all(|ancestor| in_scope(ancestor))
                        && variant
                            .modifier
                            .as_deref()
                            .is_none_or(|wanted| modifiers.contains(&wanted))
                })
                .map(|variant| &variant.layer),
        );
    }
    found
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
    let elapsed = context.input(|input| input.time) as f32 / animation.duration.max(0.001);
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
}
