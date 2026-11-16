use crate::expr::{Expr, Scope, Template, truthy};
use crate::layout::{Align, FormFactor, Justify, MAX_DEPTH, Node, Size};
use serde_json::Value;
use std::collections::BTreeMap;

const MAX_NODES: usize = 4000;
const DEFAULT_LIST_LIMIT: usize = 200;

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
    pub padding: Option<f32>,
    pub align: Option<Align>,
    pub justify: Option<Justify>,
    pub children: Vec<Resolved>,
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

pub fn resolve(
    root: &Node,
    includes: &BTreeMap<String, Node>,
    data: &Value,
    form: FormFactor,
) -> Result<Vec<Resolved>, ResolveError> {
    let mut resolver = Resolver {
        includes,
        form,
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
        let Value::Array(items) = self.eval(source, key)? else {
            return Ok(());
        };
        let alias = node.item.as_deref().unwrap_or("item");
        let limit = node.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        for (index, item) in items.into_iter().take(limit).enumerate() {
            self.scope.push(alias, item);
            self.scope.push("index", Value::from(index));
            let result = self.expand(template, &format!("{key}.{index}"), depth + 1, out);
            self.scope.pop();
            self.scope.pop();
            result?;
        }
        Ok(())
    }

    fn build(&mut self, node: &Node, key: &str, depth: usize) -> Result<Resolved, ResolveError> {
        let mut children = Vec::new();
        for (index, child) in node.children.iter().enumerate() {
            self.expand(child, &format!("{key}.{index}"), depth + 1, &mut children)?;
        }
        let render = |source: &Option<String>, scope: &Scope| {
            source
                .as_deref()
                .and_then(|source| Template::parse(source).ok())
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
        Ok(Resolved {
            kind: node.kind.clone(),
            key: key.to_owned(),
            style: render(&node.style, &self.scope),
            class: render(&node.class, &self.scope),
            i18n: node.i18n.clone(),
            text: render(&node.text, &self.scope),
            src: render(&node.src, &self.scope),
            props,
            action,
            width: node.width.as_ref().and_then(Size::px),
            height: node.height.as_ref().and_then(Size::px),
            width_vw: node.width.as_ref().and_then(|size| size.view("vw")),
            height_vh: node.height.as_ref().and_then(|size| size.view("vh")),
            grow: node.grow,
            fill: node.fill,
            gap: node.gap,
            padding: node.padding,
            align: node.align,
            justify: node.justify,
            children,
        })
    }

    fn eval(&self, source: &str, key: &str) -> Result<Value, ResolveError> {
        Expr::parse(source)
            .map(|expr| expr.eval(&self.scope))
            .map_err(|error| ResolveError(format!("{key}: {error}")))
    }

    fn value(&self, value: &Value) -> Value {
        match value {
            Value::String(source) => Template::parse(source)
                .map(|template| template.value(&self.scope))
                .unwrap_or_else(|_| value.clone()),
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
        resolve(&root, &includes, &data, form).unwrap()
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
        assert!(resolve(&root, &includes, &json!({}), FormFactor::Desktop).is_err());
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
        );
        assert!(result.is_err());
    }
}
