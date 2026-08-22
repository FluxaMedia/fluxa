use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

const FLUXA_DISCORD_APP_ID: &str = "1518004842860122174";
const RECONNECT_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Clone)]
struct Button {
    label: String,
    url: String,
}

#[derive(Clone)]
enum PresenceKind {
    Browsing {
        label: String,
    },
    Viewing {
        title: String,
        poster_url: Option<String>,
        button: Option<Button>,
    },
    Playing {
        title: String,
        detail: Option<String>,
        paused: bool,
        start_unix_secs: Option<i64>,
        end_unix_secs: Option<i64>,
        poster_url: Option<String>,
        button: Option<Button>,
    },
}

pub struct DiscordPresenceState {
    client: Mutex<Option<DiscordIpcClient>>,
    enabled: AtomicBool,
    last_activity: Mutex<PresenceKind>,
    reconnect_loop_started: AtomicBool,
}

impl Default for DiscordPresenceState {
    fn default() -> Self {
        Self {
            client: Mutex::new(None),
            enabled: AtomicBool::new(false),
            last_activity: Mutex::new(PresenceKind::Browsing {
                label: "Browsing".to_string(),
            }),
            reconnect_loop_started: AtomicBool::new(false),
        }
    }
}

fn activity_for<'a>(kind: &'a PresenceKind) -> activity::Activity<'a> {
    match kind {
        PresenceKind::Browsing { label } => activity::Activity::new().state(label).assets(
            activity::Assets::new()
                .large_image("logo")
                .large_text("Fluxa"),
        ),
        PresenceKind::Viewing {
            title,
            poster_url,
            button,
        } => {
            let large_image = poster_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .unwrap_or("logo");
            let mut act = activity::Activity::new()
                .details(title)
                .state("Viewing details")
                .assets(
                    activity::Assets::new()
                        .large_image(large_image)
                        .large_text(title)
                        .small_image("logo")
                        .small_text("Fluxa"),
                );
            if let Some(b) = button {
                act = act.buttons(vec![activity::Button::new(&b.label, &b.url)]);
            }
            act
        }
        PresenceKind::Playing {
            title,
            detail,
            paused,
            start_unix_secs,
            end_unix_secs,
            poster_url,
            button,
        } => {
            let state_text = detail.clone().unwrap_or_else(|| {
                if *paused {
                    "Paused".to_string()
                } else {
                    "Watching".to_string()
                }
            });
            let large_image = poster_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .unwrap_or("logo");
            let mut act = activity::Activity::new()
                .details(title)
                .state(state_text)
                .assets(
                    activity::Assets::new()
                        .large_image(large_image)
                        .large_text(title)
                        .small_image("logo")
                        .small_text("Fluxa"),
                );
            if start_unix_secs.is_some() || end_unix_secs.is_some() {
                let mut ts = activity::Timestamps::new();
                if let Some(start) = start_unix_secs {
                    ts = ts.start(*start);
                }
                if let Some(end) = end_unix_secs {
                    ts = ts.end(*end);
                }
                act = act.timestamps(ts);
            }
            if let Some(b) = button {
                act = act.buttons(vec![activity::Button::new(&b.label, &b.url)]);
            }
            act
        }
    }
}

fn apply(client: &mut DiscordIpcClient, kind: &PresenceKind) {
    let _ = client.set_activity(activity_for(kind));
}

pub fn spawn_reconnect_loop(app: &AppHandle) {
    let state = app.state::<DiscordPresenceState>();
    if state.reconnect_loop_started.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(RECONNECT_INTERVAL);
        let state = app.state::<DiscordPresenceState>();
        if !state.enabled.load(Ordering::SeqCst) {
            continue;
        }
        let mut guard = state.client.lock().unwrap();
        if guard.is_some() {
            continue;
        }
        let mut client = DiscordIpcClient::new(FLUXA_DISCORD_APP_ID);
        if client.connect().is_ok() {
            let kind = state.last_activity.lock().unwrap().clone();
            apply(&mut client, &kind);
            *guard = Some(client);
        }
    });
}

#[tauri::command]
pub fn discord_presence_configure(
    app: AppHandle,
    state: State<DiscordPresenceState>,
    enabled: bool,
) {
    state.enabled.store(enabled, Ordering::SeqCst);
    let mut guard = state.client.lock().unwrap();
    if let Some(mut old) = guard.take() {
        let _ = old.close();
    }
    if enabled {
        let mut client = DiscordIpcClient::new(FLUXA_DISCORD_APP_ID);
        if client.connect().is_ok() {
            let kind = state.last_activity.lock().unwrap().clone();
            apply(&mut client, &kind);
            *guard = Some(client);
        }
    }
    drop(guard);
    spawn_reconnect_loop(&app);
}

#[tauri::command]
pub fn discord_presence_update(
    state: State<DiscordPresenceState>,
    title: String,
    detail: Option<String>,
    paused: bool,
    start_unix_secs: Option<i64>,
    end_unix_secs: Option<i64>,
    poster_url: Option<String>,
    button_label: Option<String>,
    button_url: Option<String>,
) {
    let button = match (button_label, button_url) {
        (Some(label), Some(url)) if !label.is_empty() && !url.is_empty() => {
            Some(Button { label, url })
        }
        _ => None,
    };
    let kind = PresenceKind::Playing {
        title,
        detail,
        paused,
        start_unix_secs,
        end_unix_secs,
        poster_url,
        button,
    };
    *state.last_activity.lock().unwrap() = kind.clone();
    let mut guard = state.client.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        apply(client, &kind);
    }
}

#[tauri::command]
pub fn discord_presence_set_viewing(
    state: State<DiscordPresenceState>,
    title: String,
    poster_url: Option<String>,
    button_label: Option<String>,
    button_url: Option<String>,
) {
    let button = match (button_label, button_url) {
        (Some(label), Some(url)) if !label.is_empty() && !url.is_empty() => {
            Some(Button { label, url })
        }
        _ => None,
    };
    let kind = PresenceKind::Viewing {
        title,
        poster_url,
        button,
    };
    *state.last_activity.lock().unwrap() = kind.clone();
    let mut guard = state.client.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        apply(client, &kind);
    }
}

#[tauri::command]
pub fn discord_presence_set_browsing(state: State<DiscordPresenceState>, label: String) {
    let kind = PresenceKind::Browsing { label };
    *state.last_activity.lock().unwrap() = kind.clone();
    let mut guard = state.client.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        apply(client, &kind);
    }
}

#[tauri::command]
pub fn discord_presence_clear(state: State<DiscordPresenceState>) {
    *state.last_activity.lock().unwrap() = PresenceKind::Browsing {
        label: "Browsing".to_string(),
    };
    let mut guard = state.client.lock().unwrap();
    if let Some(client) = guard.as_mut() {
        let _ = client.clear_activity();
    }
}
