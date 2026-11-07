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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Corner {
    Pill,
    Px(f32),
}

impl Corner {
    pub(crate) fn of(self, height: f32) -> f32 {
        match self {
            Corner::Pill => height * crate::metrics::active().pill_ratio,
            Corner::Px(px) => px,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct State {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
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
}

#[derive(Clone, Debug, Default)]
struct Rule {
    base: Patch,
    hover: Patch,
    pressed: Patch,
    focused: Patch,
    disabled: Patch,
    transition: Option<String>,
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
        disabled: field("disabled")?.unwrap_or_default(),
        transition: value
            .get("transition")
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
    Ok(Patch {
        fill: paint("fill")?,
        text: paint("text")?,
        border: paint("border")?,
        scale: value.get("scale").and_then(Value::as_f64).map(|v| v as f32),
        radius: match value.get("radius") {
            Some(Value::String(text)) if text == "pill" => Some(Corner::Pill),
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
}

fn mix(from: Style, to: Style, t: f32) -> Style {
    Style {
        fill: from.fill.lerp_to_gamma(to.fill, t),
        text: from.text.lerp_to_gamma(to.text, t),
        border: from.border.lerp_to_gamma(to.border, t),
        scale: from.scale + (to.scale - from.scale) * t,
        radius: to.radius,
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
            border: Color32::TRANSPARENT,
            scale: 1.0,
            radius: Corner::Px(0.0),
        };
    };
    let mut base = Style {
        fill: Color32::TRANSPARENT,
        text: text_primary(),
        border: Color32::TRANSPARENT,
        scale: 1.0,
        radius: Corner::Px(0.0),
    };
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
            crate::motion::Ease::Linear,
        )
    };
    let mut result = base;
    for (key, on, patch) in [
        ("hover", state.hovered, &rule.hover),
        ("focus", state.focused, &rule.focused),
        ("press", state.pressed, &rule.pressed),
    ] {
        let t = amount(key, on);
        if t > 0.0 {
            let mut target = result;
            apply(&mut target, patch, metrics);
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
