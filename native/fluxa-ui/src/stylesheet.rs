use super::*;
use serde_json::Value;
use std::sync::{Arc, RwLock};

const DEFAULT_SHEET: &str = include_str!("../../../shared/contracts/ui-styles.json");

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
    Px(f32),
}

impl Corner {
    pub(crate) fn of(self, height: f32) -> f32 {
        match self {
            Corner::Pill => height * crate::metrics::active().pill_ratio,
            Corner::Card => crate::metrics::active().card_radius,
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

#[derive(Clone, Debug)]
enum Paint {
    Rgba(Color32),
    Ink(u8),
    Named(String, f32),
}

#[derive(Clone, Debug, Default)]
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
}

#[derive(Clone, Debug, Default)]
struct Rule {
    base: Patch,
    hover: Patch,
    pressed: Patch,
    focused: Patch,
    selected: Patch,
    disabled: Patch,
    transition: Option<String>,
    easing: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Sheet {
    rules: std::collections::HashMap<String, Rule>,
}

static SHEET: RwLock<Option<Arc<Sheet>>> = RwLock::new(None);

pub(crate) fn sheet() -> Arc<Sheet> {
    if let Some(sheet) = SHEET.read().ok().and_then(|slot| slot.clone()) {
        return sheet;
    }
    let parsed = Arc::new(Sheet::parse(DEFAULT_SHEET).expect("bundled ui-styles.json"));
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(parsed.clone());
    }
    parsed
}

pub fn reload_styles(json: &str) -> Result<(), String> {
    let parsed = Sheet::parse(json)?;
    if let Ok(mut slot) = SHEET.write() {
        *slot = Some(Arc::new(parsed));
    }
    Ok(())
}

impl Sheet {
    pub(crate) fn parse(json: &str) -> Result<Self, String> {
        let root: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
        let object = root.as_object().ok_or("style sheet must be an object")?;
        let mut rules = std::collections::HashMap::new();
        for (name, value) in object {
            rules.insert(name.clone(), parse_rule(name, value)?);
        }
        Ok(Self { rules })
    }
}

fn parse_rule(name: &str, value: &Value) -> Result<Rule, String> {
    let patch = |value: &Value| parse_patch(name, value);
    let field = |key: &str| value.get(key).map(patch).transpose();
    Ok(Rule {
        base: patch(value)?,
        hover: field("hover")?.unwrap_or_default(),
        pressed: field("pressed")?.unwrap_or_default(),
        focused: field("focused")?.unwrap_or_default(),
        selected: field("selected")?.unwrap_or_default(),
        disabled: field("disabled")?.unwrap_or_default(),
        transition: value
            .get("transition")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        easing: value
            .get("easing")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn parse_patch(rule: &str, value: &Value) -> Result<Patch, String> {
    let paint = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(|text| parse_paint(text).ok_or_else(|| format!("{rule}.{key}: bad color {text}")))
            .transpose()
    };
    let length = |key: &str| -> Result<Option<Length>, String> {
        value.get(key).map_or(Ok(None), |raw| {
            Length::parse(raw).map(Some).ok_or_else(|| {
                format!("{rule}.{key}: expected a px number or a space token like \"s12\"")
            })
        })
    };
    Ok(Patch {
        padding: length("padding")?,
        gap: length("gap")?,
        max_width: length("max-width")?,
        font_size: length("font-size")?,
        opacity: value
            .get("opacity")
            .and_then(Value::as_f64)
            .map(|v| v as f32),
        fill: paint("fill")?,
        text: paint("text")?,
        border: paint("border")?,
        scale: value.get("scale").and_then(Value::as_f64).map(|v| v as f32),
        stroke: match value.get("stroke") {
            Some(Value::String(text)) => Some(match text.as_str() {
                "hairline" => stroke::hairline(),
                "theme.input.focus" => crate::metrics::active().input_focus_width,
                other => return Err(format!("{rule}.stroke: unknown width {other}")),
            }),
            Some(Value::Number(number)) => number.as_f64().map(|width| width as f32),
            Some(_) => return Err(format!("{rule}.stroke: expected a width")),
            None => None,
        },
        radius: match value.get("radius") {
            Some(Value::String(text)) if text == "pill" => Some(Corner::Pill),
            Some(Value::String(text)) if text == "card" => Some(Corner::Card),
            Some(Value::Number(number)) => number.as_f64().map(|px| Corner::Px(px as f32)),
            Some(_) => return Err(format!("{rule}.radius: expected \"pill\" or a number")),
            None => None,
        },
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
    }
}

fn seconds(name: Option<&str>) -> f32 {
    match name {
        Some("fast") => crate::motion::fast(),
        Some("normal") => crate::motion::normal(),
        Some("slow") => crate::motion::slow(),
        Some(other) => other.trim_end_matches('s').parse().unwrap_or(0.0),
        None => 0.0,
    }
}

pub(crate) fn look(context: &egui::Context, rule: &str) -> Style {
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
    let Some(rule) = sheet.rules.get(rule) else {
        return Color32::MAGENTA;
    };
    let mut base = Style::blank();
    apply(&mut base, &rule.base, None);
    base.fill
}

pub(crate) fn resolve(
    context: &egui::Context,
    rule: &str,
    id: Id,
    state: State,
    metrics: Option<&UiMetrics>,
) -> Style {
    let sheet = sheet();
    let Some(rule) = sheet.rules.get(rule) else {
        return Style {
            fill: Color32::MAGENTA,
            text: Color32::MAGENTA,
            ..Style::blank()
        };
    };
    let mut base = Style::blank();
    apply(&mut base, &rule.base, metrics);
    if !state.enabled {
        apply(&mut base, &rule.disabled, metrics);
        return base;
    }
    let duration = seconds(rule.transition.as_deref());
    let amount = |key: &str, on: bool| {
        crate::motion::ease(
            context,
            id.with(key),
            f32::from(u8::from(on)),
            duration,
            match rule.easing.as_deref() {
                Some("token") => crate::motion::Ease::Token,
                Some("out") => crate::motion::Ease::Out,
                _ => crate::motion::Ease::Linear,
            },
        )
    };
    let mut result = base;
    for (key, on, patch) in [
        ("select", state.selected, &rule.selected),
        ("hover", state.hovered, &rule.hover),
        ("focus", state.focused, &rule.focused),
        ("press", state.pressed, &rule.pressed),
    ] {
        let t = amount(key, on);
        if t > 0.0 {
            let mut target = result;
            apply(&mut target, patch, metrics);
            target.selected = if key == "select" {
                1.0
            } else {
                result.selected
            };
            result = mix(result, target, t);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(rule: &str, state: State) -> Style {
        let context = egui::Context::default();
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
        apply(&mut style, &sheet.rules["a"].base, None);
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
    fn unknown_color_names_are_rejected_at_parse_time_only_for_bad_syntax() {
        assert!(Sheet::parse(r##"{"a": {"fill": "#zz"}}"##).is_err());
        assert!(Sheet::parse(r#"{"a": {"fill": "ink/soft"}}"#).is_ok());
    }
}
