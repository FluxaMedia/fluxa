use super::*;

pub(crate) fn remember(frame: SkinFrame) -> HomeLayout {
    if let Ok(mut slot) = TARGETS.lock() {
        *slot = frame.targets;
    }
    frame.layout
}

pub(crate) fn remember_more(frame: SkinFrame) -> HomeLayout {
    if let Ok(mut slot) = TARGETS.lock() {
        slot.extend(frame.targets);
    }
    frame.layout
}

pub(crate) fn action_for_node(node: u64) -> Option<NativeAction> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    match target {
        SkinTarget::Action(action) => native_action(action),
        SkinTarget::Field(_) | SkinTarget::Control => None,
    }
}

pub(crate) enum Local {
    Choose(String, String),
    CalendarDay(u32),
    CalendarMonth(i32),
    Downloads(String, String),
    Detail(String, usize),
    Profiles(String, usize),
    Player(String),
    PlayerRow(usize),
    Update(String),
    Window(String),
    Modal(String, usize),
    Folder(String, usize),
    Guide(u64),
    Welcome(String),
    Watch(String, usize),
    Skins(String, String),
    Script(String, String, Value),
    Page(String),
    Node(u64, String),
    MenuPick(usize),
    MenuDismiss,
}

pub(crate) fn local_for_node(node: u64) -> Option<Local> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    let SkinTarget::Action(action) = target else {
        return None;
    };
    if let Some(rest) = action.name.strip_prefix("ext.")
        && let Some((id, method)) = rest.split_once('.')
    {
        return Some(Local::Script(
            id.to_owned(),
            method.to_owned(),
            json!(action.args),
        ));
    }
    match action.name.as_str() {
        "ext_setting" => Some(Local::Script(
            text(action, "id")?,
            "setting".to_owned(),
            json!({"key": text(action, "key")?, "value": action.args.get("value")?}),
        )),
        "page" => Some(Local::Page(text(action, "name")?)),
        "node" => Some(Local::Node(
            text(action, "id")?.parse().ok()?,
            text(action, "fill").unwrap_or_default(),
        )),
        "menu_pick" => Some(Local::MenuPick(text(action, "index")?.parse().ok()?)),
        "menu_dismiss" => Some(Local::MenuDismiss),
        "choose" => Some(Local::Choose(text(action, "key")?, text(action, "value")?)),
        "calendar_day" => Some(Local::CalendarDay(text(action, "day")?.parse().ok()?)),
        "calendar_month" => Some(Local::CalendarMonth(text(action, "delta")?.parse().ok()?)),
        "downloads" => Some(Local::Downloads(text(action, "op")?, text(action, "id")?)),
        "folder" => Some(Local::Folder(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "welcome" => Some(Local::Welcome(text(action, "op")?)),
        "watch" => Some(Local::Watch(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "skins" => Some(Local::Skins(
            text(action, "op")?,
            text(action, "id").unwrap_or_default(),
        )),
        "guide" => Some(Local::Guide(text(action, "node")?.parse().ok()?)),
        "player" => Some(Local::Player(text(action, "op")?)),
        "update" => Some(Local::Update(text(action, "op")?)),
        "window" => Some(Local::Window(text(action, "op")?)),
        "modal" => Some(Local::Modal(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "player_row" => Some(Local::PlayerRow(text(action, "index")?.parse().ok()?)),
        "profiles" => Some(Local::Profiles(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "detail" => Some(Local::Detail(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        _ => None,
    }
}

pub(crate) fn fill_for_node(node: u64) -> Vec<(u64, String)> {
    let Some(Local::Node(_, fill)) = local_for_node(node) else {
        return Vec::new();
    };
    fill.split(';')
        .filter_map(|pair| {
            let (target, field) = pair.split_once('=')?;
            Some((target.parse().ok()?, field_value(field).unwrap_or_default()))
        })
        .collect()
}

pub(crate) fn detail_node(node: u64) -> Option<u64> {
    use fluxa_ui::*;
    let local = local_for_node(node)?;
    if let Local::Profiles(op, index) = &local {
        let index = *index as u64;
        return match op.as_str() {
            "select" | "edit" | "add" => Some(NODE_PROFILES_CELL_BASE + index),
            "manage" => Some(NODE_PROFILES_MANAGE),
            "gear" => Some(NODE_PROFILES_GEAR),
            _ => None,
        };
    }
    if let Local::Guide(node) | Local::Node(node, _) = &local {
        return Some(*node);
    }
    if let Local::Folder(op, index) = &local {
        return match op.as_str() {
            "back" => Some(NODE_FOLDER_BACK),
            "tab" => Some(NODE_FOLDER_TAB_BASE + *index as u64),
            _ => None,
        };
    }
    if let Local::Update(op) = &local {
        return match op.as_str() {
            "later" => Some(NODE_UPDATE_LATER),
            "download" => Some(NODE_UPDATE_DOWNLOAD),
            "install" => Some(NODE_UPDATE_INSTALL),
            _ => None,
        };
    }
    if matches!(local, Local::Window(_)) {
        return Some(NODE_WINDOW_FULLSCREEN);
    }
    if let Local::PlayerRow(index) = &local {
        return Some(NODE_PLAYER_PANEL_ROW_BASE + *index as u64);
    }
    if let Local::Player(op) = &local {
        return Some(match op.as_str() {
            "close" => NODE_PLAYER_CLOSE,
            "toggle" => NODE_PLAYER_TOGGLE,
            "mute" => NODE_PLAYER_MUTE,
            "fullscreen" => NODE_PLAYER_FULLSCREEN,
            "previous" => NODE_PLAYER_PREVIOUS,
            "next" => NODE_PLAYER_NEXT,
            "tracks" => NODE_PLAYER_TRACKS,
            "speed" => NODE_PLAYER_SPEED,
            "episodes" => NODE_PLAYER_EPISODES,
            "settings" => NODE_PLAYER_SETTINGS,
            "info" => NODE_PLAYER_INFO,
            "cast" => NODE_PLAYER_CAST,
            "show" => NODE_PLAYER_CONTROLS,
            "hide" => NODE_PLAYER_HIDE_CONTROLS,
            "panel_close" => NODE_PLAYER_PANEL_CLOSE,
            "expand" => NODE_PLAYER_EXPAND,
            "skip" => NODE_PLAYER_SKIP,
            "next_play" => NODE_PLAYER_NEXT_PLAY,
            "next_dismiss" => NODE_PLAYER_NEXT_DISMISS,
            _ => return None,
        });
    }
    let Local::Detail(op, index) = local else {
        return None;
    };
    let index = index as u64;
    Some(match op.as_str() {
        "back" => NODE_DETAIL_BACK,
        "play" => NODE_DETAIL_PLAY,
        "watchlist" => NODE_DETAIL_WATCHLIST,
        "completed" => NODE_DETAIL_COMPLETED,
        "dropped" => NODE_DETAIL_DROPPED,
        "favorite" => NODE_DETAIL_FAVORITE,
        "shuffle" => NODE_DETAIL_SHUFFLE,
        "episode" => NODE_DETAIL_EPISODE_BASE + index,
        "cast" => NODE_DETAIL_CAST_BASE + index,
        "trailer" => NODE_DETAIL_TRAILER_BASE + index,
        "rating" => NODE_DETAIL_RATING_BASE + index,
        _ => return None,
    })
}

pub(crate) fn item_for_node(node: u64) -> Option<String> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    match target {
        SkinTarget::Action(action) if matches!(action.name.as_str(), "open_detail" | "play") => {
            text(action, "id")
        }
        _ => None,
    }
}

pub(super) fn field_name(node: u64) -> Option<String> {
    let targets = TARGETS.lock().ok()?;
    targets.iter().find_map(|(id, target)| match target {
        SkinTarget::Field(name) if *id == node => Some(name.clone()),
        _ => None,
    })
}

pub(crate) fn field_value(name: &str) -> Option<String> {
    let fields = FIELDS.lock().ok()?;
    fields.as_ref()?.get(name).cloned()
}

pub(crate) fn is_field(node: u64) -> bool {
    field_name(node).is_some()
}

pub(crate) fn field_text(node: u64) -> Option<String> {
    let name = field_name(node)?;
    let fields = FIELDS.lock().ok()?;
    Some(
        fields
            .as_ref()
            .and_then(|fields| fields.get(&name).cloned())
            .unwrap_or_default(),
    )
}

pub(crate) fn set_field(name: &str, value: String) {
    if let Ok(mut fields) = FIELDS.lock() {
        fields
            .get_or_insert_default()
            .insert(name.to_owned(), value);
    }
}

pub(crate) fn take_field(name: &str) -> String {
    FIELDS
        .lock()
        .ok()
        .and_then(|mut fields| fields.as_mut()?.remove(name))
        .unwrap_or_default()
}

pub(crate) fn edit_field(node: u64, edit: impl FnOnce(&mut String)) -> bool {
    touch();
    let Some(name) = field_name(node) else {
        return false;
    };
    let Ok(mut fields) = FIELDS.lock() else {
        return false;
    };
    let value = fields
        .get_or_insert_default()
        .entry(name.clone())
        .or_default();
    edit(value);
    if name == "customCss" {
        let css = value.clone();
        drop(fields);
        if let Some(path) = custom_css_path() {
            let _ = std::fs::write(path, &css);
        }
        apply_custom_css(css);
    }
    true
}

pub(super) fn text(action: &ResolvedAction, key: &str) -> Option<String> {
    match action.args.get(key)? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}
