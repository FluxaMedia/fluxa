use super::*;
use crate::navigation::nav_text;
use std::sync::Mutex;

static STATE: Mutex<UpdateState> = Mutex::new(UpdateState::Idle);

pub fn set_nav_update(state: &UpdateState) {
    if let Ok(mut current) = STATE.lock()
        && *current != *state
    {
        *current = state.clone();
    }
}

pub(crate) struct Status<'a> {
    pub title: String,
    pub detail: String,
    pub progress: Option<f32>,
    pub icon: &'a str,
}

pub(crate) fn nav_status() -> Option<Status<'static>> {
    let state = STATE.lock().ok()?.clone();
    Some(match state {
        UpdateState::Available(version) => Status {
            title: nav_text("settings.updates.available"),
            detail: format!("v{version}"),
            progress: None,
            icon: "Downloads",
        },
        UpdateState::Downloading(percent) => Status {
            title: nav_text("update.downloading"),
            detail: format!("{percent}%"),
            progress: Some(f32::from(percent) / 100.0),
            icon: "Downloads",
        },
        UpdateState::Ready(version) => Status {
            title: nav_text("settings.updates.restart"),
            detail: format!("v{version}"),
            progress: None,
            icon: "Check",
        },
        UpdateState::Failed(_) => Status {
            title: nav_text("settings.updates.failed"),
            detail: nav_text("common.retry"),
            progress: None,
            icon: "Refresh",
        },
        _ => return None,
    })
}

pub(crate) fn update_data() -> serde_json::Value {
    match nav_status() {
        Some(status) => serde_json::json!({
            "active": true,
            "icon": status.icon,
            "ring": status.progress.unwrap_or(-1.0),
            "title": status.title,
            "detail": status.detail,
        }),
        None => serde_json::json!({"active": false, "icon": "", "ring": -1.0}),
    }
}

pub(crate) fn state_sig(hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    if let Ok(state) = STATE.lock() {
        format!("{state:?}").hash(hasher);
    }
}
