//! Platform-independent interaction model for mouse, touch, keyboard, and TV
//! remote hosts. It contains no Android, winit, Compose, or web types.

use std::collections::HashMap;

use crate::Rect;

pub type NodeId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNodeKind {
    Root,
    Container,
    Button,
    Input,
    Image,
    Text,
    Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusPolicy {
    None,
    Focusable,
    Scope,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub bounds: Rect,
    pub kind: UiNodeKind,
    pub focus: FocusPolicy,
    pub enabled: bool,
    pub visible: bool,
    pub order: u32,
    pub label: Option<String>,
    pub children: Vec<NodeId>,
}

impl UiNode {
    pub fn new(id: NodeId, kind: UiNodeKind, bounds: Rect) -> Self {
        Self {
            id,
            parent: None,
            bounds,
            kind,
            focus: FocusPolicy::None,
            enabled: true,
            visible: true,
            order: 0,
            label: None,
            children: Vec::new(),
        }
    }

    pub fn focusable(mut self) -> Self {
        self.focus = FocusPolicy::Focusable;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Enter,
    Escape,
    Tab,
    ShiftTab,
    Up,
    Down,
    Left,
    Right,
    Back,
}

/// Logical controller buttons. Hosts translate their platform-specific
/// controller APIs into this small vocabulary before dispatching to a screen.
/// Keeping this here makes gamepad and TV remote navigation use the same focus
/// graph as keyboard navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Start,
    Select,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEvent {
    PointerMove {
        position: [f32; 2],
    },
    PointerDown {
        position: [f32; 2],
        button: PointerButton,
    },
    PointerUp {
        position: [f32; 2],
        button: PointerButton,
    },
    KeyDown(Key),
    GamepadButton {
        button: GamepadButton,
        pressed: bool,
    },
    TextInput(String),
    Scroll {
        delta: [f32; 2],
    },
    FocusRequest(NodeId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiAction {
    FocusChanged {
        previous: Option<NodeId>,
        current: Option<NodeId>,
    },
    Activated(NodeId),
    PointerMoved {
        node: Option<NodeId>,
        position: [f32; 2],
    },
    PointerPressed(NodeId),
    PointerReleased(NodeId),
    TextInput {
        node: NodeId,
        value: String,
    },
    Scroll {
        node: Option<NodeId>,
        delta: [f32; 2],
    },
    Back,
}

pub struct UiTree {
    root: NodeId,
    next_id: NodeId,
    nodes: HashMap<NodeId, UiNode>,
    focused: Option<NodeId>,
}

impl Default for UiTree {
    fn default() -> Self {
        let root = UiNode::new(1, UiNodeKind::Root, Rect::default());
        let mut nodes = HashMap::new();
        nodes.insert(root.id, root);
        Self {
            root: 1,
            next_id: 2,
            nodes,
            focused: None,
        }
    }
}

impl UiTree {
    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    pub fn node(&self, id: NodeId) -> Option<&UiNode> {
        self.nodes.get(&id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(&id)
    }

    pub fn add(&mut self, parent: NodeId, mut node: UiNode) -> Option<NodeId> {
        if !self.nodes.contains_key(&parent) || self.nodes.contains_key(&node.id) {
            return None;
        }
        node.parent = Some(parent);
        let id = node.id;
        self.nodes.get_mut(&parent)?.children.push(id);
        self.nodes.insert(id, node);
        self.next_id = self.next_id.max(id.saturating_add(1));
        Some(id)
    }

    pub fn create(&mut self, parent: NodeId, kind: UiNodeKind, bounds: Rect) -> Option<NodeId> {
        let id = self.next_id;
        self.add(parent, UiNode::new(id, kind, bounds))
    }

    pub fn set_focus(&mut self, requested: Option<NodeId>) -> Option<UiAction> {
        let next = requested.filter(|id| self.is_focusable(*id));
        if self.focused == next {
            return None;
        }
        let previous = self.focused;
        self.focused = next;
        Some(UiAction::FocusChanged {
            previous,
            current: next,
        })
    }

    pub fn dispatch(&mut self, event: UiEvent) -> Vec<UiAction> {
        match event {
            UiEvent::FocusRequest(id) => self.set_focus(Some(id)).into_iter().collect(),
            UiEvent::KeyDown(key) => self.dispatch_key(key),
            UiEvent::GamepadButton { button, pressed } => {
                if !pressed {
                    return Vec::new();
                }
                let key = match button {
                    GamepadButton::South | GamepadButton::Start => Key::Enter,
                    GamepadButton::East | GamepadButton::Select => Key::Back,
                    GamepadButton::DPadUp => Key::Up,
                    GamepadButton::DPadDown => Key::Down,
                    GamepadButton::DPadLeft => Key::Left,
                    GamepadButton::DPadRight => Key::Right,
                    GamepadButton::North | GamepadButton::West => return Vec::new(),
                };
                self.dispatch_key(key)
            }
            UiEvent::PointerMove { position } => vec![UiAction::PointerMoved {
                node: self.hit_test(position),
                position,
            }],
            UiEvent::PointerDown { position, button } => {
                let Some(node) = self.hit_test(position) else {
                    return Vec::new();
                };
                if button == PointerButton::Primary && self.is_focusable(node) {
                    let mut actions = self.set_focus(Some(node)).into_iter().collect::<Vec<_>>();
                    actions.push(UiAction::PointerPressed(node));
                    actions
                } else {
                    Vec::new()
                }
            }
            UiEvent::PointerUp { position, button } => {
                if button == PointerButton::Primary {
                    self.hit_test(position)
                        .map(UiAction::PointerReleased)
                        .into_iter()
                        .collect()
                } else {
                    Vec::new()
                }
            }
            UiEvent::TextInput(value) => self
                .focused
                .filter(|id| {
                    self.node(*id)
                        .is_some_and(|node| node.kind == UiNodeKind::Input)
                })
                .map(|node| UiAction::TextInput { node, value })
                .into_iter()
                .collect(),
            UiEvent::Scroll { delta } => vec![UiAction::Scroll {
                node: self.focused,
                delta,
            }],
        }
    }

    pub fn hit_test(&self, position: [f32; 2]) -> Option<NodeId> {
        self.nodes
            .values()
            .filter(|node| node.visible && node.enabled && node.bounds.contains(position))
            .max_by_key(|node| (node.order, node.id))
            .map(|node| node.id)
    }

    fn dispatch_key(&mut self, key: Key) -> Vec<UiAction> {
        match key {
            Key::Enter => self
                .focused
                .filter(|id| self.is_focusable(*id))
                .map(UiAction::Activated)
                .into_iter()
                .collect(),
            Key::Escape | Key::Back => vec![UiAction::Back],
            Key::Tab => self.move_linear(1).into_iter().collect(),
            Key::ShiftTab => self.move_linear(-1).into_iter().collect(),
            Key::Up => self
                .move_spatial(NavigationDirection::Up)
                .into_iter()
                .collect(),
            Key::Down => self
                .move_spatial(NavigationDirection::Down)
                .into_iter()
                .collect(),
            Key::Left => self
                .move_spatial(NavigationDirection::Left)
                .into_iter()
                .collect(),
            Key::Right => self
                .move_spatial(NavigationDirection::Right)
                .into_iter()
                .collect(),
        }
    }

    fn is_focusable(&self, id: NodeId) -> bool {
        self.nodes.get(&id).is_some_and(|node| {
            node.focus == FocusPolicy::Focusable && node.visible && node.enabled
        })
    }

    fn focusable_nodes(&self) -> Vec<&UiNode> {
        let mut nodes = self
            .nodes
            .values()
            .filter(|node| self.is_focusable(node.id))
            .collect::<Vec<_>>();
        nodes.sort_by_key(|node| (node.order, node.id));
        nodes
    }

    fn move_linear(&mut self, step: i32) -> Option<UiAction> {
        let nodes = self.focusable_nodes();
        if nodes.is_empty() {
            return None;
        }
        let current = self
            .focused
            .and_then(|id| nodes.iter().position(|node| node.id == id));
        let index = match current {
            Some(index) => (index as i32 + step).rem_euclid(nodes.len() as i32) as usize,
            None if step >= 0 => 0,
            None => nodes.len() - 1,
        };
        self.set_focus(Some(nodes[index].id))
    }

    pub fn focus_nearest_to_edge(
        &mut self,
        edge: f32,
        top: f32,
        bottom: f32,
        allow: impl Fn(NodeId) -> bool,
    ) -> Option<UiAction> {
        let current = self.focused.and_then(|id| self.node(id))?.bounds.center();
        let candidate = self
            .focusable_nodes()
            .into_iter()
            .filter(|node| allow(node.id))
            .filter(|node| {
                let center = node.bounds.center()[1];
                center >= top && center <= bottom
            })
            .map(|node| {
                let center = node.bounds.center();
                let score = (center[1] - edge).abs() * 1000.0 + (center[0] - current[0]).abs();
                (score, node.id)
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))?;
        self.set_focus(Some(candidate.1))
    }

    fn move_spatial(&mut self, direction: NavigationDirection) -> Option<UiAction> {
        let current_id = self.focused?;
        let bounds = self.node(current_id)?.bounds;
        let current = bounds.center();
        let candidate = self
            .focusable_nodes()
            .into_iter()
            .filter(|node| node.id != current_id)
            .filter(|node| match direction {
                NavigationDirection::Left | NavigationDirection::Right => {
                    node.bounds.y < bounds.y + bounds.height
                        && node.bounds.y + node.bounds.height > bounds.y
                }
                _ => true,
            })
            .filter_map(|node| {
                let center = node.bounds.center();
                let primary = match direction {
                    NavigationDirection::Up if center[1] < current[1] => current[1] - center[1],
                    NavigationDirection::Down if center[1] > current[1] => center[1] - current[1],
                    NavigationDirection::Left if center[0] < current[0] => current[0] - center[0],
                    NavigationDirection::Right if center[0] > current[0] => center[0] - current[0],
                    _ => return None,
                };
                let secondary = match direction {
                    NavigationDirection::Up | NavigationDirection::Down => {
                        (center[0] - current[0]).abs()
                    }
                    NavigationDirection::Left | NavigationDirection::Right => {
                        (center[1] - current[1]).abs()
                    }
                };
                Some((primary * 1000.0 + secondary, node.id))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))?;
        self.set_focus(Some(candidate.1))
    }
}

impl Rect {
    fn contains(self, position: [f32; 2]) -> bool {
        position[0] >= self.x
            && position[0] <= self.x + self.width
            && position[1] >= self.y
            && position[1] <= self.y + self.height
    }

    fn center(self) -> [f32; 2] {
        [self.x + self.width / 2.0, self.y + self.height / 2.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpad_moves_to_the_nearest_directional_node() {
        let mut tree = UiTree::default();
        let first = tree
            .add(
                tree.root(),
                UiNode::new(10, UiNodeKind::Button, Rect::new(0.0, 0.0, 40.0, 40.0)).focusable(),
            )
            .expect("first node");
        let second = tree
            .add(
                tree.root(),
                UiNode::new(11, UiNodeKind::Button, Rect::new(60.0, 0.0, 40.0, 40.0)).focusable(),
            )
            .expect("second node");
        tree.set_focus(Some(first));
        assert_eq!(
            tree.dispatch(UiEvent::KeyDown(Key::Right)),
            vec![UiAction::FocusChanged {
                previous: Some(first),
                current: Some(second)
            }]
        );
    }

    #[test]
    fn text_is_delivered_only_to_focused_inputs() {
        let mut tree = UiTree::default();
        let input = tree
            .create(
                tree.root(),
                UiNodeKind::Input,
                Rect::new(0.0, 0.0, 100.0, 40.0),
            )
            .expect("input");
        tree.node_mut(input).expect("input node").focus = FocusPolicy::Focusable;
        tree.set_focus(Some(input));
        assert_eq!(
            tree.dispatch(UiEvent::TextInput("fluxa".to_owned())),
            vec![UiAction::TextInput {
                node: input,
                value: "fluxa".to_owned()
            }]
        );
    }

    #[test]
    fn gamepad_buttons_share_keyboard_navigation_and_activation() {
        let mut tree = UiTree::default();
        let first = tree
            .add(
                tree.root(),
                UiNode::new(20, UiNodeKind::Button, Rect::new(0.0, 0.0, 40.0, 40.0)).focusable(),
            )
            .expect("first button");
        let second = tree
            .add(
                tree.root(),
                UiNode::new(21, UiNodeKind::Button, Rect::new(60.0, 0.0, 40.0, 40.0)).focusable(),
            )
            .expect("second button");
        tree.set_focus(Some(first));
        assert_eq!(
            tree.dispatch(UiEvent::GamepadButton {
                button: GamepadButton::DPadRight,
                pressed: true,
            }),
            vec![UiAction::FocusChanged {
                previous: Some(first),
                current: Some(second),
            }]
        );
        assert_eq!(
            tree.dispatch(UiEvent::GamepadButton {
                button: GamepadButton::South,
                pressed: true,
            }),
            vec![UiAction::Activated(second)]
        );
        assert!(
            tree.dispatch(UiEvent::GamepadButton {
                button: GamepadButton::South,
                pressed: false,
            })
            .is_empty()
        );
    }
}
