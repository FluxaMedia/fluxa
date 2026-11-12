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

pub(crate) fn fixture_snapshot(live: &crate::Snapshot) -> Option<crate::Snapshot> {
    static SNAPSHOT: std::sync::OnceLock<Option<serde_json::Value>> = std::sync::OnceLock::new();
    let fixture = SNAPSHOT
        .get_or_init(|| {
            setting("FLUXA_PREVIEW_FIXTURE")?;
            let seed = match setting("FLUXA_PREVIEW_FIXTURE_FILE") {
                Some(path) => serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?,
                None => serde_json::from_str(include_str!("../../fixtures/native-ui-fixture.json"))
                    .ok()?,
            };
            Some(fluxa_core::Engine::new(seed).ok()?.snapshot())
        })
        .as_ref()?;
    let mut merged = fixture.clone();
    if let Some(navigation) = live.get("navigation") {
        merged["navigation"] = navigation.clone();
    }
    fixture_overlay(&mut merged);
    Some(crate::Snapshot::from_value(merged))
}

fn fixture_overlay(snapshot: &mut serde_json::Value) {
    let Some(target) = setting("FLUXA_PREVIEW") else {
        return;
    };
    let mut parts = target.split('/');
    match parts.next() {
        Some("calendar") => snapshot["calendar"] = calendar_fixture(),
        Some("library") if parts.next() == Some("downloads") => {
            snapshot["offline"] = downloads_fixture();
        }
        _ => {}
    }
}

fn calendar_fixture() -> serde_json::Value {
    let item = |id: &str, title: &str, episode: &str, date: &str| {
        serde_json::json!({
            "id": id,
            "type": "series",
            "title": title,
            "subtitle": episode,
            "dateIso": date,
        })
    };
    serde_json::json!({
        "year": 2026,
        "month": 11,
        "items": [
            item("fixture-show-a", "Fixture Show", "Pilot", "2026-11-03"),
            item("fixture-show-b", "Second Fixture", "Cold Open", "2026-11-12"),
            item("fixture-show-c", "Third Fixture", "The Long Night", "2026-11-12"),
            item("fixture-show-d", "Fixture Show", "Ledger", "2026-11-18"),
            item("fixture-show-e", "Fourth Fixture", "Harbor", "2026-11-24"),
            item("fixture-show-f", "Fixture Show", "Finale", "2026-11-27"),
        ],
    })
}

pub(crate) fn calendar(mut model: fluxa_ui::CalendarModel) -> fluxa_ui::CalendarModel {
    let Some(target) = setting("FLUXA_PREVIEW") else {
        return model;
    };
    let mut parts = target.split('/');
    if parts.next() != Some("calendar") {
        return model;
    }
    model.today = Some("2026-11-09".to_owned());
    model.selected_day = parts.next().and_then(|day| day.parse().ok());
    model
}

fn downloads_fixture() -> serde_json::Value {
    let row = |id: &str,
               meta: &str,
               name: &str,
               season: Option<u32>,
               episode: Option<u32>,
               title: &str,
               status: &str,
               done: u64,
               total: Option<u64>| {
        serde_json::json!({
            "id": id,
            "metaId": meta,
            "name": name,
            "season": season,
            "episode": episode,
            "episodeTitle": title,
            "status": status,
            "bytesDone": done,
            "bytesTotal": total,
            "percent": total.map_or(0.0, |total| done as f64 * 100.0 / total as f64),
            "speedBps": if status == "downloading" { 2_621_440.0 } else { 0.0 },
            "etaMs": if status == "downloading" { Some(160_000) } else { None },
            "elapsedMs": 90_000,
        })
    };
    let location = serde_json::json!({
        "path": "/home/fluxa/Downloads/Fluxa",
        "label": "Default",
        "freeBytes": 182_000_000_000_u64,
        "totalBytes": 500_000_000_000_u64,
    });
    serde_json::json!({
        "totalSpeedBps": 2_621_440.0,
        "history": [0.0, 900_000.0, 1_800_000.0, 2_400_000.0, 2_621_440.0, 2_100_000.0, 2_621_440.0],
        "location": location,
        "volumes": [location],
        "items": [
            row("dl-1", "fixture-show", "Fixture Show", Some(1), Some(1), "Pilot", "completed", 734_003_200, Some(734_003_200)),
            row("dl-2", "fixture-show", "Fixture Show", Some(1), Some(2), "Cold Open", "downloading", 300_000_000, Some(734_003_200)),
            row("dl-3", "fixture-show", "Fixture Show", Some(1), Some(3), "The Long Night", "queued", 0, None),
            row("dl-4", "fixture-film", "Fixture Film", None, None, "", "paused", 1_200_000_000, Some(2_400_000_000)),
            row("dl-5", "fixture-film-two", "Failed Fixture", None, None, "", "failed", 0, Some(2_400_000_000)),
        ],
    })
}

pub(crate) fn update() -> Option<(fluxa_ui::UpdateState, bool)> {
    let version = "1.8.0".to_owned();
    Some(match setting("FLUXA_PREVIEW_UPDATE")?.as_str() {
        "available" => (fluxa_ui::UpdateState::Available(version), false),
        "popup" => (fluxa_ui::UpdateState::Available(version), true),
        "downloading" => (fluxa_ui::UpdateState::Downloading(49), false),
        "ready" => (fluxa_ui::UpdateState::Ready(version), false),
        _ => return None,
    })
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
    let names = ["Anna", "Mike", "Kids", "Guest", "Daniel"];
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

pub(crate) fn play_demo(state: &mut RendererState) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    let Some(file) = setting("FLUXA_PREVIEW_PLAY") else {
        return;
    };
    if state.session.is_none() || DONE.swap(true, Ordering::Relaxed) {
        return;
    }
    crate::welcome::handle(state, fluxa_ui::WelcomeRequest::Offline);
    let Some(storage) = state
        .session
        .as_ref()
        .map(|session| session.storage().clone())
    else {
        return;
    };
    let first = crate::profiles::stored_profiles(&storage)
        .first()
        .and_then(|profile| profile.get("id")?.as_str().map(str::to_owned));
    if let Some(id) = first {
        crate::profiles::activate(state, &storage, &id);
    }
    let command = crate::player::open_file(state, &file, "HDR test");
    if let Some(session) = state.session.as_ref() {
        let _ = session.dispatch(command);
    }
}

pub(crate) fn decorate_player(model: &mut fluxa_ui::PlayerModel) {
    let Some(scene) = player_scene() else {
        return;
    };
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
    match scene.as_str() {
        "sources" => {
            model.passthrough = false;
            model.sources = Some(std::sync::Arc::new(sample_sources()));
        }
        "loading" => {
            model.passthrough = false;
            model.load_progress = Some(0.42);
            model.paused = false;
        }
        "recommendations" => {
            model.recommendations = [
                serde_json::json!({
                    "id": "fixture-featured",
                    "type": "movie",
                    "name": "Fixture Featured",
                    "year": 2026,
                    "runtime": "1h12min",
                    "genres": ["Family", "Action"],
                    "description": "Fixture recommendation description for visual testing.",
                }),
                serde_json::json!({
                    "id": "fixture-second",
                    "type": "series",
                    "name": "Second Fixture",
                    "year": 2025,
                    "description": "Second fixture recommendation for visual testing.",
                }),
            ]
            .iter()
            .map(|item| fluxa_ui::hero_from_meta(item, "en"))
            .collect();
        }
        "next" => {
            model.next_episode = Some(fluxa_ui::NextEpisodeCard {
                label: "S1:E4".to_owned(),
                title: "Fixture Episode Four".to_owned(),
                thumbnail: None,
                countdown: Some(7),
            });
        }
        _ => {}
    }
}

fn sample_sources() -> Vec<fluxa_ui::PlayerSource> {
    let badge = |label: &str| fluxa_ui::SourceBadge {
        label: label.to_owned(),
        ..Default::default()
    };
    let source = |addon: &str, name: &str, detail: &str, badges: Vec<fluxa_ui::SourceBadge>| {
        fluxa_ui::PlayerSource {
            addon: addon.to_owned(),
            name: name.to_owned(),
            detail: detail.to_owned(),
            badges,
            badges_top: false,
            actions: true,
        }
    };
    vec![
        source(
            "ExampleAddon",
            "Fixture 1080p WEB",
            "2.1 GB · English, Turkish",
            vec![badge("1080p"), badge("HDR")],
        ),
        source(
            "ExampleAddon",
            "Fixture 720p HDTV",
            "1.1 GB · English",
            vec![badge("720p")],
        ),
        source(
            "ExampleIndexer",
            "Fixture 2160p Remux",
            "38.4 GB · Japanese, English",
            vec![badge("4K"), badge("Dolby Vision")],
        ),
        source(
            "ExampleSource",
            "Fixture 480p",
            "640 MB · Turkish",
            vec![badge("480p")],
        ),
    ]
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

fn changed_file(
    variable: &str,
    seen: &std::sync::Mutex<Option<std::time::SystemTime>>,
) -> Option<String> {
    let path = setting(variable)?;
    if let Ok(mut next) = NEXT_CHECK.lock() {
        *next = Some(Instant::now() + Duration::from_millis(300));
    }
    let modified = std::fs::metadata(&path)
        .and_then(|file| file.modified())
        .ok()?;
    let mut seen = seen.lock().ok()?;
    if *seen == Some(modified) {
        return None;
    }
    *seen = Some(modified);
    Some(path)
}

pub(crate) fn reload_tokens() {
    static TOKENS: std::sync::Mutex<Option<std::time::SystemTime>> = std::sync::Mutex::new(None);
    static STYLES: std::sync::Mutex<Option<std::time::SystemTime>> = std::sync::Mutex::new(None);
    if let Some(path) = changed_file("FLUXA_PREVIEW_TOKENS", &TOKENS)
        && let Err(error) = fluxa_theme::theme::reload_shared_ui_tokens_from_path(&path)
    {
        host_log(format!("tokens not reloaded: {error:?}"));
    }
    if let Some(path) = changed_file("FLUXA_PREVIEW_STYLES", &STYLES) {
        let loaded = std::fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|json| fluxa_ui::reload_styles(&json));
        if let Err(error) = loaded {
            host_log(format!("styles not reloaded: {error}"));
        }
    }
}

pub(crate) fn enter_offline_profile(state: &mut RendererState) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    let Some(target) = setting("FLUXA_PREVIEW") else {
        return;
    };
    if target == "profiles"
        || target.starts_with("gallery")
        || state.session.is_none()
        || (state.welcome.is_none() && state.profiles.is_none())
        || DONE.swap(true, Ordering::Relaxed)
    {
        return;
    }
    crate::welcome::handle(state, fluxa_ui::WelcomeRequest::Offline);
    let Some(storage) = state
        .session
        .as_ref()
        .map(|session| session.storage().clone())
    else {
        return;
    };
    let first = crate::profiles::stored_profiles(&storage)
        .first()
        .and_then(|profile| profile.get("id")?.as_str().map(str::to_owned));
    if let Some(id) = first {
        crate::profiles::activate(state, &storage, &id);
    }
}

pub(crate) fn start(state: &mut RendererState) {
    if let Some(values) = setting("FLUXA_PREVIEW_SETTINGS").and_then(|raw| {
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw).ok()
    }) {
        for (key, value) in values {
            crate::stream_badges::set_setting(state, &key, value);
        }
    }
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
                name: "Anna".to_owned(),
                host: true,
                ..Default::default()
            }),
            members: vec![
                fluxa_ui::WatchMemberView {
                    name: "Anna".to_owned(),
                    host: true,
                    ..Default::default()
                },
                fluxa_ui::WatchMemberView {
                    name: "Marcus".to_owned(),
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
                member("Anna", "watching", true, true),
                member("Marcus", "paused", false, false),
                member("Daniel", "buffering", false, false),
                member("Can", "picking", false, false),
            ],
            chat: [
                ("Marcus", "This scene is legendary 🔥🔥", false),
                ("Marcus", "Look at Jesse's face 😂", false),
                (
                    "Anna",
                    "Wait for the buffer to finish, Daniel is lagging",
                    true,
                ),
                ("Daniel", "Tamam geldim 👍 🇹🇷 👨‍👩‍👧", false),
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
        state.scroll_set(ScrollKey::Screen(destination), scroll);
    }
    if let Ok(page) = first.parse() {
        state.settings.active_section = page;
        state.settings.section_open = true;
        state.settings.page_open =
            setting("FLUXA_PREVIEW_HUB").is_none() && parts.next() != Some("hub");
    }
    if destination == Route::Settings && first == "hub" {
        state.settings.section_open = false;
    }
    if destination == Route::Library && first == "downloads" {
        state.library_downloads = true;
    }
    if let Some(key) = setting("FLUXA_PREVIEW_MENU")
        && let Some(row) = (0..)
            .map_while(fluxa_ui::settings_row_by_index)
            .find(|row| row.key == key)
    {
        crate::card_menu::open_setting(state, row);
        if let Some(delta) =
            setting("FLUXA_PREVIEW_MENU_SCROLL").and_then(|value| value.parse().ok())
        {
            crate::card_menu::scroll_by(state, delta);
        }
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
