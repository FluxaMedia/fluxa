use serde_json::{Map, Value, json};

use crate::layout::Node;

#[derive(Clone, Debug)]
pub struct Element {
    pub parent: String,
    pub start: bool,
    pub node: Node,
}

#[derive(Clone, Debug)]
pub struct PageDef {
    pub name: String,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub node: Node,
}

#[derive(Clone, Debug, Default)]
pub struct Blocks {
    pub css: String,
    pub elements: Vec<Element>,
    pub pages: Vec<PageDef>,
    pub meta: Vec<(String, String)>,
}

pub fn extract(css: &str) -> Result<Blocks, Vec<String>> {
    let mut out = Blocks::default();
    let mut problems = Vec::new();
    let mut rest = css;
    while let Some(at) = next_at(rest) {
        out.css.push_str(&rest[..at]);
        let tail = &rest[at..];
        let Some(open) = tail.find('{') else {
            out.css.push_str(tail);
            rest = "";
            break;
        };
        let head = tail[..open].trim();
        let Some(close) = matching(tail, open) else {
            problems.push(format!("{head}: missing closing brace"));
            rest = "";
            break;
        };
        let body = &tail[open + 1..close];
        let mut words = head.split_whitespace();
        let kind = words.next().unwrap_or_default();
        let name = words.next().unwrap_or_default();
        match kind {
            "@element" => match element(name, body) {
                Ok(element) => out.elements.push(element),
                Err(error) => problems.push(format!("@element {name}: {error}")),
            },
            "@page" => match page(name, body) {
                Ok(page) => out.pages.push(page),
                Err(error) => problems.push(format!("@page {name}: {error}")),
            },
            "@skin" => out.meta.extend(declarations(body)),
            _ => out.css.push_str(&tail[..=close]),
        }
        rest = &tail[close + 1..];
    }
    out.css.push_str(rest);
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems)
    }
}

fn next_at(text: &str) -> Option<usize> {
    let mut search = 0;
    while let Some(found) = text[search..].find('@') {
        let at = search + found;
        let tail = &text[at..];
        if ["@element", "@page", "@skin"]
            .iter()
            .any(|kind| tail.starts_with(kind))
        {
            return Some(at);
        }
        let open = tail.find('{')?;
        search = at + matching(tail, open).map_or(tail.len(), |close| close + 1);
    }
    None
}

fn matching(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0;
    let mut quote = None;
    for (index, c) in text.char_indices().skip_while(|(index, _)| *index < open) {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '{') => depth += 1,
            (None, '}') => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn declarations(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut depth = 0;
    let mut flush = |current: &mut String| {
        if let Some((name, value)) = current.split_once(':') {
            out.push((name.trim().to_owned(), value.trim().to_owned()));
        }
        current.clear();
    };
    for c in body.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, '(') => depth += 1,
            (None, ')') => depth -= 1,
            (None, ';') if depth == 0 => {
                flush(&mut current);
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    flush(&mut current);
    out
}

pub(crate) fn unquote(value: &str) -> String {
    let value = value.trim();
    for q in ['"', '\''] {
        if let Some(inner) = value.strip_prefix(q).and_then(|v| v.strip_suffix(q)) {
            return inner.to_owned();
        }
    }
    value.to_owned()
}

fn bound(value: &str) -> Value {
    let value = value.trim();
    if value.starts_with('"') || value.starts_with('\'') {
        return json!(unquote(value));
    }
    if value.contains('.') && !value.contains(' ') && value.parse::<f64>().is_err() {
        return json!(format!("{{{value}}}"));
    }
    json!(value)
}

fn action(value: &str) -> Result<Value, String> {
    let value = value.trim();
    let Some(open) = value.find('(') else {
        return Ok(json!({"name": value}));
    };
    let name = value[..open].trim();
    let inner = value[open + 1..]
        .strip_suffix(')')
        .ok_or("action is missing its closing parenthesis")?;
    let mut args = Map::new();
    let parts = split_top(inner);
    if let [single] = parts.as_slice()
        && !single.contains(':')
        && name == "page"
    {
        args.insert("name".to_owned(), bound(single));
    } else {
        for part in parts {
            let (key, value) = part
                .split_once(':')
                .ok_or_else(|| format!("argument {part:?} needs a name: value"))?;
            args.insert(key.trim().to_owned(), bound(value));
        }
    }
    Ok(json!({"name": name, "args": args}))
}

fn split_top(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for c in text.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, ',') => {
                out.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out.into_iter().map(|part| part.trim().to_owned()).collect()
}

fn node(name: &str, fields: &[(String, String)]) -> Result<(Node, Option<String>, bool), String> {
    let mut map = Map::new();
    let mut parent = None;
    let mut start = false;
    map.insert("type".to_owned(), json!("column"));
    map.insert("class".to_owned(), json!(name));
    for (key, value) in fields {
        match key.as_str() {
            "parent" => parent = Some(value.trim().trim_start_matches('.').to_owned()),
            "position" => start = value.trim() == "start",
            "type" => {
                map.insert("type".to_owned(), json!(value.trim()));
            }
            "text" | "src" | "style" | "when" | "each" | "i18n" => {
                let value = if key == "text" || key == "src" || key == "i18n" {
                    json!(unquote(value))
                } else {
                    json!(value.trim())
                };
                map.insert(key.clone(), value);
            }
            "as" => {
                map.insert("as".to_owned(), json!(value.trim()));
            }
            "action" => {
                map.insert("action".to_owned(), action(value)?);
            }
            "class" => {
                map.insert(
                    "class".to_owned(),
                    json!(format!("{name} {}", unquote(value))),
                );
            }
            other => return Err(format!("unknown field {other:?}")),
        }
    }
    let node = serde_json::from_value(Value::Object(map)).map_err(|error| error.to_string())?;
    Ok((node, parent, start))
}

fn element(name: &str, body: &str) -> Result<Element, String> {
    if name.is_empty() {
        return Err("needs a name".to_owned());
    }
    let (node, parent, start) = node(name, &declarations(body))?;
    let parent = parent.ok_or("needs a parent")?;
    Ok(Element {
        parent,
        start,
        node,
    })
}

fn page(name: &str, body: &str) -> Result<PageDef, String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err("name uses lowercase letters, digits, - and _".to_owned());
    }
    let mut label = None;
    let mut icon = None;
    let mut rest = Vec::new();
    for (key, value) in declarations(body) {
        match key.as_str() {
            "nav-label" => label = Some(unquote(&value)),
            "nav-icon" => icon = Some(value.trim().to_owned()),
            _ => rest.push((key, value)),
        }
    }
    let (node, ..) = node(&format!("page-{name}"), &rest)?;
    Ok(PageDef {
        name: name.to_owned(),
        label,
        icon,
        node,
    })
}

pub(crate) fn graft(root: &mut Node, element: &Element) -> bool {
    let mut found = false;
    if root
        .class
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|c| c == element.parent))
    {
        if element.start {
            root.children.insert(0, element.node.clone());
        } else {
            root.children.push(element.node.clone());
        }
        found = true;
    }
    for child in &mut root.children {
        found |= graft(child, element);
    }
    if let Some(template) = root.template.as_mut() {
        found |= graft(template, element);
    }
    found
}

pub fn files_from_css(
    css: &str,
    source: Option<&str>,
    fetch: &dyn Fn(&str) -> Result<String, String>,
) -> Result<(String, Vec<(String, String)>), Vec<String>> {
    let blocks = extract(css)?;
    let meta = |key: &str| {
        blocks
            .meta
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| unquote(value))
    };
    let id = meta("id").ok_or_else(|| vec!["@skin needs an id".to_owned()])?;
    let mut manifest = json!({
        "schemaVersion": 1,
        "id": id,
        "name": meta("name").unwrap_or_else(|| id.clone()),
        "version": meta("version").unwrap_or_else(|| "1.0.0".to_owned()),
        "theme": "style.css",
    });
    if let Some(author) = meta("author") {
        manifest["author"] = json!(author);
    }
    let mut files = vec![("style.css".to_owned(), css.to_owned())];
    if let Some(url) = meta("script") {
        let code = fetch(&url).map_err(|error| vec![format!("script {url}: {error}")])?;
        let list = |key: &str| -> Vec<String> {
            meta(key)
                .unwrap_or_default()
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|part| !part.is_empty())
                .map(str::to_owned)
                .collect()
        };
        let mut extension = json!({
            "schemaVersion": 1,
            "id": meta("script-id").unwrap_or_else(|| "ext".to_owned()),
            "name": meta("name").unwrap_or_else(|| id.clone()),
            "version": meta("version").unwrap_or_else(|| "1.0.0".to_owned()),
            "kind": "script",
            "entry": "script.js",
            "permissions": list("permissions"),
            "networkHosts": list("hosts"),
        });
        if let Some(seconds) = meta("refresh").and_then(|value| value.parse::<u64>().ok()) {
            extension["refreshSeconds"] = json!(seconds);
        }
        let events = list("events");
        if !events.is_empty() {
            extension["contributes"] = json!({ "events": events });
        }
        manifest["extensions"] = json!(["ext/extension.json"]);
        files.push(("ext/extension.json".to_owned(), extension.to_string()));
        files.push(("ext/script.js".to_owned(), code));
    }
    files.push(("manifest.json".to_owned(), manifest.to_string()));
    if let Some(source) = source {
        files.push(("remote.json".to_owned(), json!({"url": source}).to_string()));
    }
    Ok((id, files))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements_pages_and_plain_css_are_separated() {
        let blocks = extract(
            r#".a { gap: 4px; }
            @element agenda-btn {
                parent: .navbar; position: start; type: row;
                action: page(agenda);
            }
            @page agenda { nav-label: "Agenda"; }
            @element agenda-row {
                parent: .page-agenda; each: ext.agenda.items; as: item;
                text: "{item.label}";
                action: ext.agenda.toggle(index: item.index);
            }
            @media (form: tv) { .a { gap: 8px; } }"#,
        )
        .unwrap();
        assert_eq!(blocks.elements.len(), 2);
        assert_eq!(blocks.pages[0].label.as_deref(), Some("Agenda"));
        assert!(blocks.css.contains(".a { gap: 4px; }"));
        assert!(blocks.css.contains("@media (form: tv)"));
        assert!(!blocks.css.contains("@element"));
        let toggle = blocks.elements[1].node.action.as_ref().unwrap();
        assert_eq!(toggle.name, "ext.agenda.toggle");
        assert_eq!(toggle.args["index"], "{item.index}");
        let open = blocks.elements[0].node.action.as_ref().unwrap();
        assert_eq!(open.args["name"], "agenda");
    }

    #[test]
    fn unknown_field_names_the_block() {
        let problems = extract("@element x { parent: .a; colour: red; }").unwrap_err();
        assert!(problems[0].contains("@element x") && problems[0].contains("colour"));
    }

    #[test]
    fn single_css_file_becomes_a_loadable_skin_with_its_script() {
        let css = r#"@skin { id: solo; name: "Solo"; script: "https://example.com/a.js";
            hosts: example.com; permissions: network; refresh: 60; }
            .x { gap: 4px; }"#;
        let (id, files) = files_from_css(css, Some("https://example.com/solo.css"), &|url| {
            assert_eq!(url, "https://example.com/a.js");
            Ok("class Provider {}".to_owned())
        })
        .unwrap();
        assert_eq!(id, "solo");
        let borrowed: Vec<_> = files
            .iter()
            .map(|(name, text)| (name.as_str(), text.as_str()))
            .collect();
        let skin = crate::load_embedded(&borrowed).unwrap();
        assert_eq!(skin.scripts()[0].extension.network_hosts, ["example.com"]);
        assert_eq!(skin.manifest.name, "Solo");
    }
}
