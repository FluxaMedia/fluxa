use serde_json::{Map, Value};
use std::collections::HashMap;

const STATES: [&str; 5] = ["hover", "pressed", "focused", "selected", "disabled"];

pub(super) fn to_value(text: &str) -> Result<Value, String> {
    let text = strip_comments(text);
    let mut vars = HashMap::new();
    let blocks = blocks(&text)?;
    for (prelude, body) in &blocks {
        if prelude.trim() == ":root" {
            for (name, value) in declarations(body)? {
                if let Some(name) = name.strip_prefix("--") {
                    vars.insert(name.to_owned(), value);
                }
            }
        }
    }
    let mut root = Map::new();
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
            let key = format!("@{}", media_key(query)?);
            let mut rules = Map::new();
            for (selectors, rule) in self::blocks(body)? {
                add_rule(&mut rules, &selectors, &rule, &vars)?;
            }
            root.insert(key, Value::Object(rules));
        } else {
            add_rule(&mut root, prelude, body, &vars)?;
        }
    }
    Ok(Value::Object(root))
}

fn media_key(query: &str) -> Result<String, String> {
    let query = query.trim();
    let Some(inner) = query.strip_prefix("media") else {
        return Ok(query.to_owned());
    };
    let inner = inner.trim().trim_start_matches('(').trim_end_matches(')');
    let (feature, value) = inner
        .split_once(':')
        .ok_or_else(|| format!("@media {inner}: expected (feature: value)"))?;
    let value = value.trim().trim_end_matches("px");
    match feature.trim() {
        "min-width" | "max-width" => Ok(format!("{}:{}", feature.trim(), value)),
        "platform" => Ok(value.to_owned()),
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
        let selector = selector.trim();
        let (key, state) = match selector.rsplit_once(':') {
            Some((key, state)) if STATES.contains(&state) => (key, Some(state)),
            _ => (selector, None),
        };
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
            target.insert(name.clone(), value.clone());
        }
    }
    Ok(())
}

fn properties(
    declarations: &[(String, String)],
    vars: &HashMap<String, String>,
) -> Result<Map<String, Value>, String> {
    let mut props = Map::new();
    for (name, raw) in declarations {
        if name.starts_with("--") {
            continue;
        }
        let value = substitute(raw, vars)?;
        if name == "transition" {
            let mut words = value.split_whitespace();
            if let Some(first) = words.next() {
                props.insert("transition".to_owned(), Value::String(first.to_owned()));
            }
            if let Some(easing) = words.next() {
                props.insert("easing".to_owned(), Value::String(easing.to_owned()));
            }
            continue;
        }
        let number = value.trim_end_matches("px").parse::<f64>().ok();
        let parsed = match (name.as_str(), number) {
            ("animation", _) | (_, None) => Value::String(value),
            (_, Some(number)) => serde_json::json!(number),
        };
        props.insert(name.clone(), parsed);
    }
    Ok(props)
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
