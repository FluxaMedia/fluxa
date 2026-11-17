use super::account::{
    ACCOUNT_PROVIDERS, ACCOUNT_SERVICES, ACCOUNT_SOURCES, SYNC_SOURCES, account_connected,
    account_source_state,
};
use super::*;
use serde_json::{Value, json};

fn group(heading: String, rows: Vec<Value>) -> Value {
    json!({"heading": heading, "rows": rows})
}

fn service_row(settings: &SettingsModel, label: &str, key: &str, language: &str) -> Value {
    let provider = label.to_lowercase();
    let handle = settings
        .tracker_names
        .iter()
        .find(|(name, _)| *name == provider);
    let on = account_connected(settings, key) && !handle.is_some_and(|(_, name)| name.is_empty());
    let prompt = settings
        .account_auth
        .as_ref()
        .filter(|prompt| prompt.provider == provider);
    let status = match prompt {
        Some(prompt) if prompt.failed => localized("settings.account_sign_in_failed", language),
        Some(prompt) => format!(
            "{} {}  ·  {}",
            localized("trakt.device.enter_code", language),
            prompt.code,
            prompt.url
        ),
        None => match handle.filter(|_| on) {
            Some((_, name)) => localized("settings.connected_as", language).replace("%s", name),
            None => localized(
                if on {
                    "settings.connected"
                } else {
                    "settings.not_connected"
                },
                language,
            ),
        },
    };
    let slot = ACCOUNT_PROVIDERS.iter().position(|name| *name == provider);
    json!({
        "kind": "service",
        "logo": provider,
        "title": label,
        "hint": status,
        "button": localized(
            if on { "settings.account_disconnect" } else { "settings.account_connect" },
            language,
        ),
        "primary": !on,
        "node": slot.map_or(0, |slot| NODE_SETTINGS_ACCOUNT_BASE + slot as u64).to_string(),
    })
}

fn choice_row(settings: &SettingsModel, label_key: &str, key: &str, language: &str) -> Value {
    let (options, current, label) = account_source_state(settings, key);
    json!({
        "kind": "choice",
        "title": localized(label_key, language),
        "key": key,
        "value": current,
        "current": label,
        "options": options
            .into_iter()
            .map(|(value, label)| json!({"value": value, "label": label}))
            .collect::<Vec<_>>(),
    })
}

fn account(settings: &SettingsModel, language: &str) -> Vec<Value> {
    let name = settings
        .profile
        .get("name")
        .or_else(|| settings.profile.get("displayName"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| localized("settings.account_empty", language));
    let avatar = settings
        .profile
        .get("avatarUrl")
        .and_then(Value::as_str)
        .filter(|url| !url.starts_with("data:"))
        .unwrap_or_default();
    let profile = group(
        localized("settings.group.account_profile", language),
        vec![
            json!({"kind": "profile", "title": name, "image": avatar}),
            json!({
                "kind": "link",
                "title": localized("settings.switch_profiles", language),
                "hint": localized("settings.switch_profiles_desc", language),
                "chevron": true,
                "node": NODE_SETTINGS_SWITCH_PROFILE.to_string(),
            }),
            json!({
                "kind": "link",
                "title": localized("auto.log_out", language),
                "danger": true,
                "node": NODE_SETTINGS_LOG_OUT.to_string(),
            }),
        ],
    );
    let services = group(
        localized("settings.group.account_services", language),
        ACCOUNT_SERVICES
            .iter()
            .map(|(label, key)| service_row(settings, label, key, language))
            .collect(),
    );
    let sync = group(
        localized("settings.group.account_sync", language),
        ACCOUNT_SOURCES
            .iter()
            .take(SYNC_SOURCES)
            .map(|(label_key, key)| choice_row(settings, label_key, key, language))
            .collect(),
    );
    vec![profile, services, sync]
}

fn about(settings: &SettingsModel, language: &str) -> Vec<Value> {
    let (status, action) = super::about::about_update(&settings.update, language);
    let hero = json!({
        "kind": "about",
        "version": settings.update.current,
        "hint": status,
        "created": format!(
            "{} {}",
            localized("settings.about.created_by", language),
            localized("settings.about.owner_name", language)
        ),
        "button": action.map(|key| localized(key, language)).unwrap_or_default(),
        "node": NODE_SETTINGS_UPDATE_ACTION.to_string(),
    });
    let links = super::about::ABOUT_LINKS
        .iter()
        .map(|link| {
            json!({
                "kind": "link",
                "logo": link.logo,
                "title": localized(link.title, language),
                "hint": localized(link.subtitle, language),
                "chevron": true,
                "node": link.node.to_string(),
            })
        })
        .collect();
    vec![
        group(String::new(), vec![hero]),
        group(localized("settings.about.project", language), links),
    ]
}

pub(super) fn page_blocks(settings: &SettingsModel, title: &str, language: &str) -> Vec<Value> {
    match title {
        "Account" => account(settings, language),
        "About" => about(settings, language),
        "Shortcuts" => shortcuts(settings, language),
        "Watch Party" => watch_party(settings, language),
        "Add-ons" => super::addons::addons_blocks(settings, language),
        "Plugins" => super::addons::plugins_blocks(settings, language),
        "Servers" => super::addons::servers_blocks(settings, language),
        "Streamkit Addons" => super::addons::streamkit_blocks(settings, language),
        _ => Vec::new(),
    }
}

fn shortcuts(settings: &SettingsModel, language: &str) -> Vec<Value> {
    let controller = group(
        localized("settings.group.controller", language),
        CONTROLLER_BINDINGS
            .iter()
            .map(|(action, keys)| {
                json!({"kind": "keys", "title": localized(action, language), "keys": keys, "node": ""})
            })
            .collect(),
    );
    let mut groups = Vec::new();
    if settings.gamepad_active {
        groups.push(controller.clone());
    }
    for (title, player) in [
        ("settings.shortcuts_group_general", false),
        ("settings.shortcuts_group_player", true),
    ] {
        let rows = settings
            .shortcuts
            .iter()
            .enumerate()
            .filter(|(_, row)| row.player == player)
            .map(|(index, shortcut)| {
                let recording =
                    settings.shortcut_recording.as_deref() == Some(shortcut.id.as_str());
                let keys = if recording {
                    localized("settings.shortcuts_recording", language)
                } else if shortcut.label.is_empty() {
                    localized("settings.shortcuts_unassigned", language)
                } else {
                    shortcut.label.clone()
                };
                json!({
                    "kind": "keys",
                    "title": localized(&format!("shortcuts.{}", shortcut.id), language),
                    "keys": keys,
                    "node": (NODE_SETTINGS_SHORTCUT_BASE + index as u64).to_string(),
                    "reset": shortcut.custom.then(|| json!({
                        "label": localized("settings.shortcuts_reset_one", language),
                        "node": (NODE_SETTINGS_SHORTCUT_RESET_BASE + index as u64).to_string(),
                    })),
                })
            })
            .collect();
        groups.push(group(localized(title, language), rows));
    }
    groups.push(group(
        localized("settings.shortcuts_reset_all", language),
        vec![json!({
            "kind": "button",
            "button": localized("settings.shortcuts_reset_all", language),
            "node": NODE_SETTINGS_SHORTCUT_RESET_ALL.to_string(),
            "fill": "",
        })],
    ));
    if !settings.gamepad_active {
        groups.push(controller);
    }
    groups
}

fn watch_party(settings: &SettingsModel, language: &str) -> Vec<Value> {
    let ticket = &settings.watch_ticket;
    let mut rows = Vec::new();
    if let Some(error) = ticket.error.as_deref() {
        rows.push(json!({"kind": "note", "title": error, "danger": true}));
    }
    if let Some(room) = ticket.room.as_deref() {
        rows.push(json!({
            "kind": "item",
            "title": localized("player.watch_party_room", language),
            "hint": room,
        }));
        rows.extend(ticket.members.iter().map(|member| {
            json!({
                "kind": "item",
                "title": member.name,
                "hint": member.status,
            })
        }));
    } else {
        rows.push(json!({
            "kind": "field",
            "title": localized("settings.watch_party.join_title", language),
            "key": "watchCode",
            "value": settings.watch_code,
            "placeholder": localized("watch.code_hint", language),
            "inputKind": "text",
        }));
        rows.push(json!({"kind": "note", "title": localized("settings.watch_party.join_help", language), "danger": false}));
        rows.push(json!({
            "kind": "button",
            "button": localized("watch.join", language),
            "node": NODE_SETTINGS_WATCH_JOIN.to_string(),
            "fill": format!("{NODE_SETTINGS_WATCH_CODE}=watchCode"),
        }));
    }
    rows.push(json!({"kind": "note", "title": localized("settings.watch_party.host_help", language), "danger": false}));
    vec![group(String::new(), rows)]
}
