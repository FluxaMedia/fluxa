use super::*;

fn setting(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|_| cfg!(debug_assertions))
}

pub fn size() -> Option<(f64, f64)> {
    let size = setting("FLUXA_PREVIEW_SIZE")?;
    let (width, height) = size.split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

pub(crate) fn form_factor(requested: &str) -> String {
    setting("FLUXA_PREVIEW_FORM").unwrap_or_else(|| requested.to_owned())
}

pub(crate) fn fixture_snapshot(
    live: &serde_json::Value,
) -> Option<std::sync::Arc<serde_json::Value>> {
    static SNAPSHOT: std::sync::OnceLock<Option<serde_json::Value>> = std::sync::OnceLock::new();
    let fixture = SNAPSHOT
        .get_or_init(|| {
            setting("FLUXA_PREVIEW_FIXTURE")?;
            let seed =
                serde_json::from_str(include_str!("../../fixtures/native-ui-fixture.json")).ok()?;
            Some(fluxa_core::Engine::new(seed).ok()?.snapshot())
        })
        .as_ref()?;
    let mut merged = fixture.clone();
    if let Some(navigation) = live.get("navigation") {
        merged["navigation"] = navigation.clone();
    }
    Some(std::sync::Arc::new(merged))
}

pub(crate) fn gallery() -> Option<String> {
    let target = setting("FLUXA_PREVIEW")?;
    let scene = target.strip_prefix("gallery")?;
    Some(scene.trim_start_matches('/').to_owned())
}

pub(crate) fn seed_profiles(storage: &fluxa_effects::Storage) {
    if setting("FLUXA_PREVIEW").as_deref() != Some("profiles") {
        return;
    }
    let names = ["Ayşe", "Mehmet", "Kids", "Guest", "Deniz"];
    let profiles = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            serde_json::json!({
                "id": format!("preview-{index}"),
                "name": name,
                "pinHash": if index == 1 { "x" } else { "" },
            })
        })
        .collect::<Vec<_>>();
    crate::profiles::write(storage, "profiles", &serde_json::Value::Array(profiles));
    crate::profiles::write(storage, "active_profile_id", &serde_json::json!(""));
    crate::profiles::write(storage, "welcome_offline", &serde_json::json!(true));
}

pub(crate) fn player_scene() -> Option<String> {
    let target = setting("FLUXA_PREVIEW")?;
    let scene = target.strip_prefix("player")?;
    Some(scene.trim_start_matches('/').to_owned())
}

pub(crate) fn player_demo(state: &mut RendererState) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    let Some(scene) = player_scene() else {
        return;
    };
    if state.session.is_none() || DONE.swap(true, Ordering::Relaxed) {
        return;
    }
    crate::welcome::handle(state, fluxa_ui::WelcomeRequest::Offline);
    crate::player::open_file(state, "about:blank", "Demo");
    crate::player::preview_scene(state, &scene);
}

pub(crate) fn decorate_player(model: &mut fluxa_ui::PlayerModel) {
    if player_scene().is_none() {
        return;
    }
    model.passthrough = true;
    model.controls_visible = true;
    model.paused = true;
    model.duration = 2712.0;
    model.position = 754.0;
    model.title = "Fenerin Ötesi".to_owned();
    model.season_episode = Some((1, 3));
    model.episode_title = Some("Alçak Basınç".to_owned());
    model.chapters = vec![
        (0.0, "Açılış".to_owned()),
        (95.0, "Fener Bekçisi".to_owned()),
        (610.0, "Fırtına".to_owned()),
    ];
    model.has_next = true;
    model.has_previous = true;
    model.has_episodes = true;
}

pub(crate) fn watch_demo() -> bool {
    setting("FLUXA_PREVIEW_WATCH").is_some()
}

pub(crate) fn outlines() -> bool {
    setting("FLUXA_PREVIEW_OUTLINE").is_some()
}

static NEXT_CHECK: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);

pub(crate) fn next_check() -> Option<Instant> {
    *NEXT_CHECK.lock().ok()?
}

pub(crate) fn reload_tokens() {
    static SEEN: std::sync::Mutex<Option<std::time::SystemTime>> = std::sync::Mutex::new(None);
    let Some(path) = setting("FLUXA_PREVIEW_TOKENS") else {
        return;
    };
    if let Ok(mut next) = NEXT_CHECK.lock() {
        *next = Some(Instant::now() + Duration::from_millis(300));
    }
    let Ok(modified) = std::fs::metadata(&path).and_then(|file| file.modified()) else {
        return;
    };
    let Ok(mut seen) = SEEN.lock() else {
        return;
    };
    if *seen == Some(modified) {
        return;
    }
    *seen = Some(modified);
    if let Err(error) = fluxa_theme::theme::reload_shared_ui_tokens_from_path(&path) {
        host_log(format!("tokens not reloaded: {error:?}"));
    }
}

pub(crate) fn start(state: &mut RendererState) {
    if let Some(url) = setting("FLUXA_PREVIEW_ADDON") {
        state
            .pending_native_actions
            .push(NativeAction::CoreCommand {
                command: serde_json::json!({
                    "type": "addonInstallRequested",
                    "transportUrl": url,
                    "forceRefresh": false,
                }),
            });
    }
    if setting("FLUXA_PREVIEW_WATCH").as_deref() == Some("ticket") {
        state.settings.watch_ticket = fluxa_ui::WatchTicket {
            room: Some("K7M2QX".to_owned()),
            title: Some("Breaking Bad".to_owned()),
            backdrop: Some(
                "https://image.tmdb.org/t/p/w1280/tsRy63Mu5cu8etL1X7ZLyf7UP1M.jpg".to_owned(),
            ),
            episode: Some("S1 · E2 · Cat's in the Bag...".to_owned()),
            host: Some(fluxa_ui::WatchMemberView {
                name: "Ayşe".to_owned(),
                host: true,
                ..Default::default()
            }),
            members: vec![
                fluxa_ui::WatchMemberView {
                    name: "Ayşe".to_owned(),
                    host: true,
                    ..Default::default()
                },
                fluxa_ui::WatchMemberView {
                    name: "Mert".to_owned(),
                    ..Default::default()
                },
            ],
            max_members: 4,
            ..Default::default()
        };
        state.settings.watch_room = Some("K7M2QX".to_owned());
    }
    if watch_demo() && setting("FLUXA_PREVIEW_WATCH").as_deref() != Some("ticket") {
        let member = |name: &str, status: &str, host: bool, you: bool| fluxa_ui::WatchMemberView {
            id: name.to_owned(),
            name: name.to_owned(),
            avatar: None,
            ready: status != "picking",
            buffering: status == "buffering",
            status: status.to_owned(),
            host,
            you,
        };
        state.watch_view = Some(fluxa_ui::WatchTogetherModel {
            language: state.settings.language().to_owned(),
            room_code: Some("K7M2QX".to_owned()),
            max_members: 6,
            host: true,
            members: vec![
                member("Ayşe", "watching", true, true),
                member("Mert", "paused", false, false),
                member("Deniz", "buffering", false, false),
                member("Can", "picking", false, false),
            ],
            chat: [
                ("Mert", "Bu sahne efsane 🔥🔥", false),
                ("Mert", "Jesse'nin yüzüne bak 😂", false),
                ("Ayşe", "Bekle buffer bitsin, Deniz takıldı", true),
                ("Deniz", "Tamam geldim 👍 🇹🇷 👨‍👩‍👧", false),
            ]
            .into_iter()
            .enumerate()
            .map(|(id, (name, text, you))| fluxa_ui::WatchChatView {
                id: id as u64,
                by: name.to_owned(),
                avatar: None,
                name: name.to_owned(),
                text: text.to_owned(),
                you,
            })
            .collect(),
            ..Default::default()
        });
    }
    let Some(target) = setting("FLUXA_PREVIEW") else {
        return;
    };
    let mut parts = target.split('/');
    let destination = Route::parse(parts.next().unwrap_or_default());
    let first = parts.next().unwrap_or_default();
    if destination == Route::Detail {
        state.pending_native_actions.push(NativeAction::Detail {
            item_type: first.to_owned(),
            id: parts.next().unwrap_or_default().to_owned(),
            preview: Value::Null,
        });
    } else if destination != Route::Home {
        state
            .pending_native_actions
            .push(NativeAction::Navigate { destination });
    }
    if let Some(scroll) = setting("FLUXA_PREVIEW_SCROLL").and_then(|value| value.parse().ok()) {
        state.screen_scroll_offsets.insert(destination, scroll);
    }
    if let Ok(page) = first.parse() {
        state.settings.active_section = page;
        state.settings.section_open = true;
        state.settings.page_open = true;
    }
}

pub(crate) fn watch_panel_demo(state: &mut RendererState) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    if setting("FLUXA_PREVIEW_WATCH").as_deref() != Some("panel")
        || state.session.is_none()
        || DONE.swap(true, Ordering::Relaxed)
    {
        return;
    }
    crate::welcome::handle(state, fluxa_ui::WelcomeRequest::Offline);
    crate::player::open_file(state, "about:blank", "Demo");
    state.watch_panel = true;
}
