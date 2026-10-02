use super::*;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Button,
    Tab,
    Switch,
    Slider,
    TextInput,
    SearchInput,
    PasswordInput,
    UrlInput,
    NumberInput,
    MultilineInput,
    Text,
    Heading,
}

const EGUI_FIELD: u64 = 0x3FFF_FF00;
const TEXT_BASE: u64 = 0x4000_0000;
const HEADING_SIZE: f32 = 18.0;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AccessNode {
    pub id: u64,
    pub role: Role,
    pub label: String,
    pub value: Option<String>,
    pub checked: Option<bool>,
    pub selected: bool,
    pub focused: bool,
    pub bounds: [f32; 4],
    pub range: Option<[f64; 3]>,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct AccessTree {
    pub nodes: Vec<AccessNode>,
    pub focused: Option<u64>,
    pub width: f32,
    pub height: f32,
    pub caret: Option<[f32; 4]>,
    pub selection: Option<[usize; 2]>,
}

impl Role {
    pub fn is_input(self) -> bool {
        matches!(
            self,
            Role::TextInput
                | Role::SearchInput
                | Role::PasswordInput
                | Role::UrlInput
                | Role::NumberInput
                | Role::MultilineInput
        )
    }
}

fn role_for(kind: fluxa_ui::InputKind) -> Role {
    use fluxa_ui::InputKind;
    match kind {
        InputKind::Text => Role::TextInput,
        InputKind::Search => Role::SearchInput,
        InputKind::Password | InputKind::Number { secret: true } => Role::PasswordInput,
        InputKind::Url => Role::UrlInput,
        InputKind::Number { secret: false } => Role::NumberInput,
        InputKind::Multiline => Role::MultilineInput,
    }
}

fn nav_selected(route: Route, id: u64) -> Option<bool> {
    let owner = match id {
        fluxa_ui::NODE_HOME => Route::Home,
        fluxa_ui::NODE_LIBRARY => Route::Library,
        fluxa_ui::NODE_DISCOVER => Route::Discover,
        fluxa_ui::NODE_CALENDAR => Route::Calendar,
        fluxa_ui::NODE_SHORTS => Route::Shorts,
        _ => return None,
    };
    Some(route == owner)
}

fn language(state: &RendererState) -> String {
    state
        .settings
        .str_value("language")
        .unwrap_or("en")
        .to_owned()
}

fn player_label(state: &RendererState, id: u64) -> Option<String> {
    let session = state.player.as_ref()?;
    let model = session.model();
    let lang = language(state);
    let text = |key: &str| fluxa_ui::localized(key, &lang);
    Some(match id {
        fluxa_ui::NODE_PLAYER_CLOSE | fluxa_ui::NODE_PLAYER_PANEL_CLOSE => text("player.close"),
        fluxa_ui::NODE_PLAYER_TOGGLE => text(if model.paused {
            "player.play"
        } else {
            "player.pause"
        }),
        fluxa_ui::NODE_PLAYER_SEEK => text("a11y.seek"),
        fluxa_ui::NODE_PLAYER_REWIND => text("a11y.rewind"),
        fluxa_ui::NODE_PLAYER_FORWARD => text("a11y.forward"),
        fluxa_ui::NODE_PLAYER_MUTE => text(if model.muted {
            "player.unmute"
        } else {
            "player.mute"
        }),
        fluxa_ui::NODE_PLAYER_FULLSCREEN => text("player.fullscreen"),
        fluxa_ui::NODE_PLAYER_SKIP => text("player.skip"),
        fluxa_ui::NODE_PLAYER_CAST => text("player.cast"),
        fluxa_ui::NODE_PLAYER_SETTINGS => text("player.settings"),
        fluxa_ui::NODE_PLAYER_TRACKS => text("player.audio_and_subtitles"),
        fluxa_ui::NODE_PLAYER_SPEED => text("player.speed"),
        fluxa_ui::NODE_PLAYER_EPISODES => text("player.episodes"),
        fluxa_ui::NODE_PLAYER_NEXT | fluxa_ui::NODE_PLAYER_NEXT_PLAY => text("a11y.next_episode"),
        fluxa_ui::NODE_PLAYER_PREVIOUS => text("a11y.previous_episode"),
        fluxa_ui::NODE_PLAYER_NEXT_DISMISS => text("player.close"),
        _ => return None,
    })
}

fn describe(
    state: &RendererState,
    id: u64,
) -> (
    Role,
    String,
    Option<String>,
    Option<bool>,
    bool,
    Option<[f64; 3]>,
) {
    let lang = language(state);
    if state.player.is_some() {
        if let Some(name) = player_label(state, id) {
            let model = state.player.as_ref().map(|player| player.model());
            return match (id, model) {
                (fluxa_ui::NODE_PLAYER_SEEK, Some(model)) => {
                    let duration = model.duration.max(0.0);
                    let now = model
                        .scrub
                        .unwrap_or(model.position)
                        .clamp(0.0, duration.max(1.0));
                    (
                        Role::Slider,
                        name,
                        Some(format!(
                            "{} / {}",
                            fluxa_ui::format_time(now),
                            fluxa_ui::format_time(duration)
                        )),
                        None,
                        false,
                        Some([0.0, duration, now]),
                    )
                }
                (fluxa_ui::NODE_PLAYER_MUTE, Some(model)) => {
                    (Role::Switch, name, None, Some(model.muted), false, None)
                }
                _ => (Role::Button, name, None, None, false, None),
            };
        }
    }
    if fluxa_ui::is_text_node(id) {
        let role = role_for(fluxa_ui::input_kind(id));
        let value = if role == Role::PasswordInput {
            None
        } else {
            actions::node_text(state, id).filter(|text| !text.is_empty())
        };
        return (
            role,
            layout::label_for_node(state, id),
            value,
            None,
            false,
            None,
        );
    }
    if let Some(selected) = nav_selected(state.route, id) {
        let key = match id {
            fluxa_ui::NODE_HOME => "nav.home",
            fluxa_ui::NODE_LIBRARY => "nav.library",
            fluxa_ui::NODE_DISCOVER => "nav.discover",
            fluxa_ui::NODE_CALENDAR => "nav.calendar",
            _ => "nav.shorts",
        };
        return (
            Role::Tab,
            fluxa_ui::localized(key, &lang),
            None,
            None,
            selected,
            None,
        );
    }
    if state.route == Route::Settings {
        let rows = fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64;
        if rows.contains(&id)
            && let Some(row) =
                fluxa_ui::settings_row_by_index((id - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize)
        {
            let name = fluxa_ui::settings_row_label(row, &lang);
            return if row.options.is_empty() {
                (
                    Role::Switch,
                    name,
                    None,
                    Some(state.settings.bool_value(row.key)),
                    false,
                    None,
                )
            } else {
                let value = fluxa_ui::option_label(
                    row.key,
                    state.settings.str_value(row.key).unwrap_or(row.options[0]),
                    &lang,
                );
                (Role::Button, name, Some(value), None, false, None)
            };
        }
    }
    if id == fluxa_ui::NODE_DETAIL_WATCHLIST && state.route == Route::Detail {
        let key = if state.detail.in_watchlist {
            "menu.remove_watchlist"
        } else {
            "menu.add_watchlist"
        };
        return (
            Role::Button,
            fluxa_ui::localized(key, &lang),
            None,
            None,
            false,
            None,
        );
    }
    (
        Role::Button,
        layout::label_for_node(state, id),
        None,
        None,
        false,
        None,
    )
}

pub(super) fn tree(state: &RendererState) -> AccessTree {
    let [width, height] = layout::logical_surface_size(state);
    let (width, height) = (width as f32, height as f32);
    let focused = state.ui.focused();
    let controls: Vec<_> = state
        .ui
        .node(state.ui.root())
        .into_iter()
        .flat_map(|root| root.children.iter())
        .filter_map(|id| state.ui.node(*id))
        .filter(|node| node.visible && node.enabled)
        .collect();
    let mut nodes: Vec<AccessNode> = controls
        .iter()
        .map(|node| {
            let (role, label, value, checked, selected, range) = describe(state, node.id);
            AccessNode {
                id: node.id,
                role,
                label,
                value,
                checked,
                selected,
                focused: focused == Some(node.id),
                bounds: [
                    node.bounds.x,
                    node.bounds.y,
                    node.bounds.width,
                    node.bounds.height,
                ],
                range,
            }
        })
        .collect();

    let field = state.access.field.as_ref();
    if let Some(field) = field {
        let lang = language(state);
        let label = if field.hint.is_empty() {
            fluxa_ui::localized("a11y.text_field", &lang)
        } else {
            field.hint.clone()
        };
        nodes.push(AccessNode {
            id: EGUI_FIELD,
            role: role_for(field.kind),
            label,
            value: (!field.kind.secret() && !field.text.is_empty()).then(|| field.text.clone()),
            checked: None,
            selected: false,
            focused: true,
            bounds: [
                field.rect.min.x,
                field.rect.min.y,
                field.rect.width(),
                field.rect.height(),
            ],
            range: None,
        });
    }

    let interactive: Vec<[f32; 4]> = nodes
        .iter()
        .map(|node| node.bounds)
        .filter(|b| b[2] * b[3] < width * height * 0.6)
        .collect();
    let mut seen = Vec::new();
    for run in &state.access.runs {
        let center = [
            run.rect[0] + run.rect[2] / 2.0,
            run.rect[1] + run.rect[3] / 2.0,
        ];
        let covered = interactive.iter().any(|b| {
            center[0] >= b[0]
                && center[0] <= b[0] + b[2]
                && center[1] >= b[1]
                && center[1] <= b[1] + b[3]
        });
        if covered || seen.contains(&(run.text.as_str(), run.rect.map(|v| v as i32))) {
            continue;
        }
        seen.push((run.text.as_str(), run.rect.map(|v| v as i32)));
        nodes.push(AccessNode {
            id: 0,
            role: if run.size >= HEADING_SIZE {
                Role::Heading
            } else {
                Role::Text
            },
            label: run.text.clone(),
            value: None,
            checked: None,
            selected: false,
            focused: false,
            bounds: run.rect,
            range: None,
        });
    }

    nodes.sort_by(|a, b| {
        let key = |n: &AccessNode| ((n.bounds[1] / 12.0) as i32, n.bounds[0] as i32);
        key(a).cmp(&key(b))
    });
    let mut text_index = 0;
    for node in nodes.iter_mut().filter(|node| node.id == 0) {
        node.id = TEXT_BASE + text_index;
        text_index += 1;
    }

    let selection = match (field, focused) {
        (Some(field), _) => Some(field.selection),
        (None, Some(id)) if fluxa_ui::is_text_node(id) => actions::node_text(state, id)
            .map(|text| text.chars().count())
            .map(|len| [len, len]),
        _ => None,
    };
    AccessTree {
        focused: field.map(|_| EGUI_FIELD).or(focused),
        nodes,
        width,
        height,
        caret: state.access.caret,
        selection,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> AccessTree {
        let host = FluxaHost::new(1.0, None);
        host.set_form_factor("desktop");
        host.surface_changed(1280, 800);
        host.accessibility_tree().unwrap()
    }

    #[test]
    fn home_exposes_nav_tabs_with_one_selected() {
        let tree = tree();
        let tabs: Vec<_> = tree.nodes.iter().filter(|n| n.role == Role::Tab).collect();
        assert!(!tabs.is_empty());
        assert_eq!(tabs.iter().filter(|n| n.selected).count(), 1);
    }

    #[test]
    fn every_node_has_a_label_and_visible_bounds() {
        for node in tree().nodes {
            assert!(!node.label.is_empty(), "node {} has no label", node.id);
            assert!(node.bounds[2] > 0.0 && node.bounds[3] > 0.0);
        }
    }

    #[test]
    fn rendered_text_becomes_ordered_static_nodes() {
        let host = FluxaHost::new(1.0, None);
        host.set_form_factor("desktop");
        host.surface_changed(1280, 800);
        let tree = host.accessibility_tree().unwrap();
        let text: Vec<_> = tree
            .nodes
            .iter()
            .filter(|n| matches!(n.role, Role::Text | Role::Heading))
            .collect();
        assert!(text.iter().all(|n| n.id >= TEXT_BASE));
        let ys: Vec<_> = text.iter().map(|n| (n.bounds[1] / 12.0) as i32).collect();
        assert!(ys.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn input_roles_follow_kind() {
        use fluxa_ui::InputKind;
        assert_eq!(
            role_for(InputKind::Number { secret: true }),
            Role::PasswordInput
        );
        assert_eq!(
            role_for(InputKind::Number { secret: false }),
            Role::NumberInput
        );
        assert_eq!(role_for(InputKind::Multiline), Role::MultilineInput);
    }
}
