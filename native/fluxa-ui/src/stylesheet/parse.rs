use super::*;

pub(super) fn sheet_from_value(root: &Value, lenient: bool) -> Result<Sheet, String> {
    let object = root.as_object().ok_or("style sheet must be an object")?;
    let mut sheet = Sheet::default();
    for (key, value) in object {
        if let Err(error) = add_entry(&mut sheet, key, value) {
            if !lenient {
                return Err(error);
            }
            eprintln!("style rule skipped: {error}");
        }
    }
    for rule in sheet.rules.values_mut() {
        rule.variants
            .sort_by_key(|variant| {
                (
                    variant.ancestors.len() + variant.modifiers.len(),
                    variant.query.is_some(),
                )
            });
    }
    Ok(sheet)
}

fn add_entry(sheet: &mut Sheet, key: &str, value: &Value) -> Result<(), String> {
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
            let layer = layer_from(selector, layer)?;
            let rule = sheet.rules.entry(target.name).or_default();
            if target.ancestors.is_empty() && target.modifiers.is_empty() {
                rule.media.push((query.clone(), layer));
            } else {
                rule.variants.push(Variant {
                    ancestors: target.ancestors,
                    modifiers: target.modifiers,
                    query: Some(query.clone()),
                    layer,
                });
            }
        }
    } else {
        let target = Selector::parse(key)?;
        let layer = layer_from(key, value)?;
        let rule = sheet.rules.entry(target.name.clone()).or_default();
        if target.ancestors.is_empty() && target.modifiers.is_empty() {
            rule.layer = layer;
            rule_options(rule, key, value)?;
        } else {
            rule.variants.push(Variant {
                ancestors: target.ancestors,
                modifiers: target.modifiers,
                query: None,
                layer,
            });
        }
    }
    Ok(())
}

impl Selector {
    fn parse(text: &str) -> Result<Self, String> {
        let mut parts: Vec<&str> = text.split_whitespace().collect();
        let last = parts.pop().ok_or("empty selector")?;
        let (name, modifiers) = match last.split_once('[') {
            Some((name, rest)) => (
                name,
                rest.strip_suffix(']')
                    .ok_or_else(|| format!("{text}: missing ]"))?
                    .split(',')
                    .map(|modifier| modifier.trim().to_owned())
                    .collect(),
            ),
            None => (last, Vec::new()),
        };
        let (name, modifiers) = match name {
            "chip.selected" => ("chip", vec!["selected".to_owned()]),
            other => (super::canonical(other), modifiers),
        };
        Ok(Self {
            ancestors: parts
                .into_iter()
                .map(|part| super::canonical(part).to_owned())
                .collect(),
            name: name.to_owned(),
            modifiers,
        })
    }
}

impl Query {
    pub(super) fn parse(key: &str) -> Result<Self, String> {
        let body = key.trim_start_matches('@');
        if let Some(width) = body.strip_prefix("min-width:") {
            return Ok(Query::MinWidth(number(key, width)?));
        }
        if let Some(width) = body.strip_prefix("max-width:") {
            return Ok(Query::MaxWidth(number(key, width)?));
        }
        if let Some(height) = body.strip_prefix("min-height:") {
            return Ok(Query::MinHeight(number(key, height)?));
        }
        if let Some(height) = body.strip_prefix("max-height:") {
            return Ok(Query::MaxHeight(number(key, height)?));
        }
        if let Some(input) = body.strip_prefix("input:") {
            return match input {
                "touch" | "pointer" | "focus" => Ok(Query::Input(input.to_owned())),
                other => Err(format!("{key}: unknown input {other}")),
            };
        }
        if let Some(theme) = body.strip_prefix("theme:") {
            return Ok(Query::Theme(theme.to_owned()));
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
    let view = |key: &str, unit: &str| -> Option<f32> {
        let text = value.get(key)?.as_str()?.strip_suffix(unit)?;
        text.trim()
            .parse::<f32>()
            .ok()
            .map(|percent| percent / 100.0)
    };
    let length = |key: &str| -> Result<Option<Length>, String> {
        value.get(key).map_or(Ok(None), |raw| {
            if raw
                .as_str()
                .is_some_and(|text| text.ends_with("vw") || text.ends_with("vh"))
            {
                return Ok(None);
            }
            Length::parse(raw).map(Some).ok_or_else(|| {
                format!("{rule}.{key}: expected a px number or a space token like \"s12\"")
            })
        })
    };
    let gradient = match value.get("gradient").and_then(Value::as_str) {
        Some(text) => Some(parse_gradient(rule, text)?),
        None => None,
    };
    let shadow = match value.get("shadow").and_then(Value::as_str) {
        Some(text) => Some(parse_shadow(rule, text)?),
        None => None,
    };
    let custom: Vec<(String, String)> = value
        .get("vars")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(name, raw)| Some((name.clone(), raw.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    Ok(Patch {
        vars: super::scoped::intern(custom),
        auto_states: match value.get("states").and_then(Value::as_str) {
            Some("auto") => Some(true),
            Some("none") => Some(false),
            Some(other) => return Err(format!("{rule}.states: expected auto or none, found {other}")),
            None => None,
        },
        gradient,
        shadow,
        margin: length("margin")?,
        width: length("width")?,
        width_vw: view("width", "vw"),
        column_width: length("column-width")?,
        columns: value
            .get("columns")
            .and_then(|columns| match columns {
                Value::String(text) => text.trim().parse::<u32>().ok(),
                other => other.as_f64().map(|columns| columns as u32),
            })
            .filter(|columns| *columns > 0),
        height_vh: view("height", "vh"),
        height: length("height")?,
        weight: value
            .get("weight")
            .and_then(Value::as_f64)
            .map(|v| v as f32),
        aspect: value
            .get("aspect")
            .and_then(Value::as_f64)
            .filter(|ratio| *ratio > 0.0)
            .map(|v| v as f32),
        stretch: value
            .get("stretch")
            .and_then(Value::as_f64)
            .map(|v| v as f32),
        squash: value
            .get("squash")
            .and_then(Value::as_f64)
            .map(|v| v as f32),
        magnify: value
            .get("magnify")
            .and_then(Value::as_f64)
            .map(|v| v as f32),
        reach: value
            .get("reach")
            .and_then(Value::as_f64)
            .filter(|v| *v > 0.0)
            .map(|v| v as f32),
        backdrop: value
            .get("backdrop")
            .and_then(Value::as_f64)
            .map(|v| v as f32),
        padding: length("padding")?,
        gap: length("gap")?,
        max_width: length("max-width")?,
        font_size: length("font-size")?,
        direction: match value.get("direction").and_then(Value::as_str) {
            Some("row") => Some(super::Direction::Row),
            Some("column") => Some(super::Direction::Column),
            Some(other) => return Err(format!("{rule}.direction: unknown {other}")),
            None => None,
        },
        align: match value.get("align").and_then(Value::as_str) {
            Some("start") => Some(super::Alignment::Start),
            Some("center") => Some(super::Alignment::Center),
            Some("end") => Some(super::Alignment::End),
            Some("stretch") => Some(super::Alignment::Stretch),
            Some(other) => return Err(format!("{rule}.align: unknown {other}")),
            None => None,
        },
        justify: match value.get("justify").and_then(Value::as_str) {
            Some("start") => Some(super::Justify::Start),
            Some("center") => Some(super::Justify::Center),
            Some("end") => Some(super::Justify::End),
            Some("space-between") => Some(super::Justify::SpaceBetween),
            Some("space-around") => Some(super::Justify::SpaceAround),
            Some("space-evenly") => Some(super::Justify::SpaceEvenly),
            Some(other) => return Err(format!("{rule}.justify: unknown {other}")),
            None => None,
        },
        grow: value
            .get("grow")
            .and_then(Value::as_f64)
            .filter(|grow| *grow >= 0.0)
            .map(|grow| grow as f32),
        fill_space: value
            .get("flex-fill")
            .and_then(Value::as_f64)
            .map(|fill| fill > 0.0),
        hidden: match value.get("display").and_then(Value::as_str) {
            Some("none") => Some(true),
            Some(_) => Some(false),
            None => None,
        },
        padding_box: match value.get("padding_box").and_then(Value::as_array) {
            Some(sides) if sides.len() == 4 => {
                let mut parsed = [Length::Px(0.0); 4];
                for (slot, side) in parsed.iter_mut().zip(sides) {
                    *slot = Length::parse(side)
                        .ok_or_else(|| format!("{rule}.padding: bad value {side}"))?;
                }
                Some(parsed)
            }
            Some(_) => return Err(format!("{rule}.padding: expected 1 to 4 values")),
            None => None,
        },
        order: value
            .get("order")
            .and_then(Value::as_f64)
            .map(|order| order as i32),
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
                "theme.input.focus" => crate::vars::px("input-focus-width"),
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

fn top_level(text: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c == separator && depth == 0 => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }
    parts.push(current);
    parts
        .into_iter()
        .map(|part| part.trim().to_owned())
        .filter(|part| !part.is_empty())
        .collect()
}

fn parse_gradient(rule: &str, text: &str) -> Result<GradientSpec, String> {
    let inner = text
        .trim()
        .strip_prefix("linear-gradient(")
        .and_then(|rest| rest.strip_suffix(')'))
        .ok_or_else(|| format!("{rule}.gradient: expected linear-gradient(...)"))?;
    let mut parts = top_level(inner, ',');
    let mut angle = 180.0;
    if let Some(first) = parts.first()
        && let Some(direction) = gradient_angle(first)
    {
        angle = direction;
        parts.remove(0);
    }
    let stops: Vec<_> = parts
        .iter()
        .map(|part| {
            let mut position = None;
            let mut color = None;
            for word in top_level(part, ' ') {
                match word.strip_suffix('%').map(str::parse::<f32>) {
                    Some(Ok(percent)) => position = Some(percent / 100.0),
                    _ => color = Some(word),
                }
            }
            let color = color.ok_or_else(|| format!("{rule}.gradient: stop without a color"))?;
            let paint =
                parse_paint(&color).ok_or_else(|| format!("{rule}.gradient: bad color {color}"))?;
            Ok((position, paint))
        })
        .collect::<Result<_, String>>()?;
    if stops.len() < 2 || stops.len() > super::MAX_GRADIENT_STOPS {
        return Err(format!(
            "{rule}.gradient: use 2 to {} color stops",
            super::MAX_GRADIENT_STOPS
        ));
    }
    Ok(GradientSpec { angle, stops })
}

fn gradient_angle(word: &str) -> Option<f32> {
    if let Some(degrees) = word.strip_suffix("deg") {
        return degrees.trim().parse().ok();
    }
    let direction = word.strip_prefix("to ")?;
    let mut sorted: Vec<&str> = direction.split_whitespace().collect();
    sorted.sort_unstable();
    Some(match sorted.as_slice() {
        ["top"] => 0.0,
        ["right"] => 90.0,
        ["bottom"] => 180.0,
        ["left"] => 270.0,
        ["right", "top"] => 45.0,
        ["bottom", "right"] => 135.0,
        ["bottom", "left"] => 225.0,
        ["left", "top"] => 315.0,
        _ => return None,
    })
}

fn parse_shadow(rule: &str, text: &str) -> Result<GlowSpec, String> {
    let text = text.trim();
    if text == "none" {
        return Ok(GlowSpec {
            x: 0.0,
            y: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: Paint::Rgba(Color32::TRANSPARENT),
        });
    }
    let mut lengths = Vec::new();
    let mut color = None;
    for word in top_level(text, ' ') {
        match word.trim_end_matches("px").parse::<f32>() {
            Ok(number) => lengths.push(number),
            Err(_) => color = Some(word),
        }
    }
    let color = color.ok_or_else(|| format!("{rule}.shadow: missing color"))?;
    let color = parse_paint(&color).ok_or_else(|| format!("{rule}.shadow: bad color {color}"))?;
    if lengths.len() < 2 || lengths.len() > 4 {
        return Err(format!("{rule}.shadow: expected x y [blur [spread]] color"));
    }
    Ok(GlowSpec {
        x: lengths[0],
        y: lengths[1],
        blur: lengths.get(2).copied().unwrap_or(0.0),
        spread: lengths.get(3).copied().unwrap_or(0.0),
        color,
    })
}
