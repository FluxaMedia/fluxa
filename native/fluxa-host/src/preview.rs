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
    if watch_demo() {
        let member = |name: &str, ready: bool, buffering: bool, host: bool, you: bool| {
            fluxa_ui::WatchMemberView {
                id: name.to_owned(),
                name: name.to_owned(),
                avatar: None,
                ready,
                buffering,
                host,
                you,
            }
        };
        state.watch_view = Some(fluxa_ui::WatchTogetherModel {
            language: state.settings.language().to_owned(),
            room_code: Some("K7M2QX".to_owned()),
            host: true,
            members: vec![
                member("Ayşe", true, false, true, true),
                member("Mert", true, false, false, false),
                member("Deniz", false, true, false, false),
                member("Can", false, false, false, false),
            ],
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
