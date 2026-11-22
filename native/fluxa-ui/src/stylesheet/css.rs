use serde_json::{Map, Value};
use std::collections::HashMap;

pub(super) const KINDS: [&str; 9] = [
    "row", "column", "stack", "grid", "text", "icon", "image", "scroll", "spacer",
];

const STATES: [&str; 8] = [
    "hover", "pressed", "focused", "selected", "disabled", "focus", "active", "checked",
];

fn state_name(state: &str) -> &str {
    match state {
        "focus" => "focused",
        "active" => "pressed",
        "checked" => "selected",
        other => other,
    }
}

fn property_name(name: &str) -> &str {
    match name {
        "background" | "background-color" => "fill",
        "color" => "text",
        "border-radius" => "radius",
        "border-color" => "border",
        "border-width" => "stroke",
        "flex-direction" => "direction",
        "align-items" => "align",
        "justify-content" => "justify",
        "box-shadow" => "shadow",
        other => other,
    }
}

fn property_value(name: &str, value: String) -> String {
    match (name, value.as_str()) {
        ("radius", v) => match v.trim_end_matches("px").parse::<f64>() {
            Ok(px) if px >= 500.0 => "pill".to_owned(),
            _ => value,
        },
        ("align" | "justify", "flex-start") => "start".to_owned(),
        ("align" | "justify", "flex-end") => "end".to_owned(),
        ("direction", "row-reverse" | "column-reverse") => value,
        _ => value,
    }
}

pub(super) type RootLayer = (Vec<String>, Vec<(String, String)>);

pub(super) fn root_layers(text: &str) -> Result<Vec<RootLayer>, String> {
    let text = strip_comments(text);
    let mut layers = Vec::new();
    for (prelude, body) in blocks(&text)? {
        let prelude = prelude.trim();
        if prelude == ":root" {
            layers.push((Vec::new(), custom_properties(&body)?));
        } else if prelude.starts_with("@media") {
            let keys = media_keys(prelude.trim_start_matches('@'))?;
            for (inner, inner_body) in blocks(&body)? {
                if inner.trim() == ":root" {
                    layers.push((keys.clone(), custom_properties(&inner_body)?));
                }
            }
        }
    }
    Ok(layers)
}

fn custom_properties(body: &str) -> Result<Vec<(String, String)>, String> {
    Ok(declarations(body)?
        .into_iter()
        .filter_map(|(name, value)| Some((name.strip_prefix("--")?.to_owned(), value)))
        .collect())
}

pub(super) fn to_value(text: &str) -> Result<Value, String> {
    let text = strip_comments(text);
    let mut vars = HashMap::new();
    let mut motion = Map::new();
    let blocks = blocks(&text)?;
    for (prelude, body) in &blocks {
        if prelude.trim() == ":root" {
            for (name, value) in declarations(body)? {
                if let Some(name) = name.strip_prefix("--") {
                    vars.insert(name.to_owned(), value);
                } else if let Some(pace) = name.strip_prefix("motion-") {
                    motion.insert(pace.to_owned(), Value::String(value));
                }
            }
        }
    }
    let mut root = Map::new();
    if !motion.is_empty() {
        root.insert("@motion".to_owned(), Value::Object(motion));
    }
    for (prelude, body) in &blocks {
        let prelude = prelude.trim();
        if prelude == ":root" {
            continue;
        }
        if let Some(name) = prelude.strip_prefix("@keyframes") {
            let mut frames = Map::new();
            for (stops, frame) in self::blocks(body)? {
                let props = properties(&declarations(&frame)?, &vars)?;
                for stop in stops.split(',') {
                    frames.insert(stop.trim().to_owned(), Value::Object(props.clone()));
                }
            }
            root.insert(format!("@keyframes {}", name.trim()), Value::Object(frames));
        } else if let Some(query) = prelude.strip_prefix('@') {
            for key in media_keys(query)? {
                let key = format!("@{key}");
                let mut rules = match root.remove(&key) {
                    Some(Value::Object(earlier)) => earlier,
                    _ => Map::new(),
                };
                for (selectors, rule) in self::blocks(body)? {
                    if selectors.trim() == ":root" {
                        continue;
                    }
                    add_rule(&mut rules, &selectors, &rule, &vars)?;
                }
                root.insert(key, Value::Object(rules));
            }
        } else {
            add_rule(&mut root, prelude, body, &vars)?;
        }
    }
    Ok(Value::Object(root))
}

fn media_keys(query: &str) -> Result<Vec<String>, String> {
    let query = query.trim();
    let Some(inner) = query.strip_prefix("media") else {
        return Ok(vec![query.to_owned()]);
    };
    inner.split(',').map(media_key).collect()
}

fn media_key(part: &str) -> Result<String, String> {
    let inner = part.trim().trim_start_matches('(').trim_end_matches(')');
    let Some((feature, value)) = inner.split_once(':') else {
        return match inner.trim() {
            "tv" => Ok("tv".to_owned()),
            other => Err(format!("@media {other}: expected (feature: value)")),
        };
    };
    let value = value.trim().trim_end_matches("px");
    match feature.trim() {
        "min-width" | "max-width" | "min-height" | "max-height" | "input" | "theme" => {
            Ok(format!("{}:{}", feature.trim(), value))
        }
        "platform" | "window" | "size" => Ok(value.to_owned()),
        other => Err(format!("@media: unknown feature {other}")),
    }
}

fn add_rule(
    into: &mut Map<String, Value>,
    selectors: &str,
    body: &str,
    vars: &HashMap<String, String>,
) -> Result<(), String> {
    let props = properties(&declarations(body)?, vars)?;
    for selector in selectors.split(',') {
        let (key, state) = translate_selector(selector.trim())?;
        let key = key.as_str();
        let slot = into
            .entry(key.to_owned())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| format!("{key}: not a rule"))?;
        let target = match state {
            Some(state) => slot
                .entry(state.to_owned())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| format!("{key}:{state}: not a rule"))?,
            None => slot,
        };
        for (name, value) in &props {
            match (name.as_str(), target.get_mut(name), value) {
                ("vars", Some(Value::Object(earlier)), Value::Object(more)) => {
                    earlier.extend(more.clone());
                }
                _ => {
                    target.insert(name.clone(), value.clone());
                }
            }
        }
    }
    Ok(())
}

fn translate_selector(selector: &str) -> Result<(String, Option<&'static str>), String> {
    let spaced = selector.replace('>', " ");
    let parts = split_top(&spaced);
    let Some((last, ancestors)) = parts.split_last() else {
        return Err("empty selector".to_owned());
    };
    let mut keys = Vec::new();
    for part in ancestors {
        let (base, pseudos) = split_pseudos(part);
        let mut state = None;
        for pseudo in &pseudos {
            match STATES.iter().find(|known| **known == pseudo.as_str()) {
                Some(known) => state = Some(state_name(known)),
                None => return Err(format!("{part}: :{pseudo} works only on the last selector")),
            }
        }
        for class in classes(base) {
            keys.push(match state {
                Some(state) => format!("{class}:{state}"),
                None => class.to_owned(),
            });
        }
    }
    let (base, pseudos) = split_pseudos(last);
    let mut state = None;
    let mut modifiers: Vec<String> = Vec::new();
    for pseudo in &pseudos {
        if let Some(known) = STATES.iter().find(|known| **known == pseudo.as_str()) {
            state = Some(state_name(known));
        } else if matches!(pseudo.as_str(), "first-child" | "last-child" | "only-child") {
            modifiers.push(pseudo.clone());
        } else if let Some(arg) = functional(pseudo, "nth-child") {
            if !(arg == "odd" || arg == "even" || arg.parse::<u32>().is_ok()) {
                return Err(format!("{last}: nth-child takes a number, odd or even"));
            }
            modifiers.push(format!("nth-child({arg})"));
        } else if let Some(arg) = functional(pseudo, "not") {
            if let Some(class) = arg.strip_prefix('.') {
                modifiers.push(format!("!{class}"));
            } else if let Some(name) = arg.strip_prefix(':') {
                modifiers.push(format!("!{name}"));
            } else {
                return Err(format!(
                    "{last}: :not() takes a class or :first-child style pseudo"
                ));
            }
        } else {
            return Err(format!("{last}: unsupported pseudo-class :{pseudo}"));
        }
    }
    let target = match base.strip_prefix('.') {
        None if KINDS.contains(&base) => format!("<{base}>"),
        Some(rest) => {
            let mut names = rest.split('.');
            let name = names.next().unwrap_or_default().to_owned();
            modifiers.splice(0..0, names.map(str::to_owned));
            name
        }
        None => base.to_owned(),
    };
    if target.is_empty() || target == "*" {
        return Err(format!("{last}: a selector needs a class or rule name"));
    }
    let target = if modifiers.is_empty() {
        target
    } else {
        format!("{target}[{}]", modifiers.join(","))
    };
    keys.push(target);
    Ok((keys.join(" "), state))
}

fn functional<'a>(pseudo: &'a str, name: &str) -> Option<&'a str> {
    pseudo
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')
        .map(str::trim)
}

fn split_top(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = None;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if ch.is_whitespace() && depth == 0 {
            if let Some(from) = start.take() {
                parts.push(&text[from..index]);
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(from) = start {
        parts.push(&text[from..]);
    }
    parts
}

fn split_pseudos(part: &str) -> (&str, Vec<String>) {
    let mut depth = 0;
    let mut cuts = Vec::new();
    for (index, ch) in part.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ':' if depth == 0 => cuts.push(index),
            _ => {}
        }
    }
    let Some(first) = cuts.first().copied() else {
        return (part, Vec::new());
    };
    let mut pseudos = Vec::new();
    for (at, cut) in cuts.iter().enumerate() {
        let end = cuts.get(at + 1).copied().unwrap_or(part.len());
        pseudos.push(part[cut + 1..end].to_owned());
    }
    (&part[..first], pseudos)
}

fn classes(base: &str) -> Vec<&str> {
    match base.strip_prefix('.') {
        Some(rest) => rest.split('.').collect(),
        None => vec![base],
    }
}

fn properties(
    declarations: &[(String, String)],
    vars: &HashMap<String, String>,
) -> Result<Map<String, Value>, String> {
    let mut props = Map::new();
    let mut scoped = Map::new();
    for (name, raw) in declarations {
        if let Some(custom) = name.strip_prefix("--") {
            scoped.insert(custom.to_owned(), Value::String(raw.trim().to_owned()));
            continue;
        }
        let colour = matches!(
            name.as_str(),
            "background" | "background-color" | "color" | "border-color"
        );
        let gradient = matches!(name.as_str(), "background" | "background-image")
            && raw.trim_start().starts_with("linear-gradient(");
        let value = if (colour || gradient) && raw.contains("var(--") {
            raw.trim().to_owned()
        } else {
            calc(&substitute(raw, vars)?)?
        };
        if name == "transition" {
            for word in split_words(&value.replace(',', " ")) {
                let timed = word.ends_with("ms")
                    || word.ends_with('s') && word.trim_end_matches('s').parse::<f32>().is_ok();
                if timed && !props.contains_key("transition") {
                    props.insert("transition".to_owned(), Value::String(word));
                } else if matches!(
                    word.as_str(),
                    "out" | "ease-out" | "ease" | "ease-in-out" | "linear" | "token"
                ) {
                    let easing = if word == "linear" {
                        "linear"
                    } else if word == "token" {
                        "token"
                    } else {
                        "out"
                    };
                    props.insert("easing".to_owned(), Value::String(easing.to_owned()));
                } else if matches!(word.as_str(), "fast" | "normal" | "slow") {
                    props.insert("transition".to_owned(), Value::String(word));
                }
            }
            continue;
        }
        if matches!(name.as_str(), "background" | "background-image")
            && value.starts_with("linear-gradient(")
        {
            props.insert("gradient".to_owned(), Value::String(value));
            continue;
        }
        if matches!(name.as_str(), "backdrop-filter" | "-webkit-backdrop-filter") {
            let blur = value
                .split_once("blur(")
                .and_then(|(_, rest)| rest.split_once(')'))
                .map(|(inner, _)| inner.trim().trim_end_matches("px"))
                .and_then(|inner| inner.parse::<f64>().ok())
                .unwrap_or(0.0);
            props.insert("backdrop".to_owned(), serde_json::json!(blur));
            continue;
        }
        if name == "font-weight" {
            let weight = match value.trim() {
                "bold" => 700.0,
                "normal" => 400.0,
                other => other.parse::<f64>().unwrap_or(400.0),
            };
            props.insert("weight".to_owned(), serde_json::json!(weight));
            continue;
        }
        if matches!(
            name.as_str(),
            "stretch" | "squash" | "magnify" | "magnify-span"
        ) {
            let key = if name == "magnify-span" {
                "reach"
            } else {
                name.as_str()
            };
            let number = value.trim().parse::<f64>().unwrap_or(0.0);
            props.insert(key.to_owned(), serde_json::json!(number));
            continue;
        }
        if matches!(name.as_str(), "flex-grow" | "order") {
            let key = if name == "flex-grow" { "grow" } else { "order" };
            let number = value.trim().parse::<f64>().unwrap_or(0.0);
            props.insert(key.to_owned(), serde_json::json!(number));
            continue;
        }
        if name == "aspect-ratio" {
            let ratio = match value.split_once('/') {
                Some((width, height)) => {
                    let width = width.trim().parse::<f64>().unwrap_or(0.0);
                    let height = height.trim().parse::<f64>().unwrap_or(0.0);
                    if height > 0.0 { width / height } else { 0.0 }
                }
                None => value.trim().parse::<f64>().unwrap_or(0.0),
            };
            props.insert("aspect".to_owned(), serde_json::json!(ratio));
            continue;
        }
        if name == "padding" {
            let words = split_words(&value);
            if words.len() > 1 {
                let side = |index: usize| {
                    let word = &words[index];
                    match word.trim_end_matches("px").parse::<f64>() {
                        Ok(px) => serde_json::json!(px),
                        Err(_) => Value::String(word.clone()),
                    }
                };
                let sides = match words.len() {
                    2 => [side(0), side(1), side(0), side(1)],
                    3 => [side(0), side(1), side(2), side(1)],
                    _ => [side(0), side(1), side(2), side(3)],
                };
                props.insert("padding_box".to_owned(), Value::Array(sides.to_vec()));
                continue;
            }
        }
        if name == "margin" {
            let first = split_words(&value).into_iter().next().unwrap_or_default();
            let number = first.trim_end_matches("px").parse::<f64>().unwrap_or(0.0);
            props.insert("margin".to_owned(), serde_json::json!(number));
            continue;
        }
        if name == "border" {
            for word in split_words(&value) {
                if let Ok(width) = word.trim_end_matches("px").parse::<f64>() {
                    props.insert("stroke".to_owned(), serde_json::json!(width));
                } else if !matches!(word.as_str(), "solid" | "none") {
                    props.insert("border".to_owned(), Value::String(word));
                }
            }
            continue;
        }
        let name = &property_name(name).to_owned();
        let value = property_value(name, value);
        let number = value.trim_end_matches("px").parse::<f64>().ok();
        let parsed = match (name.as_str(), number) {
            ("animation", _) | (_, None) => Value::String(value),
            (_, Some(number)) => serde_json::json!(number),
        };
        props.insert(name.clone(), parsed);
    }
    if !scoped.is_empty() {
        props.insert("vars".to_owned(), Value::Object(scoped));
    }
    Ok(props)
}

fn split_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for ch in value.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn calc(value: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("calc(") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 5..];
        let mut depth = 1;
        let mut end = None;
        for (index, ch) in after.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.ok_or_else(|| format!("{value}: unclosed calc("))?;
        let result = evaluate(&calc(&after[..end])?)
            .ok_or_else(|| format!("{value}: calc() supports + - * / on numbers and px values"))?;
        out.push_str(&format!("{result}"));
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn evaluate(expression: &str) -> Option<f64> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    for ch in expression.chars() {
        match ch {
            '(' | ')' | '*' | '/' | '+' => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_owned());
                }
                current.clear();
                tokens.push(ch.to_string());
            }
            '-' => {
                let after_value = tokens.last().is_some_and(|last| {
                    last == ")" || last.trim_end_matches("px").parse::<f64>().is_ok()
                });
                if current.trim().is_empty() && !after_value {
                    current.push('-');
                } else {
                    if !current.trim().is_empty() {
                        tokens.push(current.trim().to_owned());
                    }
                    current.clear();
                    tokens.push("-".to_owned());
                }
            }
            c if c.is_whitespace() => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_owned());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_owned());
    }
    let mut position = 0;
    let value = sum(&tokens, &mut position)?;
    (position == tokens.len()).then_some(value)
}

fn sum(tokens: &[String], at: &mut usize) -> Option<f64> {
    let mut total = product(tokens, at)?;
    while let Some(op) = tokens.get(*at).filter(|t| matches!(t.as_str(), "+" | "-")) {
        let plus = op == "+";
        *at += 1;
        let next = product(tokens, at)?;
        total = if plus { total + next } else { total - next };
    }
    Some(total)
}

fn product(tokens: &[String], at: &mut usize) -> Option<f64> {
    let mut total = atom(tokens, at)?;
    while let Some(op) = tokens.get(*at).filter(|t| matches!(t.as_str(), "*" | "/")) {
        let times = op == "*";
        *at += 1;
        let next = atom(tokens, at)?;
        total = if times { total * next } else { total / next };
    }
    Some(total)
}

fn atom(tokens: &[String], at: &mut usize) -> Option<f64> {
    let token = tokens.get(*at)?;
    *at += 1;
    if token == "(" {
        let inner = sum(tokens, at)?;
        (tokens.get(*at)? == ")").then_some(())?;
        *at += 1;
        return Some(inner);
    }
    token.trim_end_matches("px").parse().ok()
}

fn substitute(value: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("var(--") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 6..];
        let end = after
            .find(')')
            .ok_or_else(|| format!("{value}: unclosed var("))?;
        let name = &after[..end];
        out.push_str(
            vars.get(name)
                .ok_or_else(|| format!("unknown variable --{name}"))?,
        );
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out.trim().to_owned())
}

fn declarations(body: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for part in body
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let (name, value) = part
            .split_once(':')
            .ok_or_else(|| format!("expected `name: value`, found `{part}`"))?;
        found.push((name.trim().to_owned(), value.trim().to_owned()));
    }
    Ok(found)
}

fn blocks(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    let mut rest = text;
    while !rest.trim().is_empty() {
        let open = rest.find('{').ok_or("expected `{`")?;
        let prelude = rest[..open].trim().to_owned();
        let mut depth = 0;
        let mut close = None;
        for (index, ch) in rest[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(open + index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close.ok_or_else(|| format!("{prelude}: missing `}}`"))?;
        found.push((prelude, rest[open + 1..close].to_owned()));
        rest = &rest[close + 1..];
    }
    Ok(found)
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("*/") {
            Some(end) => &rest[start + end + 2..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}
