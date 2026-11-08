use super::*;

pub(super) fn sheet_from_value(root: &Value) -> Result<Sheet, String> {
    let object = root.as_object().ok_or("style sheet must be an object")?;
    let mut sheet = Sheet::default();
    for (key, value) in object {
        if let Some(name) = key.strip_prefix("@keyframes ") {
            sheet
                .keyframes
                .insert(name.trim().to_owned(), keyframes(name, value)?);
        } else if key.starts_with('@') {
            let query = Query::parse(key)?;
            let rules = value
                .as_object()
                .ok_or_else(|| format!("{key}: expected an object of rules"))?;
            for (selector, layer) in rules {
                let target = Selector::parse(selector)?;
                if !target.ancestors.is_empty() || target.modifier.is_some() {
                    return Err(format!("{key} {selector}: media rules take a plain name"));
                }
                sheet
                    .rules
                    .entry(target.name)
                    .or_default()
                    .media
                    .push((query.clone(), layer_from(selector, layer)?));
            }
        } else {
            let target = Selector::parse(key)?;
            let layer = layer_from(key, value)?;
            let rule = sheet.rules.entry(target.name.clone()).or_default();
            if target.ancestors.is_empty() && target.modifier.is_none() {
                rule.layer = layer;
                rule_options(rule, key, value)?;
            } else {
                rule.variants.push(Variant {
                    ancestors: target.ancestors,
                    modifier: target.modifier,
                    layer,
                });
            }
        }
    }
    for rule in sheet.rules.values_mut() {
        rule.variants.sort_by_key(|variant| {
            variant.ancestors.len() + usize::from(variant.modifier.is_some())
        });
    }
    Ok(sheet)
}

impl Selector {
    fn parse(text: &str) -> Result<Self, String> {
        let mut parts: Vec<&str> = text.split_whitespace().collect();
        let last = parts.pop().ok_or("empty selector")?;
        let (name, modifier) = match last.split_once('[') {
            Some((name, rest)) => (
                name,
                Some(
                    rest.strip_suffix(']')
                        .ok_or_else(|| format!("{text}: missing ]"))?
                        .to_owned(),
                ),
            ),
            None => (last, None),
        };
        Ok(Self {
            ancestors: parts.into_iter().map(ToOwned::to_owned).collect(),
            name: name.to_owned(),
            modifier,
        })
    }
}

impl Query {
    fn parse(key: &str) -> Result<Self, String> {
        let body = key.trim_start_matches('@');
        if let Some(width) = body.strip_prefix("min-width:") {
            return Ok(Query::MinWidth(number(key, width)?));
        }
        if let Some(width) = body.strip_prefix("max-width:") {
            return Ok(Query::MaxWidth(number(key, width)?));
        }
        match body {
            "mobile" | "desktop" | "tv" => Ok(Query::Platform(body.to_owned())),
            "compact" | "medium" | "expanded" => Ok(Query::Window(body.to_owned())),
            _ => Err(format!("{key}: unknown media query")),
        }
    }
}

fn number(key: &str, text: &str) -> Result<f32, String> {
    text.trim()
        .trim_end_matches("px")
        .parse()
        .map_err(|_| format!("{key}: expected a number"))
}

fn keyframes(name: &str, value: &Value) -> Result<Vec<(f32, Patch)>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("@keyframes {name}: expected an object"))?;
    let mut frames = Vec::new();
    for (stop, patch) in object {
        let at = match stop.as_str() {
            "from" => 0.0,
            "to" => 1.0,
            percent => number(name, percent.trim_end_matches('%'))? / 100.0,
        };
        frames.push((at, parse_patch(name, patch)?));
    }
    frames.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(frames)
}

fn rule_options(rule: &mut Rule, name: &str, value: &Value) -> Result<(), String> {
    let text = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    };
    rule.transition = text("transition");
    rule.easing = text("easing");
    rule.extends = text("extends");
    rule.animation = text("animation")
        .map(|spec| Animation::parse(name, &spec))
        .transpose()?;
    if let Some(object) = value.as_object() {
        for (key, nested) in object.iter().filter(|(key, _)| key.starts_with('@')) {
            rule.media
                .push((Query::parse(key)?, layer_from(name, nested)?));
        }
    }
    Ok(())
}

impl Animation {
    fn parse(rule: &str, spec: &str) -> Result<Self, String> {
        let mut animation = Animation {
            name: String::new(),
            duration: 1.0,
            easing: Easing::Linear,
            infinite: false,
            alternate: false,
        };
        for word in spec.split_whitespace() {
            match word {
                "infinite" => animation.infinite = true,
                "alternate" => animation.alternate = true,
                "linear" => animation.easing = Easing::Linear,
                "ease-out" | "out" => animation.easing = Easing::Out,
                other => {
                    if let Some(ms) = other.strip_suffix("ms") {
                        animation.duration = number(rule, ms)? / 1000.0;
                    } else if let Some(seconds) = other.strip_suffix('s') {
                        animation.duration = number(rule, seconds)?;
                    } else if animation.name.is_empty() {
                        animation.name = other.to_owned();
                    } else {
                        return Err(format!("{rule}.animation: unexpected {other}"));
                    }
                }
            }
        }
        if animation.name.is_empty() {
            return Err(format!("{rule}.animation: missing keyframes name"));
        }
        Ok(animation)
    }
}

fn layer_from(rule: &str, value: &Value) -> Result<Layer, String> {
    let state = |key: &str| {
        value
            .get(key)
            .map(|nested| parse_patch(rule, nested))
            .transpose()
            .map(Option::unwrap_or_default)
    };
    Ok(Layer {
        base: parse_patch(rule, value)?,
        hover: state("hover")?,
        pressed: state("pressed")?,
        focused: state("focused")?,
        selected: state("selected")?,
        disabled: state("disabled")?,
    })
}

pub(super) fn parse_patch(rule: &str, value: &Value) -> Result<Patch, String> {
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
            Some(Value::String(text)) if text == "dialog" => Some(Corner::Dialog),
            Some(Value::Number(number)) => number.as_f64().map(|px| Corner::Px(px as f32)),
            Some(_) => return Err(format!("{rule}.radius: expected \"pill\" or a number")),
            None => None,
        },
    })
}
