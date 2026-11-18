use crate::contract::{contract, extension_action};
use crate::expr::{Expr, Template};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub const MAX_DEPTH: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormFactor {
    Mobile,
    Desktop,
    Tv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub name: String,
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Node {
    #[serde(rename = "type")]
    pub kind: String,
    pub when: Option<String>,
    pub form: Vec<FormFactor>,
    pub include: Option<String>,
    pub with: BTreeMap<String, String>,
    pub each: Option<String>,
    #[serde(rename = "as")]
    pub item: Option<String>,
    pub limit: Option<usize>,
    pub window: bool,
    pub initial: Option<usize>,
    pub flow: bool,
    pub template: Option<Box<Node>>,
    pub text: Option<String>,
    pub src: Option<String>,
    pub style: Option<String>,
    pub class: Option<String>,
    pub i18n: Option<String>,
    pub action: Option<Action>,
    pub props: BTreeMap<String, Value>,
    pub width: Option<Size>,
    pub height: Option<Size>,
    pub grow: Option<f32>,
    pub fill: bool,
    pub gap: Option<Size>,
    pub padding: Option<Pad>,
    pub align: Option<Align>,
    pub justify: Option<Justify>,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Size {
    Px(f32),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Pad {
    All(f32),
    Text(String),
}

impl Pad {
    pub fn sides(&self) -> Option<[f32; 4]> {
        match self {
            Pad::All(value) => Some([*value; 4]),
            Pad::Text(text) => {
                let numbers: Option<Vec<f32>> = text
                    .split_whitespace()
                    .map(|part| part.trim_end_matches("px").parse().ok())
                    .collect();
                match numbers?.as_slice() {
                    [all] => Some([*all; 4]),
                    [vertical, horizontal] => {
                        Some([*vertical, *horizontal, *vertical, *horizontal])
                    }
                    [top, horizontal, bottom] => Some([*top, *horizontal, *bottom, *horizontal]),
                    [top, right, bottom, left] => Some([*top, *right, *bottom, *left]),
                    _ => None,
                }
            }
        }
    }
}

impl Size {
    pub fn px(&self) -> Option<f32> {
        match self {
            Size::Px(value) => Some(*value),
            Size::Text(_) => None,
        }
    }

    pub fn view(&self, unit: &str) -> Option<f32> {
        let Size::Text(text) = self else {
            return None;
        };
        let number = text.trim().strip_suffix(unit)?;
        number.trim().parse::<f32>().ok().map(|value| value / 100.0)
    }
}

const SHARED_PROPS: [&str; 34] = [
    "axis",
    "within",
    "group",
    "follow",
    "selected",
    "disabled",
    "span",
    "start",
    "lines",
    "center",
    "drag",
    "max",
    "popover",
    "hover",
    "float",
    "expand",
    "pop",
    "lift",
    "overlayCard",
    "rank",
    "expandOff",
    "expandedOnly",
    "posterOnly",
    "fallback",
    "share",
    "inset",
    "after",
    "indicator",
    "tint",
    "border",
    "color",
    "wrap",
    "logoOf",
    "hoverCompact",
];

const CONTAINERS: [&str; 8] = [
    "row", "column", "stack", "scroll", "pager", "fixed", "grid", "page",
];

pub(crate) fn is_container(kind: &str) -> bool {
    CONTAINERS.contains(&kind)
}

pub fn validate_node(
    node: &Node,
    includes: &BTreeMap<String, Node>,
    path: &str,
    problems: &mut Vec<String>,
) {
    check(node, includes, path, 0, problems);
}

fn check(
    node: &Node,
    includes: &BTreeMap<String, Node>,
    path: &str,
    depth: usize,
    problems: &mut Vec<String>,
) {
    let mut bad = |message: String| problems.push(format!("{path}: {message}"));
    if depth > MAX_DEPTH {
        bad(format!("nested deeper than {MAX_DEPTH} levels"));
        return;
    }
    for (name, size, unit) in [("width", &node.width, "vw"), ("height", &node.height, "vh")] {
        if let Some(size @ Size::Text(text)) = size
            && !text.contains('{')
            && size.view(unit).is_none()
        {
            bad(format!(
                "{name} \"{text}\" must be a number or a percentage like 60{unit}"
            ));
        }
    }
    if let Some(pad @ Pad::Text(text)) = &node.padding
        && !text.contains('{')
        && pad.sides().is_none()
    {
        bad(format!(
            "padding \"{text}\" must be 1 to 4 numbers like \"0 24 24\""
        ));
    }
    let modes = [
        !node.kind.is_empty(),
        node.include.is_some(),
        node.each.is_some(),
    ];
    if modes.iter().filter(|set| **set).count() != 1 {
        bad("needs exactly one of type, include or each".into());
        return;
    }
    if let Some(source) = &node.when
        && let Err(error) = Expr::parse(source)
    {
        bad(format!("when: {error}"));
    }
    for (name, source) in &node.with {
        if let Err(error) = Expr::parse(source) {
            bad(format!("with.{name}: {error}"));
        }
    }
    for (label, source) in [
        ("text", &node.text),
        ("src", &node.src),
        ("style", &node.style),
        ("class", &node.class),
    ] {
        if let Some(source) = source
            && let Err(error) = Template::parse(source)
        {
            bad(format!("{label}: {error}"));
        }
    }
    for (key, value) in &node.props {
        if let Value::String(source) = value
            && let Err(error) = Template::parse(source)
        {
            bad(format!("props.{key}: {error}"));
        }
    }
    if let Some(action) = &node.action {
        check_action(action, &mut bad);
    }

    if let Some(name) = &node.include {
        if !includes.contains_key(name) {
            bad(format!("unknown include {name:?}"));
        }
        return;
    }
    if let Some(source) = &node.each {
        if let Err(error) = Expr::parse(source) {
            bad(format!("each: {error}"));
        }
        match &node.template {
            Some(template) => check(
                template,
                includes,
                &format!("{path}.template"),
                depth + 1,
                problems,
            ),
            None => problems.push(format!("{path}: each needs a template")),
        }
        return;
    }

    let spec = contract();
    let known = spec.primitives.contains(&node.kind) || spec.components.contains_key(&node.kind);
    if !known {
        problems.push(format!("{path}: unknown type {:?}", node.kind));
        return;
    }
    let own = || {
        node.props
            .keys()
            .filter(|key| !SHARED_PROPS.contains(&key.as_str()))
    };
    if let Some(allowed) = spec.components.get(&node.kind) {
        for key in own().filter(|key| !allowed.contains(key)) {
            problems.push(format!("{path}: {} has no prop {key:?}", node.kind));
        }
    } else if own().next().is_some() {
        problems.push(format!("{path}: {} takes no props", node.kind));
    }
    if !node.children.is_empty() && !is_container(&node.kind) {
        problems.push(format!("{path}: {} cannot have children", node.kind));
    }
    for (index, child) in node.children.iter().enumerate() {
        let child_path = format!("{path}.children[{index}]");
        check(child, includes, &child_path, depth + 1, problems);
    }
}

fn check_action(action: &Action, bad: &mut impl FnMut(String)) {
    match contract().actions.get(&action.name) {
        Some(allowed) => {
            for key in action.args.keys().filter(|key| !allowed.contains(key)) {
                bad(format!("action {} has no argument {key:?}", action.name));
            }
        }
        None if extension_action(&action.name) => {}
        None => bad(format!("unknown action {:?}", action.name)),
    }
    for (key, value) in &action.args {
        if let Value::String(source) = value
            && let Err(error) = Template::parse(source)
        {
            bad(format!("action argument {key}: {error}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn problems(value: Value) -> Vec<String> {
        let node: Node = serde_json::from_value(value).unwrap();
        let mut found = Vec::new();
        validate_node(&node, &BTreeMap::new(), "screen", &mut found);
        found
    }

    #[test]
    fn valid_tree_has_no_problems() {
        let found = problems(json!({
            "type": "column",
            "children": [
                {"type": "text", "text": "{home.title}"},
                {"type": "row", "children": [],
                 "action": {"name": "play", "args": {"id": "{home.itemId}"}}}
            ]
        }));
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn unknown_action_and_prop_are_reported_with_their_path() {
        let found = problems(json!({
            "type": "row",
            "children": [{"type": "icon", "props": {"colour": "red"}, "action": {"name": "format_disk"}}]
        }));
        assert!(
            found
                .iter()
                .any(|f| f.contains("children[0]") && f.contains("colour"))
        );
        assert!(found.iter().any(|f| f.contains("unknown action")));
    }

    #[test]
    fn extension_actions_are_allowed_by_shape() {
        let found = problems(json!({"type": "row", "action": {"name": "ext.sync-tool.run"}}));
        assert!(found.is_empty(), "{found:?}");
        let found = problems(json!({"type": "row", "action": {"name": "ext.a.b.c"}}));
        assert!(!found.is_empty());
    }

    #[test]
    fn a_node_cannot_be_two_things() {
        assert!(!problems(json!({"type": "row", "include": "x"})).is_empty());
        assert!(!problems(json!({})).is_empty());
    }

    #[test]
    fn leaves_cannot_hold_children() {
        let found = problems(json!({"type": "text", "children": [{"type": "spacer"}]}));
        assert!(found.iter().any(|f| f.contains("cannot have children")));
    }

    #[test]
    fn unknown_fields_fail_to_parse() {
        assert!(serde_json::from_value::<Node>(json!({"type": "row", "colour": 1})).is_err());
    }

    #[test]
    fn excessive_depth_stops_the_walk() {
        let mut node = json!({"type": "spacer"});
        for _ in 0..40 {
            node = json!({"type": "row", "children": [node]});
        }
        let found = problems(node);
        assert!(found.iter().any(|f| f.contains("deeper")));
    }
}

#[cfg(test)]
mod size_tests {
    use super::*;

    #[test]
    fn view_units_parse_and_plain_numbers_stay_pixels() {
        let node: Node =
            serde_json::from_str(r#"{"type":"stack","height":"62vh","width":240}"#).unwrap();
        assert_eq!(
            node.height.as_ref().and_then(|size| size.view("vh")),
            Some(0.62)
        );
        assert_eq!(node.width.as_ref().and_then(Size::px), Some(240.0));
    }

    #[test]
    fn unknown_units_are_reported() {
        let node: Node = serde_json::from_str(r#"{"type":"stack","height":"tall"}"#).unwrap();
        let mut problems = Vec::new();
        validate_node(&node, &BTreeMap::new(), "home", &mut problems);
        assert_eq!(problems.len(), 1);
    }
}

#[cfg(test)]
mod pad_tests {
    use super::*;

    #[test]
    fn css_padding_shorthand_orders_like_css() {
        let sides = |text: &str| Pad::Text(text.into()).sides();
        assert_eq!(sides("0 24 24"), Some([0.0, 24.0, 24.0, 24.0]));
        assert_eq!(sides("1 2 3 4"), Some([1.0, 2.0, 3.0, 4.0]));
        assert_eq!(sides("8 16"), Some([8.0, 16.0, 8.0, 16.0]));
        assert_eq!(sides("a b"), None);
    }
}
