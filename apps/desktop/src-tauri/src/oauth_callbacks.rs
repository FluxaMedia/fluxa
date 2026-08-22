use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

#[derive(Clone, serde::Serialize)]
pub struct OAuthCodePayload {
    code: String,
    state: Option<String>,
}

pub struct PendingOAuthCallbacks {
    state: Mutex<OAuthCallbackState>,
}

struct OAuthCallbackState {
    callbacks: HashMap<String, OAuthCodePayload>,
    consumed: HashSet<String>,
}

impl Default for PendingOAuthCallbacks {
    fn default() -> Self {
        Self {
            state: Mutex::new(OAuthCallbackState {
                callbacks: HashMap::new(),
                consumed: HashSet::new(),
            }),
        }
    }
}

pub fn queue_oauth_callback(
    app: &tauri::AppHandle,
    service: &str,
    code: String,
    state: Option<String>,
) {
    let event = match service {
        "trakt" => "trakt-oauth-code",
        "anilist" => "anilist-oauth-code",
        "simkl" => "simkl-oauth-code",
        _ => return,
    };
    let payload = OAuthCodePayload { code, state };
    let callback_id = format!("{service}:{}", payload.code);
    if let Ok(mut state) = app.state::<PendingOAuthCallbacks>().state.lock() {
        if state.consumed.contains(&callback_id) {
            return;
        }
        state.callbacks.insert(service.to_string(), payload.clone());
    }
    let _ = app.emit(event, payload);
}

#[tauri::command]
pub fn take_oauth_callback(
    service: String,
    callbacks: State<PendingOAuthCallbacks>,
) -> Result<Option<OAuthCodePayload>, String> {
    match service.as_str() {
        "trakt" | "anilist" | "simkl" => {
            let mut state = callbacks
                .state
                .lock()
                .map_err(|_| "OAuth callback state is unavailable".to_string())?;
            let payload = state.callbacks.remove(&service);
            if let Some(payload) = &payload {
                state.consumed.insert(format!("{service}:{}", payload.code));
            }
            Ok(payload)
        }
        _ => Err("unsupported OAuth service".to_string()),
    }
}
