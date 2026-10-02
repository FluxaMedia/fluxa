use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Rect, Role as AkRole, Toggled, TreeId,
    TreeInfo, TreeUpdate,
};
use fluxa_host::{AccessNode, AccessTree, FluxaHost, Role};

const ROOT: NodeId = NodeId(u64::MAX);
const SEEK_STEP: f64 = 10.0;

pub fn tree_update(tree: &AccessTree, scale: f32, title: &str) -> TreeUpdate {
    let mut nodes = Vec::with_capacity(tree.nodes.len() + 1);
    let mut root = Node::new(AkRole::Window);
    root.set_label(title);
    root.set_children(
        tree.nodes
            .iter()
            .map(|node| NodeId(node.id))
            .collect::<Vec<_>>(),
    );
    nodes.push((ROOT, root));
    for node in &tree.nodes {
        nodes.push((NodeId(node.id), node_for(node, scale)));
    }
    TreeUpdate {
        nodes,
        tree: Some(TreeInfo::new(ROOT)),
        tree_id: TreeId::ROOT,
        focus: tree.focused.map(NodeId).unwrap_or(ROOT),
    }
}

fn node_for(node: &AccessNode, scale: f32) -> Node {
    let mut out = Node::new(match node.role {
        Role::Button => AkRole::Button,
        Role::Tab => AkRole::Tab,
        Role::Switch => AkRole::Switch,
        Role::Slider => AkRole::Slider,
        Role::TextInput | Role::UrlInput => AkRole::TextInput,
        Role::SearchInput => AkRole::SearchInput,
        Role::PasswordInput => AkRole::PasswordInput,
        Role::NumberInput => AkRole::NumberInput,
        Role::MultilineInput => AkRole::MultilineTextInput,
        Role::Text => AkRole::Label,
        Role::Heading => AkRole::Heading,
    });
    out.set_label(node.label.clone());
    if let Some(value) = &node.value {
        out.set_value(value.clone());
    }
    if let Some(checked) = node.checked {
        out.set_toggled(if checked {
            Toggled::True
        } else {
            Toggled::False
        });
    }
    if node.selected {
        out.set_selected(true);
    }
    if let Some([min, max, now]) = node.range {
        out.set_min_numeric_value(min);
        out.set_max_numeric_value(max);
        out.set_numeric_value(now);
        out.set_numeric_value_step(SEEK_STEP);
        out.add_action(Action::Increment);
        out.add_action(Action::Decrement);
        out.add_action(Action::SetValue);
    }
    let [x, y, w, h] = node.bounds.map(|v| (v * scale) as f64);
    out.set_bounds(Rect {
        x0: x,
        y0: y,
        x1: x + w,
        y1: y + h,
    });
    if matches!(node.role, Role::Text | Role::Heading) {
        return out;
    }
    out.add_action(Action::Focus);
    out.add_action(Action::Click);
    if node.role.is_input() {
        out.add_action(Action::SetValue);
    }
    out
}

pub fn handle(host: &FluxaHost, request: ActionRequest) {
    let id = request.target_node.0;
    match request.action {
        Action::Click => host.activate_node(id),
        Action::Focus => host.focus_node(id),
        Action::SetValue => match request.data {
            Some(ActionData::Value(text)) => {
                host.focus_node(id);
                host.set_focused_text(&text);
            }
            Some(ActionData::NumericValue(value)) => seek(host, value),
            _ => {}
        },
        Action::Increment => step(host, SEEK_STEP),
        Action::Decrement => step(host, -SEEK_STEP),
        _ => {}
    }
}

fn step(host: &FluxaHost, delta: f64) {
    let range = host
        .accessibility_tree()
        .and_then(|tree| tree.nodes.iter().find_map(|node| node.range));
    if let Some([min, max, now]) = range {
        seek(host, (now + delta).clamp(min, max));
    }
}

fn seek(host: &FluxaHost, seconds: f64) {
    host.media_command("seekTo", seconds);
}
