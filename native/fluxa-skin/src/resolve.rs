use crate::expr::{Expr, Scope, Template, truthy};
use crate::layout::{Align, FormFactor, Justify, MAX_DEPTH, Node, Pad, Size};
use serde_json::Value;
use std::collections::BTreeMap;

const MAX_NODES: usize = 12000;
const DEFAULT_LIST_LIMIT: usize = 200;
const DEFAULT_WINDOW: (usize, usize) = (0, 80);

pub type Windows = BTreeMap<String, (usize, usize)>;

struct CachedItem {
    signature: u64,
    reads: crate::expr::Reads,
    nodes: Vec<Resolved>,
}

#[derive(Default)]
pub struct ResolveCache {
    pub windows: Windows,
    pub language: String,
    pub translate: Option<fn(&str, &str) -> String>,
    items: BTreeMap<String, CachedItem>,
}

impl ResolveCache {
    pub const fn new() -> Self {
        Self {
            windows: BTreeMap::new(),
            language: String::new(),
            translate: None,
            items: BTreeMap::new(),
        }
    }

    pub fn clear_items(&mut self) {
        self.items.clear();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    pub kind: String,
    pub key: String,
    pub style: Option<String>,
    pub class: Option<String>,
    pub i18n: Option<String>,
    pub text: Option<String>,
    pub src: Option<String>,
    pub props: BTreeMap<String, Value>,
    pub action: Option<ResolvedAction>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub width_vw: Option<f32>,
    pub height_vh: Option<f32>,
    pub grow: Option<f32>,
    pub fill: bool,
    pub gap: Option<f32>,
    pub padding: Option<[f32; 4]>,
    pub align: Option<Align>,
    pub justify: Option<Justify>,
    pub children: std::sync::Arc<Vec<Resolved>>,
}

impl Resolved {
    pub fn far(key: String) -> Self {
        Self {
            kind: "far".to_owned(),
            key,
            style: None,
            class: None,
            i18n: None,
            text: None,
            src: None,
            props: BTreeMap::new(),
            action: None,
            width: None,
            height: None,
            width_vw: None,
            height_vh: None,
            grow: None,
            fill: false,
            gap: None,
            padding: None,
            align: None,
            justify: None,
            children: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedAction {
    pub name: String,
    pub args: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolveError(pub String);

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn window_read(owner: &str, initial: Option<usize>) -> String {
    match initial {
        Some(initial) => format!("@window|{initial}|{owner}"),
        None => format!("@window|-|{owner}"),
    }
}

fn window_hash(window: (usize, usize)) -> u64 {
    (window.0 as u64) << 32 ^ window.1 as u64
}

pub fn resolve(
    root: &Node,
    includes: &BTreeMap<String, Node>,
    data: &Value,
    form: FormFactor,
    cache: &mut ResolveCache,
) -> Result<Vec<Resolved>, ResolveError> {
    let mut resolver = Resolver {
        includes,
        form,
        cache,
        scope: Scope::new(data),
        nodes: 0,
    };
    let mut out = Vec::new();
    resolver.expand(root, "0", 0, &mut out)?;
    Ok(out)
}

struct Resolver<'a> {
    includes: &'a BTreeMap<String, Node>,
    form: FormFactor,
    cache: &'a mut ResolveCache,
    scope: Scope<'a>,
    nodes: usize,
}

impl Resolver<'_> {
    fn expand(
        &mut self,
        node: &Node,
        key: &str,
        depth: usize,
        out: &mut Vec<Resolved>,
    ) -> Result<(), ResolveError> {
        if depth > MAX_DEPTH {
            return Err(ResolveError(format!("{key}: nested too deeply")));
        }
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err(ResolveError(format!("more than {MAX_NODES} nodes")));
        }
        if !node.form.is_empty() && !node.form.contains(&self.form) {
            return Ok(());
        }
        if let Some(source) = &node.when
            && !truthy(&self.eval(source, key)?)
        {
            return Ok(());
        }
        if let Some(source) = &node.each {
            return self.expand_each(node, source, key, depth, out);
        }
        if let Some(name) = &node.include {
            let target = self
                .includes
                .get(name)
                .ok_or_else(|| ResolveError(format!("{key}: unknown include {name:?}")))?;
            let mut bound = Vec::new();
            for (param, source) in &node.with {
                let value = self.eval(source, key)?;
                self.scope.push(param, value);
                bound.push(param);
            }
            let result = self.expand(target, &format!("{key}.{name}"), depth + 1, out);
            for _ in bound {
                self.scope.pop();
            }
            return result;
        }
        let resolved = self.build(node, key, depth)?;
        out.push(resolved);
        Ok(())
    }

    fn expand_each(
        &mut self,
        node: &Node,
        source: &str,
        key: &str,
        depth: usize,
        out: &mut Vec<Resolved>,
    ) -> Result<(), ResolveError> {
        let template = node
            .template
            .as_deref()
            .ok_or_else(|| ResolveError(format!("{key}: each needs a template")))?;
        let expr = Expr::parse(source).map_err(|error| ResolveError(format!("{key}: {error}")))?;
        let owned;
        let items: &[Value] = match expr.borrow(&self.scope) {
            Some(Value::Array(items)) => items,
            Some(_) => return Ok(()),
            None => {
                owned = expr.eval(&self.scope);
                match &owned {
                    Value::Array(items) => items,
                    _ => return Ok(()),
                }
            }
        };
        let alias = node.item.as_deref().unwrap_or("item");
        let limit = node.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let owner = key.rsplit_once('.').map_or(key, |(owner, _)| owner);
        let window = node.window.then(|| {
            let window = self.window_of(owner, node.initial);
            self.scope.note(window_read(owner, node.initial), window_hash(window));
            window
        });
        let first = out.len();
        for (index, item) in items.iter().take(limit).enumerate() {
            let cached = window.is_some();
            let mut signature = 0;
            if let Some((start, end)) = window {
                if !(start..end).contains(&index) {
                    let mut far = Resolved::far(format!("{key}.{index}"));
                    if node.flow {
                        far.props.insert(
                            "windowed".to_owned(),
                            Value::String(far.key.clone()),
                        );
                    }
                    out.push(far);
                    continue;
                }
                signature = self.scope.signature(item);
                if let Some(hit) = self.cache.items.get(&format!("{key}.{index}"))
                    && hit.signature == signature
                    && self.reads_unchanged(&hit.reads)
                {
                    out.extend(hit.nodes.iter().cloned());
                    continue;
                }
            }
            self.scope.push(alias, item.clone());
            self.scope.push("index", Value::from(index));
            let from = out.len();
            self.scope.begin_reads();
            let result = self.expand(template, &format!("{key}.{index}"), depth + 1, out);
            let reads = self.scope.end_reads();
            self.scope.pop();
            self.scope.pop();
            result?;
            if cached {
                if node.flow {
                    for node in &mut out[from..] {
                        node.props.insert(
                            "windowed".to_owned(),
                            Value::String(format!("{key}.{index}")),
                        );
                    }
                }
                self.cache.items.insert(
                    format!("{key}.{index}"),
                    CachedItem {
                        signature,
                        reads,
                        nodes: out[from..].to_vec(),
                    },
                );
            }
        }
        if window.is_some() {
            let shape = out[first..].iter().find(|node| node.kind != "far").cloned();
            if let Some(shape) = shape {
                for node in out[first..].iter_mut().filter(|node| node.kind == "far") {
                    node.class = shape.class.clone();
                    node.style = shape.style.clone();
                    node.width = shape.width;
                    node.height = shape.height;
                    node.width_vw = shape.width_vw;
                    node.height_vh = shape.height_vh;
                    node.grow = shape.grow;
                    node.fill = shape.fill;
                }
            }
        }
        Ok(())
    }

    fn window_of(&self, owner: &str, initial: Option<usize>) -> (usize, usize) {
        self.cache.windows.get(owner).copied().unwrap_or((
            0,
            initial.unwrap_or(DEFAULT_WINDOW.1),
        ))
    }

    fn reads_unchanged(&self, reads: &crate::expr::Reads) -> bool {
        reads.iter().all(|(name, hash)| match name.strip_prefix("@window|") {
            Some(spec) => {
                let mut parts = spec.splitn(2, '|');
                let initial = parts.next().and_then(|part| part.parse::<usize>().ok());
                let owner = parts.next().unwrap_or_default();
                window_hash(self.window_of(owner, initial)) == *hash
            }
            None => true,
        }) && self.scope.reads_unchanged(reads)
    }

    fn build(&mut self, node: &Node, key: &str, depth: usize) -> Result<Resolved, ResolveError> {
        let mut children = Vec::new();
        for (index, child) in node.children.iter().enumerate() {
            self.expand(child, &format!("{key}.{index}"), depth + 1, &mut children)?;
        }
        let render = |source: &Option<String>, scope: &Scope| {
            source
                .as_deref()
                .and_then(Template::parse_shared)
                .map(|template| template.render(scope))
        };
        let props = node
            .props
            .iter()
            .map(|(name, value)| (name.clone(), self.value(value)))
            .collect();
        let action = node.action.as_ref().map(|action| ResolvedAction {
            name: action.name.clone(),
            args: action
                .args
                .iter()
                .map(|(name, value)| (name.clone(), self.value(value)))
                .collect(),
        });
        let width = self.size(node.width.as_ref());
        let height = self.size(node.height.as_ref());
        let i18n = render(&node.i18n, &self.scope);
        let text = match (&i18n, self.cache.translate) {
            (Some(key), Some(translate)) => Some(translate(key, &self.cache.language)),
            _ => render(&node.text, &self.scope),
        };
        Ok(Resolved {
            kind: node.kind.clone(),
            key: key.to_owned(),
            style: render(&node.style, &self.scope),
            class: render(&node.class, &self.scope),
            i18n,
            text,
            src: render(&node.src, &self.scope),
            props,
            action,
            width: width.as_ref().and_then(Size::px),
            height: height.as_ref().and_then(Size::px),
            width_vw: width.as_ref().and_then(|size| size.view("vw")),
            height_vh: height.as_ref().and_then(|size| size.view("vh")),
            grow: node.grow,
            fill: node.fill,
            gap: self.size(node.gap.as_ref()).and_then(|size| size.px()),
            padding: self.pad(node.padding.as_ref()),
            align: node.align,
            justify: node.justify,
            children: children.into(),
        })
    }

    fn pad(&self, pad: Option<&Pad>) -> Option<[f32; 4]> {
        match pad? {
            Pad::Text(text) if text.contains('{') => {
                let rendered = Template::parse_shared(text)?.render(&self.scope);
                Pad::Text(rendered).sides()
            }
            other => other.sides(),
        }
    }

    fn size(&self, size: Option<&Size>) -> Option<Size> {
        match size {
            Some(Size::Text(text)) if text.contains('{') => {
                let rendered = Template::parse_shared(text)?.render(&self.scope);
                Some(match rendered.trim().parse::<f32>() {
                    Ok(value) => Size::Px(value),
                    Err(_) => Size::Text(rendered),
                })
            }
            other => other.cloned(),
        }
    }

    fn eval(&self, source: &str, key: &str) -> Result<Value, ResolveError> {
        Expr::parse(source)
            .map(|expr| expr.eval(&self.scope))
            .map_err(|error| ResolveError(format!("{key}: {error}")))
    }

    fn value(&self, value: &Value) -> Value {
        match value {
            Value::String(source) => Template::parse_shared(source)
                .map(|template| template.value(&self.scope))
                .unwrap_or_else(|| value.clone()),
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(layout: Value, includes: Value, data: Value, form: FormFactor) -> Vec<Resolved> {
        let root: Node = serde_json::from_value(layout).unwrap();
        let includes: BTreeMap<String, Node> = serde_json::from_value(includes).unwrap();
        resolve(&root, &includes, &data, form, &mut ResolveCache::default()).unwrap()
    }

    #[test]
    fn lists_expand_with_alias_and_index() {
        let out = run(
            json!({"each": "rows", "as": "row", "template": {"type": "text", "text": "{index}:{row.title}"}}),
            json!({}),
            json!({"rows": [{"title": "a"}, {"title": "b"}]}),
            FormFactor::Desktop,
        );
        let texts: Vec<_> = out.iter().map(|n| n.text.clone().unwrap()).collect();
        assert_eq!(texts, ["0:a", "1:b"]);
        assert_ne!(out[0].key, out[1].key);
    }

    #[test]
    fn when_and_form_filter_nodes() {
        let layout = json!({"type": "column", "children": [
            {"type": "text", "text": "shown", "when": "flag"},
            {"type": "text", "text": "hidden", "when": "!flag"},
            {"type": "text", "text": "tv only", "form": ["tv"]},
            {"type": "text", "text": "not tv", "form": ["mobile", "desktop"]}
        ]});
        let out = run(layout, json!({}), json!({"flag": true}), FormFactor::Tv);
        let texts: Vec<_> = out[0]
            .children
            .iter()
            .map(|n| n.text.clone().unwrap())
            .collect();
        assert_eq!(texts, ["shown", "tv only"]);
    }

    #[test]
    fn includes_receive_parameters() {
        let out = run(
            json!({"include": "label", "with": {"name": "profile.name"}}),
            json!({"label": {"type": "text", "text": "Hi {name}"}}),
            json!({"profile": {"name": "Sam"}}),
            FormFactor::Mobile,
        );
        assert_eq!(out[0].text.as_deref(), Some("Hi Sam"));
    }

    #[test]
    fn action_arguments_keep_their_types() {
        let out = run(
            json!({"type": "button", "action": {"name": "set_setting", "args": {"key": "autoplay", "value": "{flag}"}}}),
            json!({}),
            json!({"flag": true}),
            FormFactor::Desktop,
        );
        let action = out[0].action.as_ref().unwrap();
        assert_eq!(action.args["value"], json!(true));
        assert_eq!(action.args["key"], json!("autoplay"));
    }

    #[test]
    fn recursive_include_fails_instead_of_looping() {
        let root: Node = serde_json::from_value(json!({"include": "a"})).unwrap();
        let includes: BTreeMap<String, Node> =
            serde_json::from_value(json!({"a": {"include": "a"}})).unwrap();
        assert!(
            resolve(
                &root,
                &includes,
                &json!({}),
                FormFactor::Desktop,
                &mut ResolveCache::default()
            )
            .is_err()
        );
    }

    #[test]
    fn list_limit_caps_expansion() {
        let rows: Vec<Value> = (0..500).map(|n| json!({"n": n})).collect();
        let out = run(
            json!({"each": "rows", "limit": 3, "template": {"type": "spacer"}}),
            json!({}),
            json!({"rows": rows}),
            FormFactor::Desktop,
        );
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn node_budget_stops_runaway_nesting_of_lists() {
        let rows: Vec<Value> = (0..200).map(|n| json!(n)).collect();
        let root: Node = serde_json::from_value(json!({
            "each": "rows", "template": {"each": "rows", "template": {"each": "rows", "template": {"type": "spacer"}}}
        }))
        .unwrap();
        let result = resolve(
            &root,
            &BTreeMap::new(),
            &json!({"rows": rows}),
            FormFactor::Desktop,
            &mut ResolveCache::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn windowed_lists_stub_items_outside_the_window() {
        let rows: Vec<Value> = (0..10).map(|n| json!(n)).collect();
        let root: Node = serde_json::from_value(json!({
            "each": "rows", "window": true, "limit": 100, "template": {"type": "text", "text": "{item}"}
        }))
        .unwrap();
        let mut cache = ResolveCache::default();
        cache.windows.insert("0".to_owned(), (2, 4));
        let out = resolve(
            &root,
            &BTreeMap::new(),
            &json!({"rows": rows}),
            FormFactor::Desktop,
            &mut cache,
        )
        .unwrap();
        let kinds: Vec<_> = out.iter().map(|node| node.kind.as_str()).collect();
        assert_eq!(out.len(), 10);
        assert_eq!(kinds.iter().filter(|kind| **kind == "text").count(), 2);
        assert_eq!(out[0].kind, "far");
        assert_eq!(out[2].text.as_deref(), Some("2"));
    }

    #[test]
    fn cached_items_follow_root_data_they_read() {
        let root: Node = serde_json::from_value(json!({
            "each": "rows", "window": true, "limit": 100,
            "template": {"type": "text", "text": "{item}/{total}"}
        }))
        .unwrap();
        let mut cache = ResolveCache::default();
        cache.windows.insert("0".to_owned(), (0, 3));
        let run = |cache: &mut ResolveCache, total: u32| {
            resolve(
                &root,
                &BTreeMap::new(),
                &json!({"rows": [1, 2, 3], "total": total}),
                FormFactor::Desktop,
                cache,
            )
            .unwrap()
        };
        assert_eq!(run(&mut cache, 3)[1].text.as_deref(), Some("2/3"));
        assert_eq!(run(&mut cache, 3)[1].text.as_deref(), Some("2/3"));
        assert_eq!(run(&mut cache, 6)[1].text.as_deref(), Some("2/6"));
    }

    #[test]
    fn nested_windows_keep_the_node_count_flat() {
        let rows: Vec<Value> = (0..600)
            .map(|row| json!({"cards": (0..30).map(|card| json!(format!("{row}-{card}"))).collect::<Vec<_>>()}))
            .collect();
        let root: Node = serde_json::from_value(json!({
            "each": "rows", "as": "row", "limit": 100000, "window": true, "initial": 4, "flow": true,
            "template": {"type": "column", "children": [{
                "each": "row.cards", "window": true, "initial": 5, "as": "card",
                "template": {"type": "text", "text": "{card}"}
            }]}
        }))
        .unwrap();
        let mut cache = ResolveCache::default();
        let out = resolve(
            &root,
            &BTreeMap::new(),
            &json!({"rows": rows}),
            FormFactor::Desktop,
            &mut cache,
        )
        .unwrap();
        assert_eq!(out.len(), 600);
        assert_eq!(out.iter().filter(|node| node.kind == "far").count(), 596);
        let strip = &out[0].children;
        assert_eq!(strip.iter().filter(|node| node.kind == "text").count(), 5);
        assert_eq!(strip.len(), 30);
        assert_eq!(strip[7].kind, "far");

        cache.windows.insert("0.0".to_owned(), (2, 6));
        let out = resolve(
            &root,
            &BTreeMap::new(),
            &json!({"rows": (0..600).map(|row| json!({"cards": (0..30).map(|card| json!(format!("{row}-{card}"))).collect::<Vec<_>>()})).collect::<Vec<_>>()}),
            FormFactor::Desktop,
            &mut cache,
        )
        .unwrap();
        assert_eq!(out[0].children[0].kind, "far");
        assert_eq!(out[0].children[5].kind, "text");
        assert_eq!(out[0].children[6].kind, "far");
    }
}
