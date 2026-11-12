use super::*;
use fluxa_ui::{ImeOp, ImeState, InputKind, PendingField};
use std::hash::{DefaultHasher, Hash, Hasher};

const CODE_LENGTH: usize = 6;

#[derive(Default)]
pub(crate) struct ImeSession {
    field: u64,
    state: ImeState,
    notified: ImeState,
}

struct Target {
    id: u64,
    kind: InputKind,
    label: &'static str,
    hint: String,
    text: String,
    selection: [usize; 2],
    composing: Option<[usize; 2]>,
    egui: bool,
}

fn label(id: u64, kind: InputKind, hint: &str) -> &'static str {
    let code = hint == "code" || id == fluxa_ui::NODE_SETTINGS_WATCH_CODE;
    match kind {
        _ if code => "code",
        InputKind::Search => "search",
        InputKind::Password => "password",
        InputKind::Number { secret: true } => "pin",
        InputKind::Url => "url",
        InputKind::Number { secret: false } => "number",
        InputKind::Multiline => "multiline",
        InputKind::Text => "text",
    }
}

fn target(state: &RendererState) -> Option<Target> {
    if let Some(field) = &state.access.field {
        return Some(Target {
            id: field.id,
            kind: field.kind,
            label: label(field.id, field.kind, &field.hint),
            hint: field.hint.clone(),
            text: field.text.clone(),
            selection: field.selection,
            composing: field.composing,
            egui: true,
        });
    }
    let id = state.ui.focused()?;
    state
        .ui
        .node(id)
        .filter(|node| node.kind == UiNodeKind::Input)?;
    if state.keyboard_focus_visible && state.ime_engaged != Some(id) {
        return None;
    }
    let text = actions::node_text(state, id).unwrap_or_default();
    let end = text.chars().count();
    let hint = layout::label_for_node(state, id);
    let kind = fluxa_ui::input_kind(id);
    Some(Target {
        id,
        kind,
        label: label(id, kind, &hint),
        hint,
        text,
        selection: [end, end],
        composing: None,
        egui: false,
    })
}

fn adopt(state: &mut RendererState, target: &Target) {
    let mut next = ImeState::new(target.text.clone(), target.selection);
    next.composing = target.composing;
    let session = &mut state.ime;
    if session.field == target.id && !target.egui && session.state.text == target.text {
        next.selection = session.state.selection;
        next.composing = session.state.composing;
    }
    if session.field != target.id {
        session.notified = next.clone();
    }
    session.field = target.id;
    session.state = next;
}

pub(crate) fn sync(state: &mut RendererState) {
    match target(state) {
        Some(target) => adopt(state, &target),
        None => state.ime = ImeSession::default(),
    }
}

pub(crate) fn changed(state: &mut RendererState) -> bool {
    let session = &mut state.ime;
    if session.field == 0 || session.state == session.notified {
        return false;
    }
    session.notified = session.state.clone();
    true
}

fn current(state: &mut RendererState) -> Option<ImeState> {
    let target = target(state)?;
    if state.ime.field != target.id {
        adopt(state, &target);
    }
    Some(state.ime.state.clone())
}

fn constrain_code(ime: &mut ImeState) {
    let text: String = ime
        .text
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(CODE_LENGTH)
        .collect::<String>()
        .to_uppercase();
    *ime = ImeState::new(text, ime.selection);
}

fn edit(state: &mut RendererState, op: ImeOp) -> bool {
    let Some(target) = target(state) else {
        return false;
    };
    if state.ime.field != target.id {
        adopt(state, &target);
    }
    let before = state.ime.state.clone();
    state
        .ime
        .state
        .apply(&op, target.kind == InputKind::Multiline);
    if target.label == "code" {
        constrain_code(&mut state.ime.state);
    }
    let next = state.ime.state.clone();
    if next == before {
        return true;
    }
    if target.egui {
        state.field_text = Some(PendingField {
            id: target.id,
            text: next.text,
            selection: next.selection,
            composing: next.composing,
        });
    } else {
        actions::edit_text(state, target.id, |value| *value = next.text.clone());
    }
    true
}

fn op(code: u8, text: &str, cursor: i32, b: i32) -> Option<ImeOp> {
    let (a, b) = (cursor.max(0) as usize, b.max(0) as usize);
    Some(match code {
        0 => ImeOp::Commit {
            text: text.to_owned(),
            cursor,
        },
        1 => ImeOp::Compose {
            text: text.to_owned(),
            cursor,
        },
        2 => ImeOp::ComposeRegion { start: a, end: b },
        3 => ImeOp::FinishComposing,
        4 => ImeOp::DeleteSurrounding {
            before: a,
            after: b,
        },
        5 => ImeOp::DeleteSurroundingChars {
            before: a,
            after: b,
        },
        6 => ImeOp::Select { start: a, end: b },
        7 => ImeOp::SelectAll,
        8 => ImeOp::Backspace,
        _ => return None,
    })
}

impl FluxaHost {
    pub fn ime_field_json(&self) -> Option<String> {
        self.peek(|state| {
            let target = target(state)?;
            Some(
                serde_json::json!({
                    "id": target.id,
                    "kind": target.label,
                    "hint": target.hint,
                })
                .to_string(),
            )
        })
        .flatten()
    }

    pub fn ime_state(&self) -> Option<[i32; 5]> {
        self.peek(|state| {
            let ime = current(state)?;
            state.ime.notified = ime.clone();
            let [start, end] = ime.selection_units();
            let [from, to] = ime
                .composing_units()
                .map_or([-1, -1], |[from, to]| [from as i32, to as i32]);
            let mut hasher = DefaultHasher::new();
            ime.text.hash(&mut hasher);
            Some([start as i32, end as i32, from, to, hasher.finish() as i32])
        })
        .flatten()
    }

    pub fn ime_text(&self, mode: u8, count: usize) -> String {
        self.peek(|state| {
            let ime = current(state)?;
            Some(match mode {
                0 => ime.before_cursor(count),
                1 => ime.after_cursor(count),
                2 => ime.selected(),
                _ => ime.text,
            })
        })
        .flatten()
        .unwrap_or_default()
    }

    pub fn ime_edit(&self, code: u8, text: &str, a: i32, b: i32) -> bool {
        let Some(op) = op(code, text, a, b) else {
            return false;
        };
        self.with_state(|state| edit(state, op)).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host_with_focused_search() -> FluxaHost {
        let host = FluxaHost::new(1.0, None);
        host.set_form_factor("desktop");
        host.surface_changed(1280, 800);
        host.with_state(|state| {
            state.access.field = Some(fluxa_ui::FocusedField {
                id: fluxa_ui::NODE_LIBRARY_SEARCH,
                kind: InputKind::Search,
                size: 16.0,
                rect: egui::Rect::ZERO,
                text: String::new(),
                hint: "Search".to_owned(),
                selection: [0, 0],
                composing: None,
            });
        });
        host
    }

    #[test]
    fn composing_text_lands_in_the_focused_field() {
        let host = host_with_focused_search();
        assert!(host.ime_edit(1, "wor", 1, 0));
        assert_eq!(host.focused_text().as_deref(), Some("wor"));
        assert_eq!(host.ime_state().unwrap()[..4], [3, 3, 0, 3]);
        assert!(host.ime_edit(0, "world", 1, 0));
        assert_eq!(host.focused_text().as_deref(), Some("world"));
        assert_eq!(host.ime_state().unwrap()[..4], [5, 5, -1, -1]);
    }

    #[test]
    fn surrounding_text_is_served_from_the_field() {
        let host = host_with_focused_search();
        host.ime_edit(0, "hello", 1, 0);
        host.ime_edit(6, "", 2, 2);
        assert_eq!(host.ime_text(0, 10), "he");
        assert_eq!(host.ime_text(1, 10), "llo");
        host.ime_edit(4, "", 1, 1);
        assert_eq!(host.focused_text().as_deref(), Some("hlo"));
    }

    #[test]
    fn nothing_is_edited_without_a_focused_field() {
        let host = FluxaHost::new(1.0, None);
        host.set_form_factor("desktop");
        host.surface_changed(1280, 800);
        assert!(!host.ime_edit(0, "x", 1, 0));
        assert!(host.ime_field_json().is_none());
    }
}
